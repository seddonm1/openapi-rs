pub mod counter;

#[cfg(test)]
pub mod client {
    // Generate the OpenAPI client inline.
    progenitor::generate_api!("api/v1.json");
}

#[allow(clippy::module_inception)]
#[cfg(test)]
pub mod test {
    use crate::{context::Context, create_server, database::Database, entity::test::Fixture};
    use anyhow::Result;
    use dropshot::HttpServer;
    use std::{net::SocketAddr, sync::Arc};

    /// TestContext allows each test to run in isolation and therefore in parallel.
    pub struct TestContext {
        /// The server started for this individual test.
        server: Arc<HttpServer<Context>>,
        /// The in-memory database instance for this idividual test.
        database: Database,
    }

    impl TestContext {
        /// Create a new TestContext and apply any requested Fixtures to the database.
        pub async fn new(fixtures: Vec<Box<dyn Fixture>>) -> Result<Self> {
            let database = Database::open_in_memory(1).await?;
            database
                .write(move |connection| {
                    // run fixtures
                    let tx = connection.transaction()?;
                    for fixture in fixtures {
                        fixture.try_fixtures(&tx)?;
                    }
                    Ok(tx.commit()?)
                })
                .await?;

            let server = Arc::new(create_server(
                "127.0.0.1:0".parse().unwrap(),
                database.to_owned(),
            )?);
            let server_clone = server.clone();
            tokio::spawn(async { server_clone });
            Ok(Self { server, database })
        }

        pub fn bind_address(&self) -> SocketAddr {
            self.server.local_addr()
        }

        pub fn database(&self) -> &Database {
            &self.database
        }
    }
}
