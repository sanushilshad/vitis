use crate::handlers::health_check;
use crate::middlewares::{BusinessAccountValidation, HeaderValidation, RequireAuth};
use crate::openapi::ApiDoc;
use crate::routes::business::routes::business_routes;
use crate::routes::department::routes::department_routes;
// use crate::routes::department::routes::department_routes;
use crate::routes::leave::routes::leave_routes;
use crate::routes::permission::routes::permission_routes;
use crate::routes::role::routes::role_routes;
use crate::routes::setting::routes::setting_routes;
use crate::routes::user::routes::user_routes;
use crate::routes::web_socket::routes::web_socket_routes;
use actix_web::web;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub fn routes(cfg: &mut web::ServiceConfig) {
    let openapi = ApiDoc::openapi();
    cfg.route("/", web::get().to(health_check))
        .service(web::scope("/websocket").configure(web_socket_routes))
        .service(
            web::scope("/user")
                .configure(user_routes)
                .wrap(HeaderValidation),
        )
        .service(
            web::scope("/department")
                .configure(department_routes)
                .wrap(BusinessAccountValidation)
                .wrap(HeaderValidation)
                .wrap(RequireAuth {
                    allow_deleted_user: false,
                }),
        )
        .service(
            web::scope("/business")
                .configure(business_routes)
                .wrap(RequireAuth {
                    allow_deleted_user: false,
                })
                .wrap(HeaderValidation),
        )
        .service(
            web::scope("/setting")
                .configure(setting_routes)
                .wrap(RequireAuth {
                    allow_deleted_user: false,
                })
                .wrap(HeaderValidation),
        )
        .service(
            web::scope("/leave")
                .configure(leave_routes)
                .wrap(BusinessAccountValidation)
                .wrap(RequireAuth {
                    allow_deleted_user: false,
                })
                .wrap(HeaderValidation),
        )
        .service(
            web::scope("/role")
                .configure(role_routes)
                .wrap(RequireAuth {
                    allow_deleted_user: false,
                })
                .wrap(HeaderValidation),
        )
        .service(
            web::scope("/permission")
                .configure(permission_routes)
                .wrap(RequireAuth {
                    allow_deleted_user: false,
                })
                .wrap(HeaderValidation),
        )
        .service(SwaggerUi::new("/docs/{_:.*}").url("/api-docs/openapi.json", openapi.clone()));
}
