pub use actix_web::http::StatusCode as StatusCode;
pub use std::fmt;
pub use register_errors::register_errors as register_errors;

#[derive(Debug)]
pub struct AppError {
    pub code: StatusCode,
    pub kind: Option<String>,
    pub source_kind: Option<String>,
    pub message: String
}

//trait Status<T> {
//    // Define a method on the caller type which takes an
//    // additional single parameter `T` and does nothing with it.
//    fn code(self, _: T);
//}

impl AppError {
    pub fn new(code: StatusCode, message: String, kind: Option<String>) -> Self {
        AppError {
            code,
            kind,
            message,
            source_kind: None
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "code: {:?} -- kind: {:?} -- message: {:?}",
            &self.code.to_string(),
            &self.kind.as_ref().unwrap_or(&String::from("")),
            &self.message
        )
    }
}

// result type
pub type AppResult<T> = std::result::Result<T, AppError>;
pub type AppResults<T> = std::result::Result<Vec<T>, AppError>;

pub fn not_found(message: &'static str) -> AppError {
    AppError {
        code: actix_web::http::StatusCode::NOT_FOUND,
        message: message.to_string(),
        kind: None,
        source_kind: None
    }
}

pub fn conflict(message: &'static str) -> AppError {
    AppError {
        code: actix_web::http::StatusCode::CONFLICT,
        message: message.to_string(),
        kind: None,
        source_kind: None
    }
}

pub fn unprocessable_entity(message: &'static str) -> AppError {
    AppError {
        code: actix_web::http::StatusCode::UNPROCESSABLE_ENTITY,
        message: message.to_string(),
        kind: None,
        source_kind: None
    }
}

pub fn service_unavailable(message: &'static str) -> AppError {
    AppError {
        code: actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
        message: message.to_string(),
        kind: None,
        source_kind: None
    }
}
pub fn internal_server_error(message: &'static str) -> AppError {
    AppError {
        code: actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        message: message.to_string(),
        kind: None,
        source_kind: None
    }
}