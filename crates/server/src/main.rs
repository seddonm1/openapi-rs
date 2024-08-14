pub mod api;
pub mod context;
pub mod database;
pub mod entity;
pub mod error;
pub mod imp;
pub mod kratos;

// This module contains test cases and is only compiled when testing.
#[cfg(test)]
pub mod test;

use anyhow::{anyhow, Result};
use context::Context;
use database::Database;
use dropshot::{ConfigDropshot, ConfigLogging, ConfigLoggingLevel, HttpServer, HttpServerStarter};
use kratos::Kratos;
use std::{net::SocketAddr, path::Path};

#[tokio::main]
async fn main() -> Result<()> {
    // Collect command line arguments passed to the program.
    let args: Vec<String> = std::env::args().collect();

    // Generate the OpenAPI specification and write it to a file.
    std::fs::write(
        Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("api/v1.json"),
        api::generate_openapi_spec(),
    )
    .unwrap_or_else(|err| panic!("failed to write openapi spec: {}", err));

    // Create a database with 1 writer and num_cpus readers. The path is provided
    // as an argument or defaults to "./db.sqlite". This is done asynchronously.
    let database = Database::open(
        args.get(2).unwrap_or(&"./db.sqlite".to_string()),
        num_cpus::get(),
    )
    .await?;

    let kratos = Kratos::new(4433, 4434);

    // Create a context using the provided database.
    let context = Context::new(database, kratos);

    // Start the server listening on 127.0.0.1:8080 and inject the created database
    // into the context.
    let server = create_server("127.0.0.1:8080".parse().unwrap(), context)?;

    // Wait for the database to stop and return any error that occurred during this process.
    server.await.map_err(|err| anyhow!(err))
}

// Function to create an HttpServer instance with a given bind address and database.
// The context is shared among API endpoints, and a logger is configured as well.
pub fn create_server(bind_address: SocketAddr, context: Context) -> Result<HttpServer<Context>> {
    // Configure a logger that writes to stderr at the "info" level. If an error occurs
    // during logger creation, it's returned as an Anyhow error.
    let config_logging = ConfigLogging::StderrTerminal {
        level: ConfigLoggingLevel::Info,
    };
    let log = config_logging
        .to_logger("example-basic")
        .map_err(|err| anyhow!("failed to create logger: {}", err))?;

    // Set up the server with configuration, API description, shared context, and logger.
    Ok(HttpServerStarter::new(
        &ConfigDropshot {
            bind_address,
            ..Default::default()
        },
        api::server_api_mod::api_description::<imp::ServerImpl>().unwrap(),
        context,
        &log,
    )
    // If there's an error during server creation, return it as an Anyhow error.
    .map_err(|err| anyhow!("failed to create server: {}", err))?
    .start())
}
