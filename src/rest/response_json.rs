// =======================================================================
// LIBRARY IMPORTS
// =======================================================================
use rocket::{ Request, Data };
use rocket::data::Outcome;
use rocket::outcome::IntoOutcome;
use rocket::response::Responder;
use rocket_contrib;

use serde;
use ::serde::de::Deserialize;
use serde::de::DeserializeOwned;
use serde_json;
use serde_json::Value;

use util;

// =======================================================================
// STRUCT DEFINITION
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
pub struct ResponseJSON<T: DeserializeOwned> {
    success: bool,
    http_code: u16,
    data: Option<T>,
    message: Option<String>, // required for error JSON
    resource: Option<String>,
    method: Option<String>,
}

/// Default values for ResponseJSON are
///     - data: None
///     - message: None
///     - resource: None
///     - method: None
impl<T: DeserializeOwned> Default for ResponseJSON<T> {
    fn default () -> ResponseJSON<T> {
        ResponseJSON {
            success: true,
            http_code: 200,
            data: None,
            message: None,
            resource: None,
            method: None,
        }
    }
}

impl<T: DeserializeOwned> ResponseJSON<T> {
    // Create an empty OK ResponseJSON
    pub fn ok() -> ResponseJSON<T> {
        ResponseJSON{
            success: true,
            http_code: 200,
            ..Default::default()
        }
    }

    // Create an empty OK ResponseJSON
    pub fn error() -> ResponseJSON<T> {
        ResponseJSON{
            success: false,
            http_code: 500,
            message: Some("Unexpected error".to_string()),
            ..Default::default()
        }
    }

    /// Finalize ResponseJSON and transfer ownership to caller
    pub fn finalize(self) -> ResponseJSON<T> {
        self
    }

    /// Set the HTTP Code of this ResponseJSON
    pub fn http_code(&mut self, code: u16) -> &mut ResponseJSON<T> {
        self.http_code = code;
        self
    }

    /// Set the data of this ResponseJSON
    pub fn data(&mut self, data: T) -> &mut ResponseJSON<T> {
        self.data = Some(data);
        self
    }

    /// Set the error message.
    /// For Error JSON only (does nothing if `success == ok`)    
    pub fn message(&mut self, string: String) -> &mut ResponseJSON<T> {
        if !self.success {
            self.message = Some(string);
        } else {
            warn!("::AMIWO::RESPONSEJSON::MESSAGE::WARNING Trying to set `message` on an Ok JSON => ignored")
        }
        self
    }

    /// Set the resource that we tried to access.
    /// For Error JSON only (does nothing if `success == ok`)
    pub fn resource(&mut self, string: String) -> &mut ResponseJSON<T> {
        if !self.success {
            self.resource = Some(string);
        } else {
            warn!("::AMIWO::RESPONSEJSON::MESSAGE::WARNING Trying to set `resource` on an Ok JSON => ignored")
        }
        self
    }

    /// Set the method that was used (GET, POST, ...).
    /// For Error JSON only (does nothing if `success == ok`)
    pub fn method(&mut self, string: String) -> &mut ResponseJSON<T> {
        if !self.success {
            self.method = Some(string);
        } else {
            warn!("::AMIWO::RESPONSEJSON::METHOD::WARNING Trying to set `method` on an Ok JSON => ignored")
        }
        self
    }

    /// Check if the JSON described as a String is an Error JSON
    pub fn is_error_json(json_as_str: &str) -> bool
    {
        !Self::is_ok_json(json_as_str)
    }

    /// Check if the JSON described as a String is an OK JSON
    pub fn is_ok_json(json_as_str: &str) -> bool {
        match serde_json::from_str::<Value>(json_as_str) {
            Ok(json) => util::has_properties(&json, &["success", "http_code"]) 
                        && json["success"] == Value::Bool(true) 
                        && json["method"] == Value::Null 
                        && json["resource"] == Value::Null 
                        && json["message"] == Value::Null, 
            Err(_) => false,
        }
    }

    /// Check if the current object is an Error JSON
    pub fn is_error(&self) -> bool {
        self.success == false
    }

    /// Check if the current object is an OK JSON
    pub fn is_ok(&self) -> bool {
        self.success == true
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

/*
/// ResponseJSON<T> can be created from a serde_json::JSON typed as `serde_json::Value`
/// If the input JSON is like { success: false, http_code, message, resource, method } it creates an Error ResponseJSON 
/// Else it creates an Ok ResponseJSON with it's data property set to the input JSON
impl From<serde_json::Value> for ResponseJSON<T>
    where T: Deserialize
{
    fn from(json: serde_json::JSON<T>) -> Self {
        match serde_json::from_str::<serde_json::Value>(json.to_string()) {
            Ok(json) => {
                if ResponseJSON::is_error_json(&json)
            },
            Err(err) => {

            },
        }
    }
}
*/

// =======================================================================
// UNIT TESTS
// =======================================================================
#[cfg(test)]
mod tests {
    use super::ResponseJSON;
    use serde_json;
    use serde_json::Value;

    #[test]
    fn test_is_ok_and_error_json() {
        let json = r#"{
            "success": false,
            "http_code": 500,
            "resource": "some resource requested",
            "method": "GET",
            "message": "error message"
        }"#;
        assert_eq!(ResponseJSON::<String>::is_error_json(json), true);
        assert_eq!(ResponseJSON::<String>::is_ok_json(json), false);

        let json = r#"{
            "success": true,
            "http_code": 200,
            "resource": "some resource requested",
            "method": "GET",
            "message": "error message"
        }"#;
        assert_eq!(ResponseJSON::<String>::is_error_json(json), true);
        assert_eq!(ResponseJSON::<String>::is_ok_json(json), false);
    }

    #[test]
    fn test_builder_ok() {
        let mut json = ResponseJSON::ok();
        assert_eq!(json.success, true);
        assert_eq!(json.http_code, 200);
        assert_eq!(json.data, None);
        assert_eq!(json.message, None);
        assert_eq!(json.method, None);
        assert_eq!(json.resource, None);

        json.http_code(201).data("Some data".to_string()).method("GET".to_string()).resource("some path".to_string()).message("error message".to_string());
        assert_eq!(json.http_code, 201);
        assert_eq!(json.data, Some("Some data".to_string()));
        assert_eq!(json.message, None);
        assert_eq!(json.method, None);
        assert_eq!(json.resource, None);
        
        assert_eq!(json.is_ok(), true);
        assert_eq!(json.is_error(), false);
    }

    #[test]
    fn test_builder_error() {
        let mut json = ResponseJSON::error();
        assert_eq!(json.success, false);
        assert_eq!(json.http_code, 500);
        assert_eq!(json.data, None);
        assert_eq!(json.message, Some("Unexpected error".to_string()));
        assert_eq!(json.method, None);
        assert_eq!(json.resource, None);

        json.http_code(401).data("Some data".to_string()).method("GET".to_string()).resource("some path".to_string()).message("error message".to_string());
        assert_eq!(json.http_code, 401);
        assert_eq!(json.data, Some("Some data".to_string()));
        assert_eq!(json.message, Some("error message".to_string()));
        assert_eq!(json.method, Some("GET".to_string()));
        assert_eq!(json.resource, Some("some path".to_string()));

        assert_eq!(json.is_ok(), false);
        assert_eq!(json.is_error(), true);
    }

    // TODO add test from Serde JSON
    // TODO add test with POST & GET routes taking a ResponseJSON as param
}   