pub use {
    build_table_model::table_model,
    serde::{ Serialize, Deserialize },
    crate::diesel::{ RunQueryDsl, prelude::*, dsl::* },
    crate::db::db,
    crate::error::*
};

// use table_models module and it's dependencies
#[macro_export]
macro_rules! table_models {
    ( $( $model:ident )* ) => {
        pub use roolz::table_model::*;
        $( pub mod $model ;)*
    }
}

// defining and creating error types for model


register_errors!{
    { kind: diesel::result::Error, code: StatusCode::CONFLICT }
    { kind: diesel::result::ConnectionError, code: StatusCode::SERVICE_UNAVAILABLE }
}