pub use build_routes::build_routes;

#[macro_export]
macro_rules! routes {
    ( $( $input:tt )* ) => {
        use crate::controllers::*;
        roolz::route::build_routes!{ $( $input )* }
    }
}