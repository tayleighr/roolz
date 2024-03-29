pub use {
    serde::{ Serialize, Deserialize },
    crate::error::{AppError, AppResult, register_errors, ForApp, helpers::*}
};

pub mod resource_model;

#[cfg(feature="database")]
pub mod table_model;

#[macro_export]
macro_rules! model_modules {
    ( $( $model:ident )* ) => {
        $( pub mod $model ;)*
    }
}
