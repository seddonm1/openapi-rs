pub mod context;
pub mod database;
pub mod entity;
pub mod error;
pub mod routes;

use anyhow::anyhow;
use anyhow::Result;
use context::Context;
use database::Database;
use dropshot::ApiDescription;
use dropshot::ConfigDropshot;
use dropshot::ConfigLogging;
use dropshot::ConfigLoggingLevel;
use dropshot::HttpServer;
use dropshot::HttpServerStarter;
use routes::counter;
use std::net::SocketAddr;

#[allow(dead_code)]
#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Create a databawse with 1 writer and num_cpus readers.
    let database = Database::open(
        args.get(2).unwrap_or(&"./db.sqlite".to_string()),
        num_cpus::get(),
    )
    .await?;

    // Start the server listening on :8080 and inject the database into the context.
    let server = create_server("127.0.0.1:8080".parse().unwrap(), database)?;

    // Wait for the database to stop and return any error.
    server.await.map_err(|err| anyhow!(err))
}

pub fn create_server(bind_address: SocketAddr, database: Database) -> Result<HttpServer<Context>> {
    // For simplicity, we'll configure an "info"-level logger that writes to
    // stderr assuming that it's a terminal.
    let config_logging = ConfigLogging::StderrTerminal {
        level: ConfigLoggingLevel::Info,
    };
    let log = config_logging
        .to_logger("example-basic")
        .map_err(|error| anyhow!("failed to create logger: {}", error))?;

    // The functions that implement our API endpoints will share this context.
    let context = Context::new(database);

    // Build a description of the API.
    let mut api = ApiDescription::new();
    api.register(counter::get_counter).unwrap();
    api.register(counter::put_counter).unwrap();

    // Set up the server.
    Ok(HttpServerStarter::new(
        &ConfigDropshot {
            bind_address,
            ..Default::default()
        },
        api,
        context,
        &log,
    )
    .map_err(|error| anyhow!("failed to create server: {}", error))?
    .start())
}
