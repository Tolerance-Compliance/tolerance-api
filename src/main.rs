//! Tolerance API — entry point

use std::error::Error;
use std::net::SocketAddr;

use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tolerance_api::cmmc::{CmmcState, NistData, NistDocument, FarDocument, DocumentKey, DocumentRevision};
use tolerance_api::routing::app;

const DEFAULT_HOST: &str = "::";
const DEFAULT_PORT: u16  = 3000;

/// Describes one document to load at startup.
struct DocumentSpec {
    key:          DocumentKey,
    /// Environment variable that overrides the path.
    env_var:      &'static str,
    /// Built-in default path. `None` means the document is only loaded
    /// when the env var is explicitly set.
    default_path: Option<&'static str>,
}

/// All supported documents in load order.
fn document_specs() -> Vec<DocumentSpec> {
    vec![
        // NIST documents
        DocumentSpec {
            key:          DocumentKey::nist(NistDocument::Sp800171, DocumentRevision::Rev3),
            env_var:      "NIST_SP800_171_R3_PATH",
            default_path: Some("data/cprt-sp_800_171_3_0_0-20260215-171034.json"),
        },
        DocumentSpec {
            key:          DocumentKey::nist(NistDocument::Sp800171, DocumentRevision::Rev2),
            env_var:      "NIST_SP800_171_R2_PATH",
            default_path: Some("data/cprt-sp_800_171_2_0_0.json"),
        },
        DocumentSpec {
            key:          DocumentKey::nist(NistDocument::Sp800171, DocumentRevision::Rev1),
            env_var:      "NIST_SP800_171_R1_PATH",
            default_path: Some("data/cprt-sp_800_171_1_0_0.json"),
        },
        DocumentSpec {
            key:          DocumentKey::nist(NistDocument::Sp800172, DocumentRevision::V1),
            env_var:      "NIST_SP800_172_V1_PATH",
            default_path: Some("data/cprt-sp_800_172_1_0_0.json"),
        },
        DocumentSpec {
            key:          DocumentKey::nist(NistDocument::Sp800171A, DocumentRevision::V1),
            env_var:      "NIST_SP800_171A_V1_PATH",
            default_path: Some("data/cprt-sp_800_171a_1_0_0.json"),
        },
        DocumentSpec {
            key:          DocumentKey::nist(NistDocument::Sp800171A, DocumentRevision::Rev3),
            env_var:      "NIST_SP800_171A_R3_PATH",
            default_path: Some("data/cprt-sp_800_171_a_3_0_0.json"),
        },
        DocumentSpec {
            key:          DocumentKey::nist(NistDocument::Sp800172A, DocumentRevision::V1),
            env_var:      "NIST_SP800_172A_V1_PATH",
            default_path: Some("data/cprt-sp_800_172a_1_0_0.json"),
        },
        // FAR documents
        DocumentSpec {
            key:          DocumentKey::far(FarDocument::Far52_204_21, DocumentRevision::V2),
            env_var:      "FAR_52_204_21_PATH",
            default_path: Some("data/cprt-far_52_204_21-20260305.json"),
        },
    ]
}

fn try_load(spec: &DocumentSpec) -> Option<(DocumentKey, NistData)> {
    let path = match std::env::var(spec.env_var).ok().or_else(|| spec.default_path.map(str::to_string)) {
        Some(p) => p,
        None    => return None, // optional document, env var not set
    };

    info!("Loading {} from: {}", spec.key.display_name(), path);
    match CmmcState::load_json(&path) {
        Ok(data) => {
            info!("{} loaded successfully", spec.key.display_name());
            Some((spec.key, data))
        }
        Err(e) => {
            tracing::warn!("Failed to load {}: {}", spec.key.display_name(), e);
            None
        }
    }
}

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

    info!("Starting {} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

    let datasets: Vec<_> = document_specs()
        .iter()
        .filter_map(try_load)
        .collect();

    if datasets.is_empty() {
        return Err("No NIST documents loaded. Check file paths.".into());
    }

    let state = CmmcState::new(datasets);
    info!("{} document(s) indexed", state.available_documents().len());

    let host = std::env::var("HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string());
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(DEFAULT_PORT);

    let addr     = format!("{}:{}", host, port);
    let listener = TcpListener::bind(&addr).await?;

    info!("Listening on    http://{}", addr);
    info!("Swagger UI at   http://{}/", addr);
    info!("Documents at    http://{}/v1/nist/documents", addr);

    axum::serve(listener, app(state).into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}
