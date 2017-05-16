//! File holding the ResponseJSON type and associated tests
//!
//! Author: [Boris](mailto:boris@humanenginuity.com)
//! Version: 1.1
//!
//! ## Release notes
//! - v1.0 : creation

// =======================================================================
// LIBRARY IMPORTS
// =======================================================================
use rocket;
use rocket::{ Request, Data };
use rocket::data::{ FromData, Outcome };
use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::response::Responder;

use serde;
use serde::de::Deserialize;
use serde_json;
use serde_json::Value;

use util::ContainsKeys;

// =======================================================================
// CONSTANTS
// =======================================================================
static NULL: Value = Value::Null;

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
#[derive(Debug)]
pub struct ResponseJSON<'v> {
    success: bool,
    http_code: u16,
    data: &'v Value,
    message: Option<String>, // required for error JSON
    resource: Option<String>,
    method: Option<String>,
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
/// Default values for ResponseJSON are
///     - success: true
///     - http_code: 200
///     - data: Value::Null
///     - message: None
///     - resource: None
///     - method: None
impl<'v> Default for ResponseJSON<'v>
{
    fn default () -> ResponseJSON<'v> {
        ResponseJSON {
            success: true,
            http_code: 200,
            data: &NULL, // Returns a reference to `NULL` where its `'static` lifetime is coerced to 'v' 
            message: None,
            resource: None,
            method: None,
        }
    }
}

impl<'v> ResponseJSON<'v>
{
    // Create an empty OK ResponseJSON
    pub fn ok() -> ResponseJSON<'v> {
        ResponseJSON {
            success: true,
            http_code: 200,
            data: &NULL, // Returns a reference to `NULL` where its `'static` lifetime is coerced to 'v' 
            message: None,
            resource: None,
            method: None,
        }
    }

    // Create an empty OK ResponseJSON
    pub fn error() -> ResponseJSON<'v> {
        ResponseJSON {
            success: false,
            http_code: 500,
            data: &NULL, // Returns a reference to `NULL` where its `'static` lifetime is coerced to 'v' 
            message: Some("Unexpected error".to_string()),
            resource: None,
            method: None,
        }
    }

    /// Set the HTTP Code of this ResponseJSON
    pub fn http_code(mut self, code: u16) -> ResponseJSON<'v> {
        self.http_code = code;
        self
    }

    /// Set the data of this ResponseJSON
    pub fn data(mut self, ref_data: &'v Value) -> ResponseJSON<'v> 
    {
        self.data = ref_data;
        self
    }

    /// Set the error message.
    /// For Error JSON only (does nothing if `success == ok`)    
    pub fn message(mut self, string: String) -> ResponseJSON<'v> {
        if !self.success {
            self.message = Some(string);
        } else {
            warn!("::AMIWO::RESPONSEJSON::MESSAGE::WARNING Trying to set `message` on an Ok JSON => ignored")
        }
        self
    }

    /// Set the resource that we tried to access.
    /// For Error JSON only (does nothing if `success == ok`)
    pub fn resource(mut self, string: String) -> ResponseJSON<'v> {
        if !self.success {
            self.resource = Some(string);
        } else {
            warn!("::AMIWO::RESPONSEJSON::MESSAGE::WARNING Trying to set `resource` on an Ok JSON => ignored")
        }
        self
    }

    /// Set the method that was used (GET, POST, ...).
    /// For Error JSON only (does nothing if `success == ok`)
    pub fn method(mut self, string: String) -> ResponseJSON<'v> {
        if !self.success {
            self.method = Some(string);
        } else {
            warn!("::AMIWO::RESPONSEJSON::METHOD::WARNING Trying to set `method` on an Ok JSON => ignored")
        }
        self
    }

    /// ResponseJSON<T> can be created from a `serde_json::Value`, consuming the original object
    /// If the input is a valid ResponseJSON it duplicates it
    /// Else it creates an Ok ResponseJSON with it's data property set to the input JSON
    fn from_serde_value(json: &'v Value) -> ResponseJSON<'v> {
        if json.is_object() {
            if json.is_ok_json() {
                ResponseJSON::ok()
                    .http_code(json["http_code"].as_u64().unwrap() as u16)
                    .data(json.get("data").unwrap_or(&NULL))
            } else if json.is_error_json() {
                let mut rjson = ResponseJSON::error()
                    .http_code(json["http_code"].as_u64().unwrap() as u16)
                    .data(json.get("data").unwrap_or(&NULL));

                if !json["message"].is_null() { rjson = rjson.message(json["message"].as_str().unwrap().to_string()); }
                if !json["resource"].is_null() { rjson = rjson.resource(json["resource"].as_str().unwrap().to_string()); }
                if !json["method"].is_null() { rjson = rjson.method(json["method"].as_str().unwrap().to_string()); }

                rjson
            } else {
                ResponseJSON::ok()
                    .data(json.pointer("").unwrap())
            }
        } else {
            ResponseJSON::ok()
                .data(json.pointer("").unwrap())
        }
    }
}

// =======================================================================
// TRAIT IMPLEMENTATION
// ======================================================================
impl<'v> IsResponseJSON for ResponseJSON<'v> {
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

/*
/// Implement Rocket's FormData to parse a ResponseJSON from incoming POST/... form data.
///
/// - If the content type of the request data is not
/// `application/json`, `Forward`s the request.
///
/// All relevant warnings and errors are written to the console
impl<T: DeserializeOwned> FromData for ResponseJSON<T> {
    type Error = serde_json::error::Error;

    fn from_data(request: &Request, data: Data) -> Outcome<Self, serde_json::error::Error> {
        if !request.content_type().map_or(false, |ct| ct.is_json()) {
            error!("Content-Type is not JSON.");
            return Outcome::Forward(data);
        }

        let size_limit = rocket::config::active()
            .and_then(|c| c.extras.get("limits.json")) // TODO: remove placeholder when upgrading to rocket version > 0.2.6
            // .and_then(|c| c.limits.get("json") // In next version
            .and_then(|limit| limit.as_integer())
            .unwrap_or(32768) as u64;

        serde_json::from_reader(data.open().take(size_limit))
            .map(|val| JSON(val))
            .map_err(|e| { error!("Couldn't parse JSON body: {:?}", e); e })
            .into_outcome(Status::BadRequest)

    }
}
*/

/*
/// Serializes the wrapped value into JSON. Returns a response with Content-Type
/// JSON and a fixed-size body with the serialized value. If serialization
/// fails, an `Err` of `Status::InternalServerError` is returned.
impl<T: Serialize> Responder<'static> for JSON<T> {
    fn respond(self) -> response::Result<'static> {
        serde_json::to_string(&self.0).map(|string| {
            content::JSON(string).respond().unwrap()
        }).map_err(|e| {
            error_!("JSON failed to serialize: {:?}", e);
            Status::InternalServerError
        })
    }
}
*/

// =======================================================================
// UNIT TESTS
// =======================================================================
#[cfg(test)]
mod tests {
    use super::ResponseJSON;
    use super::IsResponseJSON;
    use serde_json;
    use serde_json::Value;

    #[allow(non_snake_case)]
    #[test]
    fn test_IsResponseJSON_implem() {
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
    fn test_builder_ok() {
        let data : Value = "Some data".into();

        let json : ResponseJSON = ResponseJSON::ok();
        assert_eq!(json.success, true);
        assert_eq!(json.http_code, 200);
        assert!(json.data.is_null());
        assert_eq!(json.message, None);
        assert_eq!(json.method, None);
        assert_eq!(json.resource, None);

        let json : ResponseJSON = ResponseJSON::ok()
            .http_code(201)
            .data(&data)
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
    fn test_builder_error() {
        let data : Value = "Some data".into();

        let json : ResponseJSON = ResponseJSON::error();
        assert_eq!(json.success, false);
        assert_eq!(json.http_code, 500);
        assert!(json.data.is_null());
        assert_eq!(json.message, Some("Unexpected error".to_string()));
        assert_eq!(json.method, None);
        assert_eq!(json.resource, None);

        let json : ResponseJSON = ResponseJSON::error()
            .http_code(401)
            .data(&data)
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
    fn test_from_serde_json() {
        // Simple non ResponseJSON
        let json = serde_json::from_str(r#"{
            "test1": "value1",
            "test2": "value2",
            "test3": [ 1, 2, 3 ]
        }"#).unwrap();
        let rjson = ResponseJSON::from_serde_value(&json);
        assert_eq!(rjson.is_valid_json(), true);
        assert_eq!(rjson.is_ok_json(), true);
        assert_eq!(rjson.is_error_json(), false);
        assert_eq!(rjson.data["test2"], Value::String("value2".to_string()));

        // ok json without data
        let json = serde_json::from_str(r#"{
            "success": true,
            "http_code": 204
        }"#).unwrap();
        let rjson = ResponseJSON::from_serde_value(&json);
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
        let rjson = ResponseJSON::from_serde_value(&json);
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
        let rjson = ResponseJSON::from_serde_value(&json);
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
        let rjson = ResponseJSON::from_serde_value(&json);
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

        let rjson = ResponseJSON::from_serde_value(&json);
        assert_eq!(rjson.is_valid_json(), true);
        assert_eq!(rjson.is_ok_json(), false);
        assert_eq!(rjson.is_error_json(), true);
        assert_eq!(rjson.http_code, 502);
        assert_eq!(rjson.resource.unwrap(), "some resource requested".to_string());
        assert_eq!(rjson.data["test1"], Value::String("value1".to_string()));
        // json should still be accessible
        assert_eq!(json["data"]["test2"], Value::String("value2".to_string()));
    }

    // TODO add test with POST & GET routes taking a ResponseJSON as param
}   