//! File holding the GenericError type
//!
//! Author: [Boris](mailto:boris@humanenginuity.com)
//! Version: 1.0
//!
//! ## Release notes
//! - v1.0 : creation

// =======================================================================
// LIBRARY IMPORTS
// =======================================================================
use std::boxed::Box;
use std::error::Error;
use std::fmt;
use std::io::Error as IOError;

use hyper::error::Error as HyperError;
use rocket::Error as RocketError;
use serde_json::Error as SerdeError;

// =======================================================================
// STRUCT DEFINITION
// =======================================================================
#[derive(Debug)]
pub enum GenericError {
    Hyper(HyperError),
    Io(IOError),
    Rocket(RocketError),
    Serde(SerdeError),
    Compound((String, Box<GenericError>)),
    Basic(String),
}

// =======================================================================
// STRUCT IMPLEMENTATION
// =======================================================================
impl Error for GenericError {
    fn description(&self) -> &str {
        match *self {
            GenericError::Hyper(ref err) => err.description(),
            GenericError::Io(ref err) => err.description(),
            GenericError::Serde(ref err) => err.description(),
            GenericError::Rocket(_) => "Rocket Error - not implementing Error yet",
            GenericError::Compound((ref description, ref err)) => err.description(), // TODO: append self.0
                // err.description(), //format!("{} caused by {}", description, err.description()).as_ref(), // temp value doesn't live long enough
            GenericError::Basic(ref err) => err.as_ref(),
        }
    }

    fn cause(&self) -> Option<&Error> {
        match *self {
            GenericError::Hyper(ref err) => err.cause(),
            GenericError::Io(ref err) => err.cause(),
            GenericError::Rocket(_) => None, // Rocket Error doesn't implement Error trait yet
            GenericError::Serde(ref err) => err.cause(),
            GenericError::Compound((_,ref err)) => Some(err),
            GenericError::Basic(_) => None,
        }
    }
}

impl fmt::Display for GenericError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GenericError::Hyper(ref err) => fmt::Display::fmt(err, f),
            GenericError::Io(ref err) => fmt::Display::fmt(err, f),
            GenericError::Serde(ref err) => fmt::Display::fmt(err, f),
            // GenericError::Rocket(ref err) => fmt::Display::fmt(err, f),
            _ => f.write_str(self.description()),
        }
    }
}