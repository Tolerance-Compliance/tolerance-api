pub const                         VERSION: &str = env!("CARGO_PKG_VERSION");
pub const                    SERVICE_NAME: &str = env!("CARGO_PKG_NAME");
pub const                 HEALTH_ENDPOINT: &str = "/health";

// CMMC endpoints with :level parameter (l2 = SP 800-171, l3 = SP 800-172)
pub const          CMMC_SUMMARY_ENDPOINT: &str = "/api/v1/cmmc/:level/summary";
pub const         CMMC_FAMILIES_ENDPOINT: &str = "/api/v1/cmmc/:level/families";
pub const           CMMC_FAMILY_ENDPOINT: &str = "/api/v1/cmmc/:level/families/:id";
pub const         CMMC_ELEMENTS_ENDPOINT: &str = "/api/v1/cmmc/:level/elements";
pub const          CMMC_ELEMENT_ENDPOINT: &str = "/api/v1/cmmc/:level/elements/:id";
pub const     CMMC_REQUIREMENTS_ENDPOINT: &str = "/api/v1/cmmc/:level/requirements";
pub const CMMC_SECURITY_REQS_ENDPOINT: &str = "/api/v1/cmmc/:level/security-requirements";
pub const    CMMC_RELATIONSHIPS_ENDPOINT: &str = "/api/v1/cmmc/:level/relationships";
pub const CMMC_ELEMENT_RELATIONS_ENDPOINT: &str = "/api/v1/cmmc/:level/elements/:id/relationships";