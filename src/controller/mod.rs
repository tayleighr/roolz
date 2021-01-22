pub use serde::{ Serialize, Deserialize };
pub use actix_web::{ web, Responder};
pub use super::error::{AppError, AppResult, helpers::*};


#[macro_export]
macro_rules! controllers {
    ( $( $controller:ident )* ) => {
        pub use roolz::controller::*;
        pub use crate::models;
        pub use crate::views;
        $( pub mod $controller;)*
    }
}