use actix_web::{ HttpResponse, HttpRequest, Responder };

pub use serde_json::{ json };
pub use serde::{ Serialize };
pub use actix_web::http::StatusCode;

pub use super::error::RoolzError;

#[derive(Serialize)]
pub struct JsonResponse {
    pub body: serde_json::Value,

    #[serde(skip_serializing)]
    pub status_code: StatusCode
}

impl JsonResponse {
    pub fn new(body: serde_json::Value) -> JsonResponse {
        JsonResponse { body, status_code: StatusCode::OK }
    }
}

impl Responder for JsonResponse {
    type Error = actix_web::Error;
    type Future = Result<HttpResponse, Self::Error>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        info!("{:?}\n\n", self.body);
        Ok(
            HttpResponse::build(self.status_code).
                content_type("application/json").

                body(self.body)
        )
    }
}