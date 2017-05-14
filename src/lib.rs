//! # Amiwô - API Documentation
//!
//! Hello, and welcome to the core Amiwô API documentation!
//!
//! ## Libraries
//!
//! - [Types](/types) - Various types commonly used.
#![feature(use_extern_macros)]
#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate log;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;

extern crate rocket;
extern crate rocket_contrib;

pub mod types;
pub mod util;
pub mod rest;