pub use {
    serde::{ Serialize, Deserialize },
    actix_web::{ web, Responder },
    crate::error::{ AppError, AppResult, helpers::* }
};


#[macro_export]
macro_rules! controllers {
    ( $( $controller:ident )* ) => {
        pub use roolz::controller::*;
        pub use crate::models;
        pub use crate::views;
        $( pub mod $controller;)*
    }
}