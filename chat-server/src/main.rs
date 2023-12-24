#![warn(clippy::all)]
// #![warn(clippy::nursery)]
// #![warn(clippy::pedantic)]

use std::{net::SocketAddr, sync::Arc};

use tokio::{net::TcpListener, sync::Mutex};
#[allow(unused_imports)]
use tracing::{debug, error, info, trace, warn};

mod authentication;
mod handle_connection;
mod types;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    setup()?;

    let state = Arc::new(Mutex::new(handle_connection::Shared::new()));

    let addr: SocketAddr = std::env::var("ADDRESS")
        .expect("Environment variable `ADDRESS` must be set.")
        .parse()?;
    let listener = TcpListener::bind(&addr).await?;

    info!("Server is running on {}", addr);

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let db_pool = sqlx::PgPool::connect(&db_url).await?;
    let server = types::Server::new(db_pool);

    loop {
        let (stream, addr) = listener.accept().await?;
        let server = server.clone();
        let state = Arc::clone(&state);

        tokio::spawn(async move {
            info!("Accepted connection from: {}", addr);
            if let Err(e) = handle_connection::process(server, state, stream, addr).await {
                warn!("error while processing {}; error = {:?}", addr, e);
            }
            info!("Connection closed: {}", &addr);
        });
    }
}

fn setup() -> color_eyre::Result<()> {
    dotenvy::dotenv().expect(".env file not found");
    color_eyre::install()?;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    Ok(())
}
