use std::env;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, middleware};
use actix_web::HttpServer;
use actix_web::middleware::Logger;
use actix_web::web;
use dotenv::dotenv;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::safe::{SafeInfo, SafeResponse};
use crate::safe_config::SafeConfig;
use crate::safe_handlers::*;
use crate::safe_service::SafeService;
use crate::safe_use_case::SafeUseCase;

pub(crate) mod safe_service;
pub(crate) mod safe_handlers;
pub(crate) mod safe;
pub(crate) mod safe_use_case;
pub(crate) mod safe_config;
pub(crate) mod ethers_ext;

#[derive(OpenApi)]
#[
openapi(paths(calculate_address, deploy_contract, exec_transaction),
components(schemas(SafeInfo, SafeCall, SafeResponse, SafeErr)),
tags(
(name = "safe::api", description = "Safe management endpoints.")
))
]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "full");

    env_logger::init_from_env(env_logger::Env::new());
    let safe_use_case = SafeUseCase::new(
        Arc::new(SafeService::new(SafeConfig::new()).await)
    );

    let address = env::var("ADDRESS")
        .expect("ADDRESS must be defined");

    let port = env::var("PORT")
        .map(|port| port.parse::<u16>())
        .unwrap()
        .expect("PORT must be defined");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .supports_credentials()
                    .max_age(3600),
            )
            .app_data(web::Data::new(safe_use_case.clone()))
            .service(
                SwaggerUi::new("/swagger/{_:.*}")
                    .url("/api-doc/openapi.json", ApiDoc::openapi()),
            )
            .service(calculate_address)
            .service(deploy_contract)
            .service(exec_transaction)
    })
        .bind((address, port))?
        .run()
        .await
}