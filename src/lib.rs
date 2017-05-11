//! # Amiwô - API Documentation
//!
//! Hello, and welcome to the core Amiwô API documentation!
//!
//! ## Libraries
//!
//! - [Types](/types) - Various types commonly used.

#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate log;

extern crate rocket;

pub mod types;