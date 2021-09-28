#![deny(clippy::all)]
use actix_web::{dev::ServiceRequest, web::HttpResponse, Error};
use actix_web_httpauth::extractors::bearer::BearerAuth;

use crate::AUTH_TOKEN;

#[allow(dead_code)]
pub async fn validator(
	req: ServiceRequest,
	credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
	if credentials.token() == *AUTH_TOKEN {
		Ok(req)
	} else {
		Err(HttpResponse::Unauthorized().body("Unauthorized").into())
	}
}
