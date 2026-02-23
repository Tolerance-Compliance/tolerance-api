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
        // New NIST API endpoints
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
        // Legacy CMMC API endpoints
        crate::cmmc::handler::summary::get_summary,
        crate::cmmc::handler::families::get_families,
        crate::cmmc::handler::families::get_family,
        crate::cmmc::handler::elements::get_elements,
        crate::cmmc::handler::elements::get_element,
        crate::cmmc::handler::elements::get_requirements,
        crate::cmmc::handler::elements::get_security_requirements,
        crate::cmmc::handler::relationships::get_relationships,
        crate::cmmc::handler::relationships::get_element_relationships,
    ),
    components(
        schemas(
            // Response types
            crate::cmmc::response::DataSummary,
            crate::cmmc::response::Family,
            crate::cmmc::response::Requirement,
            crate::cmmc::response::SecurityRequirement,
            crate::cmmc::response::PaginatedResponse<crate::cmmc::model::Element>,
            // Model types
            crate::cmmc::model::Element,
            crate::cmmc::model::ElementType,
            crate::cmmc::model::Document,
            crate::cmmc::model::Relationship,
            crate::cmmc::model::NistDocument,
            crate::cmmc::model::NistRevision,
            crate::cmmc::model::NistDocumentKey,
            // Handler-specific types
            crate::cmmc::handler::nist::DocumentInfo,
            // Legacy types
            crate::cmmc::CmmcLevel,
        )
    ),
    tags(
        (name = "Health", description = "Health check endpoint"),
        (name = "NIST", description = "NIST SP 800-171 & 800-172 API - Supports multiple document revisions (Recommended)"),
        (name = "CMMC (Legacy)", description = "Legacy CMMC level-based API - Maintained for backward compatibility")
    ),
    info(
        title = "Tolerance API",
        version = env!("CARGO_PKG_VERSION"),
        description = r#"REST API for NIST SP 800-171 and SP 800-172 security requirements.

## API Versions

### New NIST API (Recommended)
The `/v1/nist/:document/:revision/*` endpoints support multiple document revisions:
- SP 800-171 Rev 1, Rev 2, Rev 3
- SP 800-172 v1.0

Examples:
- `/v1/nist/sp800-171/r3/families` - SP 800-171 Rev 3
- `/v1/nist/sp800-171/r2/families` - SP 800-171 Rev 2
- `/v1/nist/sp800-172/v1/families` - SP 800-172 v1.0

### Legacy CMMC API
The `/api/v1/cmmc/:level/*` endpoints are maintained for backward compatibility:
- `l2` = SP 800-171 Rev 3
- `l3` = SP 800-172 v1.0

## Content Negotiation

This API supports content negotiation via the `Accept` header:

### JSON (Default)
Standard JSON response format. Use `Accept: application/json` or omit the header.

### TOON (LLM-Optimized)
Token-Oriented Object Notation is a compact format optimized for LLM consumption.
Request with `Accept: text/toon` header.

Benefits:
- **30-40% token reduction** compared to JSON
- Tabular arrays with explicit lengths and field headers
- Indentation-based objects for clarity
- Smart quoting (only when necessary)

**When to use TOON:**
- Passing API responses to LLMs for compliance guidance
- Reducing token costs in LLM applications
- Improving LLM parsing accuracy for structured data

**Example:**
```bash
curl -H "Accept: text/toon" http://localhost:3000/v1/nist/sp800-171/r3/families
```

**TOON Format Example:**
```
[2]{identifier,title}:
  AC,"Access Control"
  AU,"Audit and Accountability"
```

See: https://github.com/toon-format/toon

## Discovery
Use `/v1/nist/documents` to discover available documents and revisions.
"#,
        license(name = "")
    )
)]
pub struct ApiDoc;

/// Serves the OpenAPI JSON specification.
///
/// Returns a JSON response containing the complete OpenAPI specification
/// for the Tolerance API, suitable for consumption by API documentation
/// tools and client generators.
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
