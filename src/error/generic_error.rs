//! File holding the GenericError type
//!
//! Author: [Boris](mailto:boris@humanenginuity.com)
//! Version: 1.1
//!
//! ## Release notes
//! - v1.1 : added From implementation (as per book guideline to use with the `try!` macro)
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
impl GenericError {
    pub fn new_compound<T: ToString>(desc: T, err: GenericError) -> GenericError {
        let mut description = desc.to_string();
        description.push_str(" caused by ");
        description.push_str(err.description());
        GenericError::Compound((description, Box::new(err)))
    }
}

impl Error for GenericError {
    fn description(&self) -> &str {
        match *self {
            GenericError::Hyper(ref err) => err.description(),
            GenericError::Io(ref err) => err.description(),
            GenericError::Serde(ref err) => err.description(),
            GenericError::Rocket(_) => "Rocket Error - not implementing Error yet",
            GenericError::Compound((ref description, _)) => description,
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

// Implement `From` as per book guideline -> https://doc.rust-lang.org/book/error-handling.html#the-from-trait
impl From<HyperError> for GenericError {
    fn from(err: HyperError) -> GenericError {
        GenericError::Hyper(err)
    }
}

impl From<IOError> for GenericError {
    fn from(err: IOError) -> GenericError {
        GenericError::Io(err)
    }
}

impl From<RocketError> for GenericError {
    fn from(err: RocketError) -> GenericError {
        GenericError::Rocket(err)
    }
}

impl From<SerdeError> for GenericError {
    fn from(err: SerdeError) -> GenericError {
        GenericError::Serde(err)
    }
}

impl From<String> for GenericError {
    fn from(err: String) -> GenericError {
        GenericError::Basic(err)
    }
}

// =======================================================================
// UNIT TESTS
// =======================================================================
#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use std::error::Error;
    use super::GenericError;

    #[test]
    fn GenericError_test_compound() {
        let err = GenericError::new_compound("test description", GenericError::Basic("Test error".to_string()));
        assert_eq!(err.description(), "test description caused by Test error");

        match err.cause() {
            Some(err) => {
                assert_eq!(err.description(), "Test error");
                assert!(err.cause().is_none());
            },
            _ => panic!("invalid cause"),
        }
    }
}