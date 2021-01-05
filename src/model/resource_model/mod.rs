pub use {
    crate::model::*
};

// use resource_models module and it's dependencies
#[macro_export]
macro_rules! resource_models {
    ( $( $model:ident )* ) => {
        pub use roolz::model::resource_model::*;
        $( pub mod $model ;)*
    }
}
