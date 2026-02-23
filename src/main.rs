//! Tolerance API Server
//!
//! REST API for NIST SP 800-171 and SP 800-172 security requirements.
//! Supports multiple document revisions.

use std::error::Error;
use std::net::SocketAddr;

use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tolerance_api::cmmc::{CmmcState, NistDocument, NistDocumentKey, NistRevision};
use tolerance_api::routing::app;

const DEFAULT_PORT: u16 = 3000;
const DEFAULT_HOST: &str = "::";

// Default data paths for available documents
const NIST_SP800_171_R3_PATH: &str = "data/cprt-sp_800_171_3_0_0-20260215-171034.json";
const NIST_SP800_171_R2_PATH: &str = "data/cprt-sp_800_171_2_0_0.json";
const NIST_SP800_172_V1_PATH: &str = "data/cprt-sp_800_172_1_0_0.json";

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

    // Load SP 800-171 Rev 3 (default)
    let sp171r3_key = NistDocumentKey::new(NistDocument::Sp800171, NistRevision::Rev3);
    let sp171r3_path = std::env::var("NIST_SP800_171_R3_PATH")
        .unwrap_or_else(|_| NIST_SP800_171_R3_PATH.to_string());
    info!("Loading {} from: {}", sp171r3_key.display_name(), sp171r3_path);
    match CmmcState::load_json(&sp171r3_path) {
        Ok(data) => {
            datasets.push((sp171r3_key, data));
            info!("{} loaded successfully", sp171r3_key.display_name());
        }
        Err(e) => {
            tracing::warn!("Failed to load {}: {}", sp171r3_key.display_name(), e);
        }
    }

    // Load SP 800-172 v1.0 (default)
    let sp172v1_key = NistDocumentKey::new(NistDocument::Sp800172, NistRevision::V1);
    let sp172v1_path = std::env::var("NIST_SP800_172_V1_PATH")
        .unwrap_or_else(|_| NIST_SP800_172_V1_PATH.to_string());
    info!("Loading {} from: {}", sp172v1_key.display_name(), sp172v1_path);
    match CmmcState::load_json(&sp172v1_path) {
        Ok(data) => {
            datasets.push((sp172v1_key, data));
            info!("{} loaded successfully", sp172v1_key.display_name());
        }
        Err(e) => {
            tracing::warn!("Failed to load {}: {}", sp172v1_key.display_name(), e);
        }
    }

    let sp171r2_key = NistDocumentKey::new(NistDocument::Sp800171, NistRevision::Rev2);
    let sp171r2_path = std::env::var("NIST_SP800_171_R2_PATH")
        .unwrap_or_else(|_| NIST_SP800_171_R2_PATH.to_string());
    info!("Loading {} from: {}", sp171r2_key.display_name(), sp171r2_path);
    match CmmcState::load_json(&sp171r2_path) {
        Ok(data) => {
            datasets.push((sp171r2_key, data));
            info!("{} loaded successfully", sp171r2_key.display_name());
        }
        Err(e) => {
            tracing::warn!("Failed to load {}: {}", sp171r2_key.display_name(), e);
        }
    }

    // Optional: Load SP 800-171 Rev 1 if path provided
    if let Ok(sp171r1_path) = std::env::var("NIST_SP800_171_R1_PATH") {
        let sp171r1_key = NistDocumentKey::new(NistDocument::Sp800171, NistRevision::Rev1);
        info!("Loading {} from: {}", sp171r1_key.display_name(), sp171r1_path);
        match CmmcState::load_json(&sp171r1_path) {
            Ok(data) => {
                datasets.push((sp171r1_key, data));
                info!("{} loaded successfully", sp171r1_key.display_name());
            }
            Err(e) => {
                tracing::warn!("Failed to load {}: {}", sp171r1_key.display_name(), e);
            }
        }
    }

    if datasets.is_empty() {
        return Err("No NIST documents loaded successfully. Check file paths.".into());
    }

    let state = CmmcState::new(datasets);
    info!("All NIST documents indexed ({} documents loaded)", state.available_documents().len());

    let app: axum::Router = app(state);

    let host: String = std::env::var("HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p: String| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let addr: String = format!("{}:{}", host, port);
    let listener: TcpListener = TcpListener::bind(&addr).await?;

    info!("Server listening on http://{}", addr);
    info!("API documentation available at http://{}/", addr);
    info!("Available documents at http://{}/api/v1/nist/documents", addr);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
