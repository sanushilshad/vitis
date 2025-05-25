use utoipa::OpenApi;
use utoipauto::utoipauto;
#[utoipauto]
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "ONDC USER REST API", description = "ONDC USER API Endpoints")
    ),
    info(
        title = "ONDC USER API",
        description = "ONDC USER API Endpoints",
        version = "1.0.0",
        license(name = "MIT", url = "https://opensource.org/licenses/MIT")
    ),
)]
pub struct ApiDoc {}
