use crate::{context::Context, create_server, database::Database, kratos::Kratos};
use anyhow::{anyhow, Result};
use dropshot::HttpServer;
use rusqlite::Transaction;
use std::{
    net::{SocketAddr, TcpListener},
    path::Path,
    process::Stdio,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    task::JoinHandle,
    time::{sleep, timeout},
};

static AVAILABLE_PORT: Mutex<u16> = Mutex::new(0);

// This module generates the OpenAPI client inline.
pub mod client {
    progenitor::generate_api!("api/v1.json");
}

/// Fixture trait allows tests to configure database state before a test begins.
///
/// Each fixture must implement this trait and provide:
/// - `try_fixtures`: A function that takes an active database transaction and applies any necessary setup changes. If there's an error, it should return a ResultErr with the specific issue.
pub trait Fixture: Send + Sync + 'static {
    fn try_fixtures(&self, transaction: &Transaction<'_>) -> Result<()>;
}

/// Finds the first available pair of TCP ports in the range 8000 to 8999.
///
/// Returns a tuple containing two `u16` values representing the start and end
/// of the available port range. If no such range is found, it will panic.
fn get_available_ports() -> (u16, u16) {
    let mut available_port = AVAILABLE_PORT.lock().unwrap();
    (8000..9000)
        .map(|port| (port, port + 1))
        .find(|(port0, port1)| {
            match (
                TcpListener::bind(("127.0.0.1", *port0)),
                TcpListener::bind(("127.0.0.1", *port1)),
            ) {
                (Ok(_), Ok(_)) if *port0 > *available_port => {
                    *available_port = *port1;
                    true
                }
                _ => false,
            }
        })
        .unwrap()
}

/// TestContext allows each test to run in isolation and therefore in parallel.
///
/// It holds:
/// - A server instance for the current test (Arc wrapped for thread safety).
/// - An in-memory database instance for the current test.
/// - An in-memory kratos instance for the current test.
pub struct TestContext {
    /// The server started for this individual test. Arc wrapped for thread safety.
    pub server: Arc<HttpServer<Context>>,

    /// The in-memory kratos handle for this individual test.
    pub kratos_handle: JoinHandle<()>,

    /// The in-memory database instance for this individual test.
    pub database: Database,

    /// The kratos instance for this individual test.
    pub kratos: Kratos,
}

impl TestContext {
    /// Create a new TestContext and apply any requested Fixtures to the database asynchronously.
    ///
    /// This function creates an in-memory database, applies fixtures, starts a server, and spawns it as a task.
    pub async fn new(fixtures: Vec<Box<dyn Fixture>>) -> Result<Self> {
        // Open an in-memory database instance for this test
        let database = Database::open_in_memory(1).await?;

        // Write transaction to apply fixtures and commit changes
        database
            .write(move |connection| {
                let tx = connection.transaction()?;
                for fixture in fixtures {
                    // Run each fixture's setup on the transaction
                    fixture.try_fixtures(&tx)?;
                }
                Ok(tx.commit()?)
            })
            .await?;

        // find two sequential available ports then start a process in a new long lived task
        let (public_port, admin_port) = get_available_ports();
        let mut base_directory = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        base_directory.push("build/kratos");

        let kratos_handle = tokio::spawn(async move {
            tokio::process::Command::new("./kratos")
                .current_dir(base_directory)
                .arg("serve")
                .arg("--config")
                .arg("./kratos.yml")
                .arg("--dev")
                .arg("--watch-courier")
                .env(
                    "DSN",
                    "sqlite://file:ignored?mode=memory&cache=shared&_fk=true",
                )
                .env("SERVE_PUBLIC_PORT", public_port.to_string())
                .env("SERVE_ADMIN_PORT", admin_port.to_string())
                .kill_on_drop(true)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap()
                .wait()
                .await
                .ok();
        });
        // wait until the health check passes
        timeout(Duration::from_millis(5000), async {
            loop {
                if let Ok(response) =
                    reqwest::get(format!("http://localhost:{}/health/alive", public_port)).await
                {
                    if response.status() == reqwest::StatusCode::OK {
                        break;
                    }
                };
                sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .map_err(|_| anyhow!("duration waiting for kratos healthcheck elapsed"))?;

        let kratos = Kratos::new(public_port, admin_port);

        let context = Context::new(database.clone(), kratos.clone());

        // Create a server bound to a random available port and spawn it as a task
        let server = Arc::new(create_server("127.0.0.1:0".parse().unwrap(), context)?);
        let server_clone = server.clone();
        tokio::spawn(async { server_clone });

        Ok(Self {
            server,
            database,
            kratos,
            kratos_handle,
        })
    }

    /// Get the local address that the server is bound to
    pub fn bind_address(&self) -> SocketAddr {
        self.server.local_addr()
    }

    /// Returns a reference to the in-memory database instance for this test.
    pub fn database(&self) -> &Database {
        &self.database
    }

    /// Returns a reference to the in-memory database instance for this test.
    pub fn kratos(&self) -> &Kratos {
        &self.kratos
    }

    /// Returns an OpenAPI client connected to the server started for this test, with optional headers.
    ///
    /// If no headers are provided, it uses default headers. The API endpoint is derived from the server's bind address.
    pub fn client(&self, headers: Option<reqwest::header::HeaderMap>) -> client::Client {
        client::Client::new_with_client(
            &format!("http://{}", self.server.local_addr()),
            reqwest::Client::builder()
                .default_headers(headers.unwrap_or_default())
                .build()
                .unwrap(),
        )
    }
}
