pub use build_model::table_model;
pub use crate::db::db;
pub use serde::{ Serialize, Deserialize };
pub use crate::diesel::{
    RunQueryDsl,
    prelude::*,
    dsl::*
};
use crate::error::*;

// use models module and it's dependencies
#[macro_export]
macro_rules! models {
    ( $( $model:ident )* ) => {
        pub use roolz::model::*;
        $( pub mod $model ;)*
    }
}

// result type
pub type RecordResult<T> = std::result::Result<T, self::Error>;

// defining and creating error types for model

include_error_types! {
    NotFound,
    Conflict
}

roolz_error! {
    pub enum Error {
        DBConflict(diesel::result::Error, StatusCode::CONFLICT),
        DBConnection(diesel::result::ConnectionError, StatusCode::SERVICE_UNAVAILABLE),
        NotFound(NotFound, StatusCode::NOT_FOUND),
        Conflict(Conflict, StatusCode::CONFLICT)
    }
}

pub fn not_found(message: &'static str) -> self::NotFound {
    let e: self::NotFound = self::NotFound{ message: message, status_code: Some(StatusCode::NOT_FOUND) };
    e
}

pub fn conflict(message: &'static str) -> self::Conflict {
    let e: self::Conflict = self::Conflict{message: message, status_code: Some(StatusCode::CONFLICT)};
    e
}