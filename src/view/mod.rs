use actix_web::{ HttpResponse, HttpRequest, Responder };

pub use serde_json::{ json };
pub use serde::{ Serialize };
pub use actix_web::http::StatusCode;

pub use crate::error::AppError;

pub use crate::views;

// use views module and it's dependencies
#[macro_export]
macro_rules! views {
    ( $( $view:ident )* ) => {
        pub use roolz::view::*;
        pub use crate::table_models;
        $( pub mod $view ;)*
    }
}

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

pub fn success(message: &str) -> JsonResponse {
    JsonResponse {
        body: json!(
			{
				"status": "success",
				"message": message.to_string()
			}
		),
        status_code: StatusCode::OK
    }
}

pub fn none() -> JsonResponse {
    JsonResponse{
        body: json!({}),
        status_code: StatusCode::OK
    }
}

pub fn error(e: AppError) -> JsonResponse {

    JsonResponse {
        body: json!(
			{
				"status": "error",
				"message": format!("{}", e)
			}
		),
        status_code: e.code
    }
}