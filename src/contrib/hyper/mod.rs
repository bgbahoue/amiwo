//! Utility functions for `hyper` crate
//!
//! Creates a few utility function to create ResponseJSON from Hyper Response.
//! 
//! Also implements `Into<Result<ResponseJSON, GenericError>>` for `Result<hyper::client::response::Response>` and `hyper::client::response::Response` to allow simple chaining 
//!

// =======================================================================
// LIBRARY IMPORTS
// =======================================================================
use std::str::FromStr;

use hyper::client::Client;
use hyper::method::Method;
use hyper::Url;

use error::GenericError;
use types::ResponseJSON;

// =======================================================================
// PUBLIC FUNCTIONS
// =======================================================================
/// Send a simple `method` request to `url` and pre-process the response to try to build a `ResponseJSON` from it 
pub fn request(method: &str, url: &str) -> Result<ResponseJSON, GenericError> {
    let hyper_method = Method::from_str(method.to_uppercase().as_str());
    let hyper_url = Url::parse(url);

    if hyper_method.is_err() {
        return Err(GenericError::Hyper(hyper_method.unwrap_err()));
    }
    if hyper_url.is_err() {
        return Err(GenericError::Hyper(hyper_method.unwrap_err()));                
    }

    Client::new().request(hyper_method.unwrap(), hyper_url.unwrap()).send()
    .map_err(|hyper_error| GenericError::Hyper(hyper_error))
    .and_then(|response| ResponseJSON::from_reader(response))
}