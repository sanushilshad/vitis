use utoipa::OpenApi;
use utoipauto::utoipauto;
#[utoipauto]
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Vitis API", description = "Vitis API Endpoints")
    ),
    info(
        title = "Vitis API",
        description = "Vitis API Endpoints",
        version = "1.0.0",
        license(name = "MIT", url = "https://opensource.org/licenses/MIT")
    ),
)]
pub struct ApiDoc {}
