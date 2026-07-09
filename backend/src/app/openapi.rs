use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa::{Modify, OpenApi};

use crate::modules::auth::controller::auth_controller;
use crate::modules::common::dto::{DashboardData, LoginRequest, LoginResponse};
use crate::modules::dashboard::controller::dashboard_controller;
use crate::modules::site::controller::site_controller;
use crate::modules::site::entity::site::Site;

#[derive(OpenApi)]
#[openapi(
    paths(
        auth_controller::login,
        dashboard_controller::get_dashboard,
        site_controller::list_sites,
    ),
    components(schemas(
        LoginRequest,
        LoginResponse,
        DashboardData,
        Site,
    )),
    modifiers(&SecurityAddon),
    tags(
        (name = "auth", description = "认证"),
        (name = "dashboard", description = "仪表盘"),
        (name = "site", description = "站点"),
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_default();
        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}