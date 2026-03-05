pub const                         VERSION: &str = env!("CARGO_PKG_VERSION");
pub const                    SERVICE_NAME: &str = env!("CARGO_PKG_NAME");
pub const                 HEALTH_ENDPOINT: &str = "/health";

// NIST API endpoints with :document and :revision parameters
pub const          NIST_DOCUMENTS_ENDPOINT: &str = "/v1/nist/documents";
pub const            NIST_SUMMARY_ENDPOINT: &str = "/v1/nist/:document/:revision/summary";
pub const           NIST_FAMILIES_ENDPOINT: &str = "/v1/nist/:document/:revision/families";
pub const             NIST_FAMILY_ENDPOINT: &str = "/v1/nist/:document/:revision/families/:id";
pub const           NIST_ELEMENTS_ENDPOINT: &str = "/v1/nist/:document/:revision/elements";
pub const            NIST_ELEMENT_ENDPOINT: &str = "/v1/nist/:document/:revision/elements/:id";
pub const       NIST_REQUIREMENTS_ENDPOINT: &str = "/v1/nist/:document/:revision/requirements";
pub const      NIST_SECURITY_REQS_ENDPOINT: &str = "/v1/nist/:document/:revision/security-requirements";
pub const      NIST_RELATIONSHIPS_ENDPOINT: &str = "/v1/nist/:document/:revision/relationships";
pub const  NIST_ELEMENT_RELATIONS_ENDPOINT: &str = "/v1/nist/:document/:revision/elements/:id/relationships";

// POA&M validation endpoints
pub const      POAM_VALIDATE_REQ_ENDPOINT: &str = "/v1/nist/:document/:revision/poam/validate/:requirement_id";
pub const    POAM_VALIDATE_BATCH_ENDPOINT: &str = "/v1/nist/:document/:revision/poam/validate";
pub const  POAM_NON_ELIGIBLE_REQS_ENDPOINT: &str = "/v1/nist/:document/:revision/poam/non-eligible";

// FAR API endpoints with :document and :revision parameters
pub const             FAR_SUMMARY_ENDPOINT: &str = "/v1/far/:document/:revision/summary";
pub const            FAR_FAMILIES_ENDPOINT: &str = "/v1/far/:document/:revision/families";
pub const              FAR_FAMILY_ENDPOINT: &str = "/v1/far/:document/:revision/families/:id";
pub const            FAR_ELEMENTS_ENDPOINT: &str = "/v1/far/:document/:revision/elements";
pub const             FAR_ELEMENT_ENDPOINT: &str = "/v1/far/:document/:revision/elements/:id";
pub const        FAR_REQUIREMENTS_ENDPOINT: &str = "/v1/far/:document/:revision/requirements";
pub const       FAR_RELATIONSHIPS_ENDPOINT: &str = "/v1/far/:document/:revision/relationships";
pub const   FAR_ELEMENT_RELATIONS_ENDPOINT: &str = "/v1/far/:document/:revision/elements/:id/relationships";


// Documentation cache durations
pub const           FAVICON_CACHE_DURATION:  u32 = 86400;   // 1 day
pub const           OPENAPI_CACHE_DURATION:  u32 = 300;     // 5 minutes
pub const        SWAGGER_UI_CACHE_DURATION:  u32 = 300;     // 5 minutes

// Swagger UI HTML
pub const                  SWAGGER_UI_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <meta name="description" content="Tolerance API - NIST SP 800-171 & 800-172 Swagger UI" />
    <title>Tolerance API | CMMC Documentation</title>
    <link rel="icon" type="image/x-icon" href="/favicon.ico">
    <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui.css" />
    <style>
        .swagger-ui .info .title {
            color: #2563eb;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        }
        .swagger-ui .info .description {
            color: #333;
        }
        .swagger-ui .scheme-container {
            background: linear-gradient(90deg, #2563eb 0%, #60a5fa 100%);
            border: none;
            box-shadow: 0 2px 4px rgba(37,99,235,0.1);
        }
        .swagger-ui .topbar {
            background: linear-gradient(90deg, #2563eb 0%, #60a5fa 100%);
            border-bottom: 1px solid #e3e3e3;
        }
        .swagger-ui .topbar .download-url-wrapper .download-url-button {
            background: #2563eb;
            border-color: #2563eb;
        }
        .swagger-ui .btn.authorize {
            background: #2563eb;
            border-color: #2563eb;
        }
        .swagger-ui .btn.execute {
            background: #2563eb;
            border-color: #2563eb;
        }
    </style>
</head>
<body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui-bundle.js" crossorigin></script>
    <script src="https://unpkg.com/swagger-ui-dist@5.9.0/swagger-ui-standalone-preset.js" crossorigin></script>
    <script>
        window.onload = () => {
            window.ui = SwaggerUIBundle({
                url: '/api-docs/openapi.json',
                dom_id: '#swagger-ui',
                presets: [
                    SwaggerUIBundle.presets.apis,
                    SwaggerUIStandalonePreset
                ],
                layout: "StandaloneLayout",
                deepLinking: true,
                showExtensions: true,
                showCommonExtensions: true,
                tryItOutEnabled: true,
                filter: true,
                requestInterceptor: (request) => {
                    return request;
                },
                responseInterceptor: (response) => {
                    return response;
                }
            });
        };
    </script>
</body>
</html>"#;