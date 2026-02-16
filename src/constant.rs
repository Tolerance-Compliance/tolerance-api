pub const                         VERSION: &str = env!("CARGO_PKG_VERSION");
pub const                    SERVICE_NAME: &str = env!("CARGO_PKG_NAME");
pub const                 HEALTH_ENDPOINT: &str = "/health";

// CMMC / NIST 800-171 endpoints
pub const          CMMC_SUMMARY_ENDPOINT: &str = "/api/v1/cmmc/summary";
pub const         CMMC_FAMILIES_ENDPOINT: &str = "/api/v1/cmmc/families";
pub const           CMMC_FAMILY_ENDPOINT: &str = "/api/v1/cmmc/families/:id";
pub const         CMMC_ELEMENTS_ENDPOINT: &str = "/api/v1/cmmc/elements";
pub const          CMMC_ELEMENT_ENDPOINT: &str = "/api/v1/cmmc/elements/:id";
pub const     CMMC_REQUIREMENTS_ENDPOINT: &str = "/api/v1/cmmc/requirements";
pub const CMMC_SECURITY_REQS_ENDPOINT: &str = "/api/v1/cmmc/security-requirements";
pub const    CMMC_RELATIONSHIPS_ENDPOINT: &str = "/api/v1/cmmc/relationships";
pub const CMMC_ELEMENT_RELATIONS_ENDPOINT: &str = "/api/v1/cmmc/elements/:id/relationships";