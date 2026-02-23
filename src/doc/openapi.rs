//! OpenAPI specification and JSON endpoint.

use axum::{
    response::Response,
    http::{StatusCode, header}
};
use utoipa::OpenApi;
use serde_json;

use crate::constant::OPENAPI_CACHE_DURATION;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::handler::health::health_check,
        crate::cmmc::handler::nist::get_documents,
        crate::cmmc::handler::nist::get_summary,
        crate::cmmc::handler::nist::get_families,
        crate::cmmc::handler::nist::get_family,
        crate::cmmc::handler::nist::get_elements,
        crate::cmmc::handler::nist::get_element,
        crate::cmmc::handler::nist::get_requirements,
        crate::cmmc::handler::nist::get_security_requirements,
        crate::cmmc::handler::nist::get_relationships,
        crate::cmmc::handler::nist::get_element_relationships,
    ),
    components(
        schemas(
            crate::cmmc::response::DataSummary,
            crate::cmmc::response::Family,
            crate::cmmc::response::Requirement,
            crate::cmmc::response::SecurityRequirement,
            crate::cmmc::response::PaginatedResponse<crate::cmmc::model::Element>,
            crate::cmmc::model::Element,
            crate::cmmc::model::ElementType,
            crate::cmmc::model::Document,
            crate::cmmc::model::Relationship,
            crate::cmmc::model::NistDocument,
            crate::cmmc::model::NistRevision,
            crate::cmmc::model::NistDocumentKey,
            crate::cmmc::handler::nist::DocumentInfo,
        )
    ),
    tags(
        (name = "Health", description = "Health check endpoint"),
        (name = "NIST",   description = "NIST SP 800-171 & 800-172 API"),
    ),
    info(
        title = "Tolerance API",
        version = env!("CARGO_PKG_VERSION"),
        description = r#"REST API for NIST SP 800-171 and SP 800-172 security requirements.

## Endpoints

The `/v1/nist/:document/:revision/*` endpoints support all document revisions:
- SP 800-171 Rev 1, Rev 2, Rev 3
- SP 800-172 v1.0

Examples:
- `/v1/nist/sp800-171/r3/families` — SP 800-171 Rev 3
- `/v1/nist/sp800-171/r2/families` — SP 800-171 Rev 2
- `/v1/nist/sp800-172/v1/families` — SP 800-172 v1.0

Use `/v1/nist/documents` to discover all loaded documents and revisions.

## Content Negotiation

Send `Accept: text/toon` for Token-Oriented Object Notation — a compact,
LLM-friendly format with 30-40% fewer tokens than JSON.

```bash
curl -H "Accept: text/toon" http://localhost:3000/v1/nist/sp800-171/r3/families
```

See: https://github.com/toon-format/toon
"#,
        license(name = "")
    )
)]
pub struct ApiDoc;

pub async fn openapi_json() -> Response<axum::body::Body> {
    let openapi = ApiDoc::openapi();
    let json = serde_json::to_string_pretty(&openapi)
        .expect("Failed to serialize OpenAPI specification");

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CACHE_CONTROL, format!("public, max-age={}", OPENAPI_CACHE_DURATION))
        .body(axum::body::Body::from(json))
        .expect("Failed to build OpenAPI JSON response")
}
