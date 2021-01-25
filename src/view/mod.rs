use {
    crate::error::ErrorMeta
};

pub use {
    actix_web::{ web, HttpResponse },
    serde_json::{ json },
    serde::{ Serialize },
    actix_web::http::StatusCode,
    crate::error::{ AppError, AppResult, helpers::* }
};

// use views module and it's dependencies
#[macro_export]
macro_rules! views {
    ( $( $view:ident )* ) => {
        pub use roolz::view::*;
        pub use crate::models;
        $( pub mod $view ;)*
    }
}

pub fn from(body: serde_json::Value) -> HttpResponse {
    json_response(
        StatusCode::OK,
        body
    )
}

pub fn json_response(status: StatusCode, body: serde_json::Value) -> HttpResponse {
    HttpResponse::build(status).
        content_type("application/json").
        body(body)
}

pub fn success(message: &str) -> HttpResponse {
    json_response(
        StatusCode::OK,
        json!(
			{
				"status": "success",
				"message": message.to_string()
			}
		)
    )
}

pub fn none() -> HttpResponse {
    json_response(
        StatusCode::OK,
        json!({})
    )
}

pub fn error(e: &AppError) -> HttpResponse {
    json_response(
        e.code().unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        json!(
			{
				"status": "error",
				"message": format!("{}", e)
			}
		)
    )
}