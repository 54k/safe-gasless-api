use actix_web::{get, post, put, ResponseError};
use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::web;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::safe::SafeError;
use crate::safe_use_case::SafeUseCase;

type SafeResult<R> = Result<R, SafeError>;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SafeErr {
    code: u16,
    message: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SafeCall {
    to: String,
    value: String,
    data: Vec<u8>,
    operation: u8,
    safe_tx_gas: String,
    base_gas: String,
    gas_price: String,
    gas_token: String,
    refund_receiver: String,
    signatures: Vec<u8>,
}

impl ResponseError for SafeError {
    fn status_code(&self) -> StatusCode {
        match self {
            SafeError::RpcError(_) => StatusCode::SERVICE_UNAVAILABLE,
            _ => StatusCode::BAD_REQUEST
        }
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let json = SafeErr {
            code: self.status_code().as_u16(),
            message: format!("{self}"),
        };
        match self {
            SafeError::RpcError(_) => HttpResponse::ServiceUnavailable(),
            _ => HttpResponse::BadRequest(),
        }.json(json)
    }
}

#[utoipa::path(
get,
tag = "safe::api",
path = "/v1/safe/{address}",
responses(
(status = 200, description = "safe info", body = SafeInfo),
(status = 400, description = "bad params", body = SafeErr),
(status = 503, description = "service unavailable", body = SafeErr)
),
params(
("address" = String, Path, description = "user's public address"),
)
)]
#[get("/v1/safe/{address}")]
pub(crate) async fn calculate_address(address: web::Path<String>, service: web::Data<SafeUseCase>) -> SafeResult<impl Responder> {
    let address = address.into_inner();
    let response = service.info(address.as_str()).await?;
    Ok(
        HttpResponse::Ok().json(response)
    )
}

#[utoipa::path(
post,
tag = "safe::api",
path = "/v1/safe/{address}",
responses(
(status = 201, description = "safe response", body = SafeResponse),
(status = 400, description = "bad params", body = SafeErr),
(status = 503, description = "service unavailable", body = SafeErr)
),
params(
("address" = String, Path, description = "user's public address"),
)
)]
#[post("/v1/safe/{address}")]
pub(crate) async fn deploy_contract(address: web::Path<String>, service: web::Data<SafeUseCase>) -> SafeResult<impl Responder> {
    let address = address.into_inner();
    let response = service.deploy(address.as_str()).await?;
    Ok(
        HttpResponse::Created().json(response)
    )
}

#[utoipa::path(
put,
tag = "safe::api",
path = "/v1/safe/{address}",
responses(
(status = 200, description = "safe response", body = SafeResponse),
(status = 400, description = "bad params", body = SafeErr),
(status = 503, description = "service unavailable", body = SafeErr)
),
params(
("address" = String, Path, description = "user's public address"),
),
request_body(content = SafeCall, description = "safe operation request", content_type = "application/json"),
)]
#[put("/v1/safe/{address}")]
pub(crate) async fn exec_transaction(address: web::Path<String>,
                                     params: web::Json<SafeCall>,
                                     service: web::Data<SafeUseCase>) -> SafeResult<impl Responder> {
    let address = address.into_inner();
    let params = params.into_inner();
    let response = service.exec(
        address.as_str(),
        &params.to,
        &params.value,
        params.data,
        params.operation,
        &params.safe_tx_gas,
        &params.base_gas,
        &params.gas_price,
        &params.gas_token,
        &params.refund_receiver,
        params.signatures,
    ).await?;
    Ok(
        HttpResponse::Ok().json(response)
    )
}
