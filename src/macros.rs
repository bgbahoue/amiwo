//! ## Macros
//!
//! This module defintes the following macros
//!
//! - hyper_request! : pseudo function `fn hyper_request(hyper::method::Method, url: hyper::client::IntoUrl, [headers: hyper::header::Headers], [body: Into<hyper::body::Body<'a>>]) -> Result<amiwo::contrib::rocket::ResponseJSON, GenericError>
//! - amiwo_macro : pseudo functions
//!      `fn amiwo_macro(description, cause) -> Result<_, amiwo::GenericError::Simple>`
//!      `fn amiwo_macro(error) -> Result<_, amiwo::GenericError::Basic>`
//!      `fn amiwo_macro(string, arg1, ..., argN) -> Result<_, amiwo::GenericError::Basic>` sugar for `amiwo_macro(format!(string, arg1, ..., argN))`
// =======================================================================
// MACRO DEFINITIONS
// =======================================================================
#[cfg(feature = "amiwo_hyper")]
macro_rules! hyper_request {
    ($method:path, $url:expr) => {
        Client::new().request($method, $url).send()
            .map_err(|hyper_error| GenericError::Hyper(hyper_error))
            .and_then(|response| ResponseJSON::from_reader(response))
    }
}

macro_rules! amiwo_error {
    ($description:expr, $cause:expr) => {
        Err(GenericError::new_compound($description, $cause))
    };
    ($error:expr) => {
        Err(GenericError::Basic($error))
    };
}