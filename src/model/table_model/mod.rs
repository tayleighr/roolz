pub use {
    build_table_model::table_model,
    crate::diesel::{ RunQueryDsl, prelude::*, dsl::* },
    crate::db::{ db, DBError, DBResult },
    crate::model::*
};

// use table_models module and it's dependencies
#[macro_export]
macro_rules! table_models {
    ( $( $model:ident )* ) => {
        pub use roolz::model::table_model::*;
        $( pub mod $model ;)*
    }
}