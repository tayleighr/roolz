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
pub type RecordsResult<T> = std::result::Result<Vec<T>, self::Error>;

// defining and creating error types for model

include_error_types! {
    NotFound,
    Conflict,
    UnprocessableEntity
}

roolz_error! {
    pub enum Error {
        DBConflict(diesel::result::Error, StatusCode::CONFLICT),
        DBConnection(diesel::result::ConnectionError, StatusCode::SERVICE_UNAVAILABLE),
        NotFound(NotFound, StatusCode::NOT_FOUND),
        Conflict(Conflict, StatusCode::CONFLICT),
        UnprocessableEntity(UnprocessableEntity, StatusCode::UNPROCESSABLE_ENTITY)
    }
}

pub fn not_found(message: &'static str) -> Error {
    Error::from(self::NotFound{ message: message, status_code: Some(StatusCode::NOT_FOUND) })
}

pub fn conflict(message: &'static str) -> Error {
    Error::from(self::Conflict{message: message, status_code: Some(StatusCode::CONFLICT)})
}

pub fn unprocessable_entity(message: &'static str) -> Error {
    Error::from(self::UnprocessableEntity{message: message, status_code: Some(StatusCode::UNPROCESSABLE_ENTITY)})

}