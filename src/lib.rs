
#[cfg(feature="database")]
extern crate diesel;

#[cfg(feature="database")]
pub extern crate build_table_model;

//roolz procdural macros
pub extern crate build_routes;
pub extern crate register_errors;

//roolz modules
#[cfg(feature="database")]
pub mod db;

pub mod model;
pub mod error;
pub mod route;
pub mod view;
pub mod controller;