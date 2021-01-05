pub use actix_web::http::StatusCode as StatusCode;
pub use std::fmt;
pub use register_errors::register_errors as register_errors;

pub trait ErrorMeta: std::error::Error {
    fn code(&self) -> Option<StatusCode>;
    fn typ(&self) -> String;
    fn kind(&self) -> String;
    fn origin(&self) -> String;
    fn reason(&self) -> String;
}

pub trait ForApp<T> {
    fn for_app(e: T) -> AppError;
}

pub type AppResult<T> = std::result::Result<T, AppError>;
pub type AppResults<T> = std::result::Result<Vec<T>, AppError>;

#[derive(Debug)]
pub enum AppError {
    New(&'static str, Option<StatusCode>),
    From(Box<dyn ErrorMeta>)
}

impl AppError {
    pub fn new(message: &'static str, code: Option<StatusCode>) -> Self {
        AppError::New(message, code)
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
//        match *self {
//            AppError::New( _,_ ) => None,
//            AppError::From(ref e) => Some(e) // don't have a size known at compile time
//        }
    }
}

impl ForApp<AppError> for AppError {
    fn for_app(e: AppError) -> AppError{
        e
    }
}

impl ErrorMeta for AppError {
    fn code(&self) -> Option<StatusCode> {
        match &*self {
            AppError::New(_, c) => *c,
            AppError::From(e) => e.code()
        }
    }

    fn typ(&self) -> String {
        match &*self {
            AppError::New(_, _) => String::from("AppError"),
            AppError::From(e) => e.typ()
        }
    }

    fn kind(&self) -> String {
        match &*self {
            AppError::New(_, _) => String::from("New"),
            AppError::From(e) => e.kind()
        }
    }

    fn origin(&self) -> String {
        match &*self {
            AppError::New(_, _) => String::new(),
            AppError::From(e) => e.origin()
        }
    }

    fn reason(&self) -> String {
        match &*self {
            AppError::New(m, _) => m.to_string(),
            AppError::From(e) => {
                if let Some(source) = e.source() {
                    source.to_string()
                } else {
                    String::from("No reason provided")
                }
            }
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "type: {:} - kind: {:} - origin: {:} - reason: {:}",
            self.typ(),
            self.kind(),
            self.origin(),
            self.reason()
        )
    }
}

pub fn not_found(message: &'static str) -> AppError {
    AppError::New(message, Some(actix_web::http::StatusCode::NOT_FOUND))
}

pub fn conflict(message: &'static str) -> AppError {
    AppError::New(message, Some(actix_web::http::StatusCode::CONFLICT))
}

pub fn unprocessable_entity(message: &'static str) -> AppError {
    AppError::New(message, Some(actix_web::http::StatusCode::UNPROCESSABLE_ENTITY))
}

pub fn service_unavailable(message: &'static str) -> AppError {
    AppError::New(message, Some(actix_web::http::StatusCode::SERVICE_UNAVAILABLE))
}
pub fn internal_server_error(message: &'static str) -> AppError {
    AppError::New(message, Some(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR))
}