pub use serde::{ Serialize, Deserialize };
pub use actix_web::{ web, Responder};
use super::error::*;


#[macro_export]
macro_rules! controllers {
    ( $( $controller:ident )* ) => {
        pub use roolz::controller::*;
        pub use crate::models;
        pub use crate::views;
        $( pub mod $controller ;)*
    }
}

include_error_types! {
    UnprocessableEntity,
    Unauthorized
}

roolz_error! {
    pub enum Error {
        UnprocessableEntity(UnprocessableEntity, StatusCode::UNPROCESSABLE_ENTITY),
        Unauthorized(Unauthorized, StatusCode::UNAUTHORIZED)
    }
}

pub fn unprocessable(message: &'static str) -> self::UnprocessableEntity {
    let e: self::UnprocessableEntity = self::UnprocessableEntity{message: message, status_code: Some(StatusCode::UNPROCESSABLE_ENTITY)};
    e
}

pub fn unauthorized(message: &'static str) -> self::Unauthorized {
    let e: self::Unauthorized = self::Unauthorized{message: message, status_code: Some(StatusCode::UNAUTHORIZED)};
    e
}