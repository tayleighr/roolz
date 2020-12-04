// extern modules
extern crate diesel;
#[macro_use] extern crate log;

//roolz modules
#[macro_use] pub mod model;
#[macro_use] pub mod error;
pub mod db;
pub mod view;
pub mod controller;
pub mod server;