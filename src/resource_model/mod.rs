pub use {
    serde::{ Serialize, Deserialize },
    crate::error::*
};

// defining and creating error types for model

//
//register_errors!{
//    { kind: diesel::result::Error, code: StatusCode::CONFLICT }
//    { kind: diesel::result::ConnectionError, code: StatusCode::SERVICE_UNAVAILABLE }
//}

// use resource_models module and it's dependencies
#[macro_export]
macro_rules! resource_models {
    ( $( $model:ident )* ) => {
        pub use roolz::resource_model::*;
        $( pub mod $model ;)*
    }
}
