//! Tolerance API Server
//!
//! REST API for NIST SP 800-171 Rev 3 security requirements.

use std::error::Error;
use std::net::SocketAddr;

use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tolerance_api::cmmc::CmmcState;
use tolerance_api::routing::app;

const DEFAULT_PORT: u16 = 3000;
const DEFAULT_HOST: &str = "::";
const NIST_DATA_PATH: &str = "cprt-sp_800_171_3_0_0-20260215-171034.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tolerance_api=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!(
        "Starting {} v{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    );

    // Load NIST 800-171 data
    let data_path: String = std::env::var("NIST_DATA_PATH").unwrap_or_else(|_| NIST_DATA_PATH.to_string());
    info!("Loading NIST 800-171 data from: {}", data_path);

    let state: CmmcState = CmmcState::from_json_file(&data_path)?;
    info!("NIST data loaded successfully");

    let app: axum::Router = app(state);

    let host: String = std::env::var("HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p: String| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let addr: String = format!("{}:{}", host, port);
    let listener: TcpListener = TcpListener::bind(&addr).await?;

    info!("Server listening on http://{}", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
