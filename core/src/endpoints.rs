//! Endpoint path templates, shared by the native router and the Worker router.
//!
//! Both axum and `worker::Router` use the same `:param` placeholder syntax, so a
//! single set of constants keeps the two routers in lockstep.

pub const HEALTH_ENDPOINT: &str = "/health";

// NIST API endpoints with :document and :revision parameters
pub const NIST_DOCUMENTS_ENDPOINT: &str = "/v1/nist/documents";
pub const NIST_SUMMARY_ENDPOINT: &str = "/v1/nist/:document/:revision/summary";
pub const NIST_FAMILIES_ENDPOINT: &str = "/v1/nist/:document/:revision/families";
pub const NIST_FAMILY_ENDPOINT: &str = "/v1/nist/:document/:revision/families/:id";
pub const NIST_ELEMENTS_ENDPOINT: &str = "/v1/nist/:document/:revision/elements";
pub const NIST_ELEMENT_ENDPOINT: &str = "/v1/nist/:document/:revision/elements/:id";
pub const NIST_REQUIREMENTS_ENDPOINT: &str = "/v1/nist/:document/:revision/requirements";
pub const NIST_SECURITY_REQS_ENDPOINT: &str = "/v1/nist/:document/:revision/security-requirements";
pub const NIST_RELATIONSHIPS_ENDPOINT: &str = "/v1/nist/:document/:revision/relationships";
pub const NIST_ELEMENT_RELATIONS_ENDPOINT: &str =
    "/v1/nist/:document/:revision/elements/:id/relationships";

// POA&M validation endpoints
pub const POAM_VALIDATE_REQ_ENDPOINT: &str =
    "/v1/nist/:document/:revision/poam/validate/:requirement_id";
pub const POAM_VALIDATE_BATCH_ENDPOINT: &str = "/v1/nist/:document/:revision/poam/validate";
pub const POAM_NON_ELIGIBLE_REQS_ENDPOINT: &str = "/v1/nist/:document/:revision/poam/non-eligible";

// FAR API endpoints with :document and :revision parameters
pub const FAR_SUMMARY_ENDPOINT: &str = "/v1/far/:document/:revision/summary";
pub const FAR_FAMILIES_ENDPOINT: &str = "/v1/far/:document/:revision/families";
pub const FAR_FAMILY_ENDPOINT: &str = "/v1/far/:document/:revision/families/:id";
pub const FAR_ELEMENTS_ENDPOINT: &str = "/v1/far/:document/:revision/elements";
pub const FAR_ELEMENT_ENDPOINT: &str = "/v1/far/:document/:revision/elements/:id";
pub const FAR_REQUIREMENTS_ENDPOINT: &str = "/v1/far/:document/:revision/requirements";
pub const FAR_RELATIONSHIPS_ENDPOINT: &str = "/v1/far/:document/:revision/relationships";
pub const FAR_ELEMENT_RELATIONS_ENDPOINT: &str =
    "/v1/far/:document/:revision/elements/:id/relationships";
