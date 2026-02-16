pub const                         VERSION: &str = env!("CARGO_PKG_VERSION");
pub const                    SERVICE_NAME: &str = env!("CARGO_PKG_NAME");
pub const                 HEALTH_ENDPOINT: &str = "/health";
pub const          CHECK_UPDATES_ENDPOINT: &str = "/api/v1/update/check";
pub const             GET_LATEST_ENDPOINT: &str = "/api/v1/update/latest";
pub const             GET_BUNDLE_ENDPOINT: &str = "/api/v1/bundle";
pub const          CREATE_BUNDLE_ENDPOINT: &str = "/api/v1/bundle";
pub const            LIST_BUNDLE_ENDPOINT: &str = "/api/v1/bundle/list";
pub const        REGISTER_DEVICE_ENDPOINT: &str = "/api/v1/devices";
pub const UPDATE_DEVICE_SETTINGS_ENDPOINT: &str = "/api/v1/devices/settings";
pub const                 MAX_CONNECTIONS:  u32 = 5;
pub const             DEFAULT_SERVER_PORT:  u16 = 3000;
pub const           DEFAULT_DATABASE_HOST: &str = "::"; // IPv6 localhost
pub const         DEFAULT_FILE_SERVER_URL: &str = "http://45.76.29.165/files/snapshots";

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