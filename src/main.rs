//! Tolerance API Server
//!
//! REST API for NIST SP 800-171 (CMMC L2) and SP 800-172 (CMMC L3) security requirements.

use std::error::Error;
use std::net::SocketAddr;

use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tolerance_api::cmmc::{CmmcLevel, CmmcState};
use tolerance_api::routing::app;

const DEFAULT_PORT: u16 = 3000;
const DEFAULT_HOST: &str = "::";
const NIST_L2_DATA_PATH: &str = "cprt-sp_800_171_3_0_0-20260215-171034.json";
const NIST_L3_DATA_PATH: &str = "cprt-sp_800_172_1_0_0.json";

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

    let mut datasets = Vec::new();

    let l2_path = std::env::var("NIST_L2_DATA_PATH").unwrap_or_else(|_| NIST_L2_DATA_PATH.to_string());
    info!("Loading CMMC L2 (SP 800-171) data from: {}", l2_path);
    let l2_data = CmmcState::load_json(&l2_path)?;
    datasets.push((CmmcLevel::L2, l2_data));
    info!("CMMC L2 data loaded");

    let l3_path = std::env::var("NIST_L3_DATA_PATH").unwrap_or_else(|_| NIST_L3_DATA_PATH.to_string());
    info!("Loading CMMC L3 (SP 800-172) data from: {}", l3_path);
    let l3_data = CmmcState::load_json(&l3_path)?;
    datasets.push((CmmcLevel::L3, l3_data));
    info!("CMMC L3 data loaded");

    let state = CmmcState::new(datasets);
    info!("All CMMC datasets indexed");

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
