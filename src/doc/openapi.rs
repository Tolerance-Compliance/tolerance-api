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
        // POA&M validation endpoints
        crate::cmmc::handler::poam::validate_poam_requirement,
        crate::cmmc::handler::poam::validate_poam_batch,
        crate::cmmc::handler::poam::get_non_eligible_requirements,
        // FAR endpoints
        crate::cmmc::handler::far::get_summary,
        crate::cmmc::handler::far::get_families,
        crate::cmmc::handler::far::get_family,
        crate::cmmc::handler::far::get_elements,
        crate::cmmc::handler::far::get_element,
        crate::cmmc::handler::far::get_requirements,
        crate::cmmc::handler::far::get_relationships,
        crate::cmmc::handler::far::get_element_relationships,
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
            crate::cmmc::model::FarDocument,
            crate::cmmc::model::DocumentSource,
            crate::cmmc::model::NistRevision,
            crate::cmmc::model::DocumentRevision,
            crate::cmmc::model::NistDocumentKey,
            crate::cmmc::model::DocumentKey,
            crate::cmmc::handler::nist::DocumentInfo,
            // Scoring and POA&M types
            crate::cmmc::scoring::RequirementScore,
            crate::cmmc::scoring::CmmcLevel,
            crate::cmmc::scoring::Priority,
            crate::cmmc::poam::PoamValidation,
            crate::cmmc::poam::PoamEligibility,
            crate::cmmc::poam::IneligibilityReason,
            crate::cmmc::poam::PoamCondition,
            crate::cmmc::handler::poam::BatchValidationRequest,
            crate::cmmc::handler::poam::BatchValidationResponse,
        )
    ),
    tags(
        (name = "Health", description = "Health check endpoint"),
        (name = "NIST",   description = "NIST SP 800-171, 800-171A, 800-172 & 800-172A API"),
        (name = "POA&M",  description = "Plan of Action & Milestones validation API"),
        (name = "FAR",    description = "Federal Acquisition Regulation (FAR) documents API"),
    ),
    info(
        title = "Tolerance API",
        version = env!("CARGO_PKG_VERSION"),
        description = r#"REST API for NIST SP 800-171, SP 800-171A, SP 800-172, SP 800-172A, and FAR 52.204-21 security requirements and assessment procedures.

## Supported Documents

### NIST Documents

The `/v1/nist/:document/:revision/*` endpoints support:

| Document     | Revision               | Description                                       |
|--------------|------------------------|---------------------------------------------------|
| `sp800-171`  | `r1`, `r2`, `r3`       | Security Requirements for CUI.                    |
| `sp800-171a` | `v1`, `r3`             | Assessing Security Requirements for CUI. (Revision 2 was skipped, use Version 1 for 171 Revision 2.)         |
| `sp800-172`  | `v1`                   | Enhanced Security Requirements for CUI.           |
| `sp800-172a` | `v1`                   | Assessing Enhanced Security Requirements for CUI. |

Use `/v1/nist/documents` to discover all loaded NIST documents at runtime.

### FAR Documents

The `/v1/far/:document/:revision/*` endpoints support:

| Document     | Revision | Description                                                    |
|--------------|----------|----------------------------------------------------------------|
| `52.204-21`  | `v2`     | Basic Safeguarding of Covered Contractor Information Systems (CMMC Level 1). |

## Assessment Guide Element Types (171A / 172A)

The assessment guides introduce additional element types beyond `family`,
`requirement`, and `security_requirement`:

| Type | Prefix | Description |
|---                 |---    |---                                         |
| `determination`    | `DS-` | Pass/fail criteria for a requirement.      |
| `examine`          | `E-`  | Documents and artifacts to review.         |
| `interview`        | `I-`  | Personnel to interview.                    |
| `test`             | `T-`  | Mechanisms and processes to exercise.      |
| `odp`              | —     | Organization-Defined Parameter definition. |
| `odp_statement`    | `OS-` | ODP placeholder text in the requirement.   |
| `odp_type`         | —     | ODP input type (Assignment / Selection).   |

Use the relationships endpoint to traverse from a `security_requirement` to
its linked `examine`, `interview`, `test`, `determination`, and `odp` elements:

```
GET /v1/nist/sp800-172a/v1/elements/3.12.1e/relationships
```

## Content Negotiation

Send `Accept: text/toon`, see [the TOON format](https://github.com/toon-format/toon).

Send `Accept: application/json` for JSON.

Examples

- `text/toon`:
```bash
curl -H "Accept: text/toon" http://localhost:3000/v1/nist/sp800-171/r3/families
```

- `application/json`:
```bash
curl -H "Accept: application/json" http://localhost:3000/v1/nist/sp800-171/r3/families
```
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
