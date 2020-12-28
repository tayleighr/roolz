// extern modules
#[macro_use] extern crate log;
extern crate diesel;

//roolz procdural macros
pub extern crate build_routes;
pub extern crate build_table_model;
pub extern crate register_errors;

//roolz modules
pub mod table_model;
pub mod resource_model;
pub mod error;
pub mod route;
pub mod db;
pub mod view;
pub mod controller;