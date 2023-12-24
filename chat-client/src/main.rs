#![warn(clippy::all)]
// #![warn(clippy::nursery)]
// #![warn(clippy::pedantic)]

use std::net::SocketAddr;

use color_eyre::Result;
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

mod cli;
mod network;
mod types;

#[tokio::main]
async fn main() -> Result<()> {
    setup()?;

    let addr: SocketAddr = std::env::var("ADDRESS")
        .expect("Environment variable `ADDRESS` must be set.")
        .parse()?;

    network::handle_connection(&addr).await?;

    Ok(())
}

fn setup() -> Result<()> {
    dotenvy::dotenv().expect(".env file not found");
    color_eyre::install()?;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt()
        .without_time()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    Ok(())
}
