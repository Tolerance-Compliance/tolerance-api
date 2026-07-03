//! Documentation and API interface modules.
//!
//! This module contains all documentation-related functionality including
//! OpenAPI specification, Swagger UI, and static assets.

mod assets;
mod openapi;
mod swagger;

pub use assets::favicon;
pub use openapi::{ApiDoc, openapi_json};
pub use swagger::swagger_ui;
