//! # Amiwô - API Documentation
//!
//! Hello, and welcome to the core Amiwô API documentation!
//! This crate contains both various utility functions & types 
//! that I used across several applications as well as
//! contribution to other third party modules
//!
//! # Structure
//! Each module in this library is held behind a feature flag. 
//! The present feature list is below, with an asterisk next to 
//! the features that are enabled by default:
//!
//! * "rest" => Rocket extension
//! * "json" => Serde extension
//!
//! The recommend way to include features from this crate via Cargo in your
//! project is by adding a `[dependencies.amiwo]` section to your
//! `Cargo.toml` file, setting `default-features` to false, and specifying
//! features manually. For example, to use the Rocket module, you would add:
//!
//! ```toml,ignore
//! [dependencies.amiwo]
//! version = "*"
//! default-features = false
//! features = ["rest"]
//! ```
//!
//! This crate is expected to grow with time, adding new elements to outside crates

#![feature(use_extern_macros)]
#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate log;

extern crate serde;
#[macro_use] extern crate serde_json;

// Contribution to third party modules
//  - Rocket
#[cfg(feature = "amiwo_rocket")]
extern crate rocket;

// Amiwo specific modules
pub mod contrib;
pub mod types;
pub mod util;
