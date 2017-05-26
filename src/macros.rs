//! ## Macros
//!
//! This module defintes the following macros
//!
//! - hyper_request! : pseudo function `fn hyper_request(hyper::method::Method, url: hyper::client::IntoUrl, [headers: hyper::header::Headers], [body: Into<hyper::body::Body<'a>>]) -> Result<amiwo::contrib::rocket::ResponseJSON, GenericError>
//! - amiwo_macro : pseudo functions
//!      `fn amiwo_macro(description: ToString, cause: GenericError) -> Result<_, amiwo::GenericError::Compound>`
//!      `fn amiwo_macro(error) -> Result<_, amiwo::GenericError::Basic>`

// =======================================================================
// MACRO DEFINITIONS
// =======================================================================
macro_rules! amiwo_error {
    ($description:expr, $cause:expr) => {
        Err(GenericError::new_compound($description, $cause))
    };
    ($error:expr) => {
        Err(GenericError::Basic($error))
    };
}

// =======================================================================
// UNIT TESTS
// =======================================================================
#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use std::error::Error;
    use error::GenericError;

    #[test]
    fn macros_test_compound() {
        let err : Result<(), _> = amiwo_error!("test description", GenericError::Basic("Test error".to_string()));
        let err = err.unwrap_err();
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