//! File holding the ResponseJSON type and associated tests
//!
//! Author: [Boris](mailto:boris@humanenginuity.com)
//! Version: 1.1
//!
//! ## Release notes
//! - v1.1 : changed `data` to Value instead of &Value
//! - v1.0 : creation

// =======================================================================
// LIBRARY IMPORTS
// =======================================================================
use std::error::Error;
use std::io::Read;
use std::string::ToString;

use hyper;

use rocket;
use rocket::{ Data, Request, Response };
use rocket::response::content;
use rocket::data::{ FromData, Outcome };
use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::response::Responder;
use serde_json;
use serde_json::Value;

use error::GenericError;
use util::ContainsKeys;

// =======================================================================
// STRUCT & TRAIT DEFINITION
// =======================================================================
/// JSON wrapper for a JSON response from a REST route
/// It wraps an optional generic type `T` that just needs to implement [serde's Deserialize](https://docs.serde.rs/serde/de/trait.Deserializer.html)
///
/// It derives Rocket's [Responder trait](https://api.rocket.rs/rocket/response/trait.Responder.html) so it can be used as such in a Rocket's route as illustrated below
///
/// ```rust,ignore
/// #[get("/")]
/// fn index() -> ResponseJSON<T> { ... }
/// ```
#[derive(Clone, Debug)]
pub struct ResponseJSON {
    pub success: bool,
    pub http_code: u16,
    pub data: Value,
    pub message: Option<String>, // required for error JSON
    pub resource: Option<String>,
    pub method: Option<String>,
}

/// Test if the underlying structure is a valid ResponseJSON
pub trait IsResponseJSON {
    fn is_valid_json(&self) -> bool;
    fn is_ok_json(&self) -> bool;
    fn is_error_json(&self) -> bool;
}

// =======================================================================
// STRUCT IMPLEMENTATION
// =======================================================================
impl ResponseJSON {
    // Create an empty OK ResponseJSON
    pub fn ok() -> ResponseJSON {
        ResponseJSON {
            success: true,
            http_code: 200,
            data: Value::Null,
            message: None,
            resource: None,
            method: None,
        }
    }

    // Create an empty OK ResponseJSON
    pub fn error() -> ResponseJSON {
        ResponseJSON {
            success: false,
            http_code: 500,
            data: Value::Null,
            message: Some("Unexpected error".to_string()),
            resource: None,
            method: None,
        }
    }

    /// Set the HTTP Code of this ResponseJSON
    pub fn http_code(mut self, code: u16) -> ResponseJSON {
        self.http_code = code;
        self
    }

    /// Set the data of this ResponseJSON
    pub fn data(mut self, data: Value) -> ResponseJSON {
        self.data = data;
        self
    }

    /// Set the error message.
    /// For Error JSON only (does nothing if `success == ok`)    
    pub fn message(mut self, string: String) -> ResponseJSON {
        if !self.success {
            self.message = Some(string);
        } else {
            warn!("::AMIWO::CONTRIB::ROCKET::RESPONSEJSON::MESSAGE::WARNING Trying to set `message` on an Ok JSON => ignored")
        }
        self
    }

    /// Set the resource that we tried to access.
    /// For Error JSON only (does nothing if `success == ok`)
    pub fn resource(mut self, string: String) -> ResponseJSON {
        if !self.success {
            self.resource = Some(string);
        } else {
            warn!("::AMIWO::CONTRIB::ROCKET::RESPONSEJSON::RESOURCE::WARNING Trying to set `resource` on an Ok JSON => ignored")
        }
        self
    }

    /// Set the method that was used (GET, POST, ...).
    /// For Error JSON only (does nothing if `success == ok`)
    pub fn method(mut self, string: String) -> ResponseJSON {
        if !self.success {
            self.method = Some(string);
        } else {
            warn!("::AMIWO::CONTRIB::ROCKET::RESPONSEJSON::METHOD::WARNING Trying to set `method` on an Ok JSON => ignored")
        }
        self
    }

    /// ResponseJSON<T> can be created from a `serde_json::Value`, consuming the original object
    /// If the input is a valid ResponseJSON it duplicates it
    /// Else it creates an Ok ResponseJSON with it's data property set to the input JSON
    pub fn from_serde_value(json: Value) -> ResponseJSON {
        if json.is_object() {
            if json.is_ok_json() {
                ResponseJSON::ok()
                    .http_code(json["http_code"].as_u64().unwrap() as u16)
                    .data(json.get("data").unwrap_or(&Value::Null).clone())
            } else if json.is_error_json() {
                let mut rjson = ResponseJSON::error()
                    .http_code(json["http_code"].as_u64().unwrap() as u16)
                    .data(json.get("data").unwrap_or(&Value::Null).clone());

                if !json["message"].is_null() { rjson = rjson.message(json["message"].as_str().unwrap().to_string()); }
                if !json["resource"].is_null() { rjson = rjson.resource(json["resource"].as_str().unwrap().to_string()); }
                if !json["method"].is_null() { rjson = rjson.method(json["method"].as_str().unwrap().to_string()); }

                rjson
            } else {
                ResponseJSON::ok()
                    .data(json.pointer("").unwrap().clone())
            }
        } else {
            ResponseJSON::ok()
                .data(json.pointer("").unwrap().clone())
        }
    }

    /// Deserialize a ResponseJSON from a string of JSON text
    pub fn from_str<'s>(s: &'s str) -> Result<ResponseJSON, GenericError> {
        serde_json::from_str(s)
            .map( |value : Value| Self::from_serde_value(value) )
            .map_err( |serde_err| GenericError::Serde(serde_err) )
    }

    /// Deserialize a ResponseJSON from an IO stream of JSON
    pub fn from_reader<R: Read>(reader: R) -> Result<ResponseJSON, GenericError> {
        serde_json::from_reader(reader)
            .map( |value : Value| Self::from_serde_value(value) )
            .map_err( |serde_err| GenericError::Serde(serde_err) )
    }
  
    /// Consumes the ResponseJSON wrapper and returns the wrapped item.
    // Note: Contrary to `serde_json::to_string()`, serialization can't fail.
    pub fn into_string(self) -> String {
        self.to_string()
    }
}

// =======================================================================
// TRAIT IMPLEMENTATION
// ======================================================================
/// Serialize the given ResponseJSON as a String
impl ToString for ResponseJSON {
    // Note: Contrary to `serde_json::to_string()`, serialization can't fail.
    fn to_string(&self) -> String {
        json!({
            "success": self.success,
            "http_code": self.http_code,
            "data": &self.data,
            "message": &self.message,
            "resource": &self.resource,
            "method": &self.method
        }).as_object_mut()
        .map_or(
            "{\"http_code\":500,\"message\":\"Invalid ResponseJSON\",\"success\":false}".to_string(),
            |map| {
                if map["data"].is_null() { map.remove("data"); };
                if map["message"].is_null() { map.remove("message"); };
                if map["resource"].is_null() { map.remove("resource"); };
                if map["method"].is_null() { map.remove("method"); };

                serde_json::to_string(map).unwrap()
            }
        )
    }
}

/// Parse a ResponseJSON from incoming POST/... form data.
/// If the content type of the request data is not
/// `application/json`, `Forward`s the request.
///
/// All relevant warnings and errors are written to the console
impl FromData for ResponseJSON {
    type Error = GenericError;

    fn from_data<'r>(request: &'r Request, data: Data) -> Outcome<Self, GenericError> {
        if !request.content_type().map_or(false, |ct| ct.is_json()) {
            error!("::AMIWO::CONTRIB::ROCKET::RESPONSEJSON::FROM_DATA::ERROR Content-Type is not JSON.");
            return rocket::Outcome::Forward(data);
        }

        let size_limit = rocket::config::active()
            .and_then(|c| c.extras.get("limits.json")) // TODO: remove placeholder when upgrading to rocket version > 0.2.6
            // .and_then(|c| c.limits.get("json") // In next version
            .and_then(|limit| limit.as_integer())
            .unwrap_or(1 << 20) as u64; // default limit is 1MB for JSON

        // ResponseJSON::from_reader(data.open().take(size_limit))
        serde_json::from_reader(data.open().take(size_limit))
            .map_err(|serde_err| { error!("::AMIWO::CONTRIB::ROCKET::RESPONSEJSON::FROM_DATA::ERROR Unable to create JSON from reader => {:?}", serde_err); GenericError::Serde(serde_err) })
            .map( |value| ResponseJSON::from_serde_value(value) )
            .into_outcome()
    }
}

/// Serializes the wrapped value into a ResponseJSON. Returns a response with Content-Type
/// JSON and a fixed-size body with the serialized value. If serialization
/// fails, an `Err` of `Status::InternalServerError` is returned.
impl<'r> Responder<'r> for ResponseJSON {
    fn respond(self) -> Result<Response<'r>, Status> {
        content::JSON(self.into_string()).respond()
    }
}

/// Sugar to convert a valid Hyper Response into a ResponseJSON
/// Since `From` can't fail it will return an error ResponseJSON when it can't parse
/// it's body into a valid ResponseJSON
///
/// ```rust
/// extern crate hyper;
/// extern crate amiwo;
///
/// hyper::client::Client::new()
///     .get("http://some/url")
///     .send()
///     .map(::std::convert::From::from)
///     .map(|json : amiwo::ResponseJSON| println!("JSON received from request = {:?}", json) );
/// ```
impl From<hyper::client::response::Response> for ResponseJSON {
    fn from(response: hyper::client::response::Response) -> Self {
        ResponseJSON::from_reader(response)
            .unwrap_or_else( |err| ResponseJSON::error().data(Value::String(format!("Error converting response into a ResponseJSON > {}", err.description()))) )
    }
}

impl IsResponseJSON for ResponseJSON {
    /// Check if the JSON described as a String is a valid ResponseJSON
    fn is_valid_json(&self) -> bool {
        true
    }
    
    /// Check if the JSON described as a String is an Error JSON
    fn is_error_json(&self) -> bool
    {
        self.success == false
    }

    /// Check if the JSON described as a String is an OK JSON
    fn is_ok_json(&self) -> bool {
        self.success == true &&
        self.method.is_none() &&
        self.message.is_none() &&
        self.resource.is_none()
    }
}

impl IsResponseJSON for serde_json::map::Map<String, Value> {
    fn is_valid_json(&self) -> bool {
        self.contains_keys(&["success", "http_code"]) 
    }

    fn is_ok_json(&self) -> bool {
        self.is_valid_json() && 
        self["success"] == Value::Bool(true) &&
        self["http_code"].is_number() &&
        self["method"].is_null() &&
        self["resource"].is_null() &&
        self["message"].is_null()
    }

    fn is_error_json(&self) -> bool {
        self.is_valid_json() && 
        self["success"] == Value::Bool(false) &&
        self["http_code"].is_number() &&
        (self.get("message").is_none() || self["message"].is_string()) &&
        (self.get("resource").is_none() || self["resource"].is_string()) &&
        (self.get("method").is_none() || self["method"].is_string())
    }
}

impl IsResponseJSON for Value {
    fn is_valid_json(&self) -> bool {
        self.contains_keys(&["success", "http_code"]) 
    }

    fn is_ok_json(&self) -> bool {
        self.is_valid_json() && 
        self["success"] == Value::Bool(true) &&
        self["http_code"].is_number() &&
        self["method"].is_null() &&
        self["resource"].is_null() &&
        self["message"].is_null()
    }

    fn is_error_json(&self) -> bool {
        self.is_valid_json() && 
        self["success"] == Value::Bool(false) &&
        self["http_code"].is_number() &&
        (self.get("message").is_none() || self["message"].is_string()) &&
        (self.get("resource").is_none() || self["resource"].is_string()) &&
        (self.get("method").is_none() || self["method"].is_string())
    }
}

impl IsResponseJSON for String {
    fn is_valid_json(&self) -> bool{
        serde_json::from_str(&self)
            .ok()
            .map_or(
                false,
                |json : Value| json.is_valid_json()
            )
    } 

    fn is_ok_json(&self) -> bool {
        serde_json::from_str(&self)
            .ok()
            .map_or(
                false,
                |json : Value| json.is_ok_json()
            )
    }

    fn is_error_json(&self) -> bool {
        serde_json::from_str(&self)
            .ok()
            .map_or(
                false,
                |json : Value| json.is_error_json()
            )
    }
}

impl IsResponseJSON for str {
    fn is_valid_json(&self) -> bool{
        serde_json::from_str(&self)
            .ok()
            .map_or(
                false,
                |json : Value| json.is_valid_json()
            )
    } 

    fn is_ok_json(&self) -> bool {
        serde_json::from_str(&self)
            .ok()
            .map_or(
                false,
                |json : Value| json.is_ok_json()
            )
    }

    fn is_error_json(&self) -> bool {
        serde_json::from_str(&self)
            .ok()
            .map_or(
                false,
                |json : Value| json.is_error_json()
            )
    }
}

impl<T: ToString> PartialEq<T> for ResponseJSON {
    fn eq(&self, other: &T) -> bool {
        self.to_string() == other.to_string()
    }
}

macro_rules! __impl_rjson_partial_eq {
    (to_string @ $other:ty) => { __impl_rjson_partial_eq!(to_string @ $other, ResponseJSON); };
    (to_string @ $other:ty, <$($args:tt),* $(,)*> ) => { __impl_rjson_partial_eq!(to_string @ $other, ResponseJSON, [$($args),*]); };
    (to_string @ $Lhs:ty, $Rhs:ty) => {
        impl PartialEq<$Rhs> for $Lhs 
            where 
                $Lhs: ToString,
                $Rhs: ToString
        {
            fn eq(&self, other: &$Rhs) -> bool {
                self.to_string() == other.to_string()
            }
        }
    };
    (to_string @ $Lhs:ty, $Rhs:ty, [$($args:tt),* $(,)*] ) => { // Note: changed from '<>' to '[]' to avoid infinite macro recursion
        impl<$($args),*> PartialEq<$Rhs> for $Lhs 
            where 
                $Lhs: ToString,
                $Rhs: ToString
        {
            fn eq(&self, other: &$Rhs) -> bool {
                self.to_string() == other.to_string()
            }
        }
    };
}

__impl_rjson_partial_eq!(to_string @ Value);
__impl_rjson_partial_eq!(to_string @ String);
__impl_rjson_partial_eq!(to_string @ &'r str, <'r>);

// =======================================================================
// UNIT TESTS
// =======================================================================
#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    #![allow(unmounted_route)]

    use super::ResponseJSON;
    use super::IsResponseJSON;

    use serde_json;
    use serde_json::Value;

    use rocket;
    use rocket::testing::MockRequest;
    use rocket::http::{ ContentType, Method, Status };

    use contrib::rocket::FormHashMap;

    #[test]
    fn ResponseJSON_test_IsResponseJSON_implem() {
        let json = r#"{
            "success": false,
            "http_code": 500,
            "resource": "some resource requested",
            "method": "GET",
            "message": "error message"
        }"#;
        assert_eq!(json.is_valid_json(), true);
        assert_eq!(json.is_ok_json(), false);
        assert_eq!(json.is_error_json(), true);

        let json = r#"{
            "success": true,
            "http_code": 200,
            "resource": "some resource requested",
            "method": "GET",
            "message": "error message"
        }"#;
        assert_eq!(json.is_valid_json(), true);
        assert_eq!(json.is_error_json(), false);
        assert_eq!(json.is_ok_json(), false);

        let json = r#"{
            "http_code": 200,
            "resource": "some resource requested",
            "method": "GET",
            "message": "error message"
        }"#;
        assert_eq!(json.is_valid_json(), false);
        assert_eq!(json.is_error_json(), false);
        assert_eq!(json.is_ok_json(), false);

        let json = r#"{
            "success": true,
            "resource": "some resource requested",
            "method": "GET",
            "message": "error message"
        }"#;
        assert_eq!(json.is_valid_json(), false);
        assert_eq!(json.is_error_json(), false);
        assert_eq!(json.is_ok_json(), false);
    }

    #[test]
    fn ResponseJSON_test_builder_ok() {
        let json : ResponseJSON = ResponseJSON::ok();
        assert_eq!(json.success, true);
        assert_eq!(json.http_code, 200);
        assert!(json.data.is_null());
        assert_eq!(json.message, None);
        assert_eq!(json.method, None);
        assert_eq!(json.resource, None);

        let json : ResponseJSON = ResponseJSON::ok()
            .http_code(201)
            .data("Some data".into())
            .method("GET".to_string())
            .resource("some path".to_string())
            .message("error message".to_string());
        assert_eq!(json.http_code, 201);
        assert_eq!(json.data.as_str(), Some("Some data"));
        assert_eq!(json.message, None);
        assert_eq!(json.method, None);
        assert_eq!(json.resource, None);
        
        assert_eq!(json.is_valid_json(), true);
        assert_eq!(json.is_ok_json(), true);
        assert_eq!(json.is_error_json(), false);
    }

    #[test]
    fn ResponseJSON_test_builder_error() {
        let json : ResponseJSON = ResponseJSON::error();
        assert_eq!(json.success, false);
        assert_eq!(json.http_code, 500);
        assert!(json.data.is_null());
        assert_eq!(json.message, Some("Unexpected error".to_string()));
        assert_eq!(json.method, None);
        assert_eq!(json.resource, None);

        let json : ResponseJSON = ResponseJSON::error()
            .http_code(401)
            .data("Some data".into())
            .method("GET".to_string())
            .resource("some path".to_string())
            .message("error message".to_string());
        assert_eq!(json.http_code, 401);
        assert_eq!(json.data.as_str(), Some("Some data"));
        assert_eq!(json.message, Some("error message".to_string()));
        assert_eq!(json.method, Some("GET".to_string()));
        assert_eq!(json.resource, Some("some path".to_string()));

        assert_eq!(json.is_valid_json(), true);
        assert_eq!(json.is_ok_json(), false);
        assert_eq!(json.is_error_json(), true);
    }

    #[test]
    fn ResponseJSON_test_from_str() {
        // Simple non ResponseJSON
        let json = ResponseJSON::from_str(r#"{
            "test1": "value1",
            "test2": "value2",
            "test3": [ 1, 2, 3 ]
        }"#).unwrap();
        assert_eq!(json.is_valid_json(), true);
        assert_eq!(json.is_ok_json(), true);
        assert_eq!(json.is_error_json(), false);
        assert_eq!(json.data["test2"], Value::String("value2".to_string()));

        // ok json without data
        let json = ResponseJSON::from_str(r#"{
            "success": true,
            "http_code": 204
        }"#).unwrap();
        assert_eq!(json.is_valid_json(), true);
        assert_eq!(json.is_ok_json(), true);
        assert_eq!(json.is_error_json(), false);
        assert_eq!(json.http_code, 204);
        assert_eq!(json.method.is_none(), true);
        assert_eq!(json.resource.is_none(), true);
        assert_eq!(json.message.is_none(), true);
        assert_eq!(json.data.is_null(), true);

        // improper ok json (yet still parsed but everything will be moved in data)
        let json = ResponseJSON::from_str(r#"{
            "success": true,
            "http_code": 201,
            "resource": "some resource requested",
            "method": "GET",
            "message": "error message"
        }"#).unwrap();
        assert_eq!(json.is_valid_json(), true);
        assert_eq!(json.is_ok_json(), true);
        assert_eq!(json.is_error_json(), false);
        assert_eq!(json.http_code, 200);
        assert_eq!(json.method.is_none(), true);
        assert_eq!(json.resource.is_none(), true);
        assert_eq!(json.message.is_none(), true);
        let val : Value = serde_json::from_str("201").unwrap();
        assert_eq!(json.data["http_code"], val);

        // ok json with data
        let json = ResponseJSON::from_str(r#"{
            "success": true,
            "http_code": 202,
            "data": {
                "test1": "value1",
                "test2": "value2",
                "test3": [ 1, 2, 3 ]
            }
        }"#).unwrap();
        assert_eq!(json.is_valid_json(), true);
        assert_eq!(json.is_ok_json(), true);
        assert_eq!(json.is_error_json(), false);
        assert_eq!(json.http_code, 202);
        assert_eq!(json.data["test2"], Value::String("value2".to_string()));

        // error json without data
        let json = ResponseJSON::from_str(r#"{
            "success": false,
            "http_code": 501,
            "resource": "some resource requested",
            "method": "GET",
            "message": "error message"
        }"#).unwrap();
        assert_eq!(json.is_valid_json(), true);
        assert_eq!(json.is_ok_json(), false);
        assert_eq!(json.is_error_json(), true);
        assert_eq!(json.http_code, 501);
        assert_eq!(json.resource.unwrap(), "some resource requested".to_string());

        // error json with data
        let json = ResponseJSON::from_str(r#"{
            "success": false,
            "http_code": 502,
            "data": {
                "test1": "value1",
                "test2": "value2",
                "test3": [ 1, 2, 3 ]
            },
            "resource": "some resource requested",
            "method": "GET",
            "message": "error message"
        }"#).unwrap();
        assert_eq!(json.data["test2"], Value::String("value2".to_string()));

        assert_eq!(json.is_valid_json(), true);
        assert_eq!(json.is_ok_json(), false);
        assert_eq!(json.is_error_json(), true);
        assert_eq!(json.http_code, 502);
        assert_eq!(json.resource.unwrap(), "some resource requested".to_string());
        assert_eq!(json.data["test1"], Value::String("value1".to_string()));
    }

    #[test]
    fn ResponseJSON_test_from_serde_json() {
        // Simple non ResponseJSON
        let json = serde_json::from_str(r#"{
            "test1": "value1",
            "test2": "value2",
            "test3": [ 1, 2, 3 ]
        }"#).unwrap();
        let rjson = ResponseJSON::from_serde_value(json);
        assert_eq!(rjson.is_valid_json(), true);
        assert_eq!(rjson.is_ok_json(), true);
        assert_eq!(rjson.is_error_json(), false);
        assert_eq!(rjson.data["test2"], Value::String("value2".to_string()));

        // ok json without data
        let json = serde_json::from_str(r#"{
            "success": true,
            "http_code": 204
        }"#).unwrap();
        let rjson = ResponseJSON::from_serde_value(json);
        assert_eq!(rjson.is_valid_json(), true);
        assert_eq!(rjson.is_ok_json(), true);
        assert_eq!(rjson.is_error_json(), false);
        assert_eq!(rjson.http_code, 204);
        assert_eq!(rjson.method.is_none(), true);
        assert_eq!(rjson.resource.is_none(), true);
        assert_eq!(rjson.message.is_none(), true);
        assert_eq!(rjson.data.is_null(), true);

        // improper ok json (yet still parsed but everything will be moved in data)
        let json = serde_json::from_str(r#"{
            "success": true,
            "http_code": 201,
            "resource": "some resource requested",
            "method": "GET",
            "message": "error message"
        }"#).unwrap();
        let rjson = ResponseJSON::from_serde_value(json);
        assert_eq!(rjson.is_valid_json(), true);
        assert_eq!(rjson.is_ok_json(), true);
        assert_eq!(rjson.is_error_json(), false);
        assert_eq!(rjson.http_code, 200);
        assert_eq!(rjson.method.is_none(), true);
        assert_eq!(rjson.resource.is_none(), true);
        assert_eq!(rjson.message.is_none(), true);
        let val : Value = serde_json::from_str("201").unwrap();
        assert_eq!(rjson.data["http_code"], val);

        // ok json with data
        let json = serde_json::from_str(r#"{
            "success": true,
            "http_code": 202,
            "data": {
                "test1": "value1",
                "test2": "value2",
                "test3": [ 1, 2, 3 ]
            }
        }"#).unwrap();
        let rjson = ResponseJSON::from_serde_value(json);
        assert_eq!(rjson.is_valid_json(), true);
        assert_eq!(rjson.is_ok_json(), true);
        assert_eq!(rjson.is_error_json(), false);
        assert_eq!(rjson.http_code, 202);
        assert_eq!(rjson.data["test2"], Value::String("value2".to_string()));

        // error json without data
        let json = serde_json::from_str(r#"{
            "success": false,
            "http_code": 501,
            "resource": "some resource requested",
            "method": "GET",
            "message": "error message"
        }"#).unwrap();
        let rjson = ResponseJSON::from_serde_value(json);
        assert_eq!(rjson.is_valid_json(), true);
        assert_eq!(rjson.is_ok_json(), false);
        assert_eq!(rjson.is_error_json(), true);
        assert_eq!(rjson.http_code, 501);
        assert_eq!(rjson.resource.unwrap(), "some resource requested".to_string());

        // error json with data
        let json : Value = serde_json::from_str(r#"{
            "success": false,
            "http_code": 502,
            "data": {
                "test1": "value1",
                "test2": "value2",
                "test3": [ 1, 2, 3 ]
            },
            "resource": "some resource requested",
            "method": "GET",
            "message": "error message"
        }"#).unwrap();
        assert_eq!(json["data"]["test2"], Value::String("value2".to_string()));

        let rjson = ResponseJSON::from_serde_value(json);
        assert_eq!(rjson.is_valid_json(), true);
        assert_eq!(rjson.is_ok_json(), false);
        assert_eq!(rjson.is_error_json(), true);
        assert_eq!(rjson.http_code, 502);
        assert_eq!(rjson.resource.unwrap(), "some resource requested".to_string());
        assert_eq!(rjson.data["test1"], Value::String("value1".to_string()));

        // should not compile
        // assert_eq!(json["data"]["test2"], Value::String("value2".to_string()));
    }

    #[test]
    fn ResponseJSON_test_into_string() {
        let json = ResponseJSON::ok()
            .http_code(201)
            .data("Some data".into());

        let ref_json : Value = json!({
            "success": true, 
            "http_code": 201, 
            "data": "Some data"
        });

        let string = json.into_string();
        let test_json : Value = serde_json::from_str(&string).unwrap();
        assert_eq!(ref_json, test_json);
    }

    #[test]
    fn ResponseJSON_test_to_string() {
        let json = ResponseJSON::ok()
            .http_code(201)
            .data("Some data".into());

        let ref_json : Value = json!({
            "success": true, 
            "http_code": 201, 
            "data": "Some data"
        });
        assert_eq!(json.to_string(), ref_json.to_string());
        assert_eq!(json.is_ok_json(), true); // ensure value is not moved
    }

    #[test]
    fn ResponseJSON_test_eq() {
        let json = ResponseJSON::ok()
            .http_code(201)
            .data("Some data".into());

        let ref_json : Value = json!({
            "success": true, 
            "http_code": 201, 
            "data": "Some data"
        });
        assert_eq!(json, ref_json);
        assert_eq!(ref_json, json);
        assert_eq!(json, json);

        let string = ref_json.to_string();
        assert_eq!(json, string);
        assert_eq!(string, json);

        let str_slice : &str = string.as_ref();
        assert_eq!(json, str_slice);
        assert_eq!(str_slice, json);
    }

    #[test]
    fn ResponseJSON_test_route_with_ok_response_json() {
        let input_rjson = ResponseJSON::from_str(r#"{
            "success": true,
            "http_code": 200,
            "data": {
                "test1": "value1",
                "test2": [ 1, 2, 3 ]
            }
        }"#).unwrap();

        #[post("/test", data="<params>")]
        fn test_route(params: ResponseJSON) -> &'static str {
            assert_eq!(params.success, true);
            assert_eq!(params.http_code, 200);
            assert_eq!(params.data["test1"], Value::String("value1".to_string()));
            "It's working !"
        }

        let rocket = rocket::ignite()
            .mount("/post", routes![test_route]);

        let mut req = MockRequest::new(Method::Post, "/post/test")
            .header(ContentType::JSON)
            .body(input_rjson.clone().to_string());

        let mut response = req.dispatch_with(&rocket);
        let body_str = response.body().and_then(|b| b.into_string());

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(body_str, Some("It's working !".to_string()));
    }

    #[test]
    fn ResponseJSON_test_route_with_error_response_json() {
        let input_rjson = ResponseJSON::from_str(r#"{
            "success": false,
            "http_code": 500,
            "data": {
                "test1": "value1",
                "test2": [ 1, 2, 3 ]
            },
            "method": "GET",
            "resource": "/back/test",
            "message": "Unexpected error"
        }"#).unwrap();

        #[post("/test", data="<params>")]
        fn test_route(params: ResponseJSON) -> &'static str {
            assert_eq!(params.success, false);
            assert_eq!(params.http_code, 500);
            assert_eq!(params.method.unwrap(), "GET");
            assert_eq!(params.resource.unwrap(), "/back/test");
            assert_eq!(params.message.unwrap(), "Unexpected error");
            "It's working !"
        }

        let rocket = rocket::ignite()
            .mount("/post", routes![test_route]);

        let mut req = MockRequest::new(Method::Post, "/post/test")
            .header(ContentType::JSON)
            .body(input_rjson.clone().to_string());

        let mut response = req.dispatch_with(&rocket);
        let body_str = response.body().and_then(|b| b.into_string());

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(body_str, Some("It's working !".to_string()));
    }

    #[test]
    fn ResponseJSON_test_route_with_returned_response_json() {
        #[get("/test?<params>")]
        fn test_route(params: FormHashMap) -> ResponseJSON {
            let json = json!({
                "success": true,
                "http_code": 200,
                "data": {
                    "message": params["message"]
                }
            });
            ResponseJSON::from_serde_value(json)
        }

        let rocket = rocket::ignite()
            .mount("/get", routes![test_route]);

        let message = "hello_world";
        let mut req = MockRequest::new(Method::Get, "/get/test?message=".to_string() + message);

        let mut response = req.dispatch_with(&rocket);
        let body_str = response.body().and_then(|b| b.into_string()).unwrap();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(ResponseJSON::from_str(&body_str).unwrap(), ResponseJSON::from_serde_value(json!({
            "success": true,
            "http_code": 200,
            "data": {
                "message": message
            }
        })));
    }

    // TODO add test with Errors being generated
}   