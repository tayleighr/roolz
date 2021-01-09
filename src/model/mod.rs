pub use {
    serde::{ Serialize, Deserialize },
    crate::error::*
};

pub mod resource_model;
pub mod table_model;

#[macro_export]
macro_rules! model_modules {
    ( $( $model:ident )* ) => {
        $( pub mod $model ;)*
    }
}
