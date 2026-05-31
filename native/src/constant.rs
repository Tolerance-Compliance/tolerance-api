pub const                         VERSION: &str = env!("CARGO_PKG_VERSION");
pub const                    SERVICE_NAME: &str = env!("CARGO_PKG_NAME");

// Endpoint path templates are shared with the Worker via the core crate so both
// routers stay in lockstep.
pub use tolerance_api_core::endpoints::*;


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