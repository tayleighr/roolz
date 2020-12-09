// extern modules
#[macro_use] extern crate log;
extern crate diesel;

//roolz procdural macros
pub extern crate build_routes;
pub extern crate build_model;

//roolz modules
pub mod model;
pub mod error;
pub mod route;
pub mod db;
pub mod view;
pub mod controller;
pub mod server;