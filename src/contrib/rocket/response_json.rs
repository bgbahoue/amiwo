// =======================================================================
// LIBRARY IMPORTS
// =======================================================================
use rocket;
use rocket::{ Request, Data };
use rocket::data::{ FromData, Outcome };
use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::response::Responder;
use std::marker::PhantomData;

use serde;
use ::serde::de::Deserialize;
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
pub struct ResponseJSON<'v> {
    success: bool,
    http_code: u16,
    data: &'v Value,
    message: Option<String>, // required for error JSON
    resource: Option<String>,
    method: Option<String>,
    _phantom: PhantomData<&'v Value>
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
            _phantom: PhantomData,
        }
    }
}

impl<'v> ResponseJSON<'v>
{
    // Create an empty OK ResponseJSON
    pub fn ok() -> ResponseJSON<'v> {
        ResponseJSON{
            success: true,
            http_code: 200,
            ..Default::default()
        }
    }

    // Create an empty OK ResponseJSON
    pub fn error() -> ResponseJSON<'v> {
        ResponseJSON{
            success: false,
            http_code: 500,
            message: Some("Unexpected error".to_string()),
            ..Default::default()
        }
    }

    /// Finalize ResponseJSON and transfer ownership to caller
    pub fn finalize(self) -> ResponseJSON<'v> {
        self
    }

    /// Set the HTTP Code of this ResponseJSON
    pub fn http_code(&mut self, code: u16) -> &mut ResponseJSON<'v> {
        self.http_code = code;
        self
    }

    /// Set the data of this ResponseJSON
    pub fn data(&mut self, ref_data: &'v Value) -> &mut ResponseJSON<'v> 
    {
        self.data = ref_data;
        self
    }

    /// Set the error message.
    /// For Error JSON only (does nothing if `success == ok`)    
    pub fn message(&mut self, string: String) -> &mut ResponseJSON<'v> {
        if !self.success {
            self.message = Some(string);
        } else {
            warn!("::AMIWO::RESPONSEJSON::MESSAGE::WARNING Trying to set `message` on an Ok JSON => ignored")
        }
        self
    }

    /// Set the resource that we tried to access.
    /// For Error JSON only (does nothing if `success == ok`)
    pub fn resource(&mut self, string: String) -> &mut ResponseJSON<'v> {
        if !self.success {
            self.resource = Some(string);
        } else {
            warn!("::AMIWO::RESPONSEJSON::MESSAGE::WARNING Trying to set `resource` on an Ok JSON => ignored")
        }
        self
    }

    /// Set the method that was used (GET, POST, ...).
    /// For Error JSON only (does nothing if `success == ok`)
    pub fn method(&mut self, string: String) -> &mut ResponseJSON<'v> {
        if !self.success {
            self.method = Some(string);
        } else {
            warn!("::AMIWO::RESPONSEJSON::METHOD::WARNING Trying to set `method` on an Ok JSON => ignored")
        }
        self
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
        self.get("http_code").is_some() && self.get("http_code").unwrap().is_number() &&
        self["method"].is_null() &&
        self["resource"].is_null() &&
        self["message"].is_null()
    }

    fn is_error_json(&self) -> bool {
        self.is_valid_json() && 
        self["success"] == Value::Bool(false) &&
        self.get("http_code").is_some() && self.get("http_code").unwrap().is_number()
    }
}

impl IsResponseJSON for Value {
    fn is_valid_json(&self) -> bool {
        self.contains_keys(&["success", "http_code"]) 
    }

    fn is_ok_json(&self) -> bool {
        self.is_valid_json() && 
        self["success"] == Value::Bool(true) &&
        self.get("http_code").is_some() && self.get("http_code").unwrap().is_number() &&
        self["method"].is_null() &&
        self["resource"].is_null() &&
        self["message"].is_null()
    }

    fn is_error_json(&self) -> bool {
        self.is_valid_json() && 
        self["success"] == Value::Bool(false) &&
        self.get("http_code").is_some() && self.get("http_code").unwrap().is_number()
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
/// ResponseJSON<T> can be created from a serde_json::JSON typed as `Value`
/// If the input JSON is like { success: false, http_code, message, resource, method } it creates an Error ResponseJSON 
/// Else it creates an Ok ResponseJSON with it's data property set to the input JSON
impl From<Value> for ResponseJSON<Value>
{
    fn from(json: Value) -> Self {
        json.as_object() // Option<&Map<String, Value>>
            .map_or_else( // compute the data to be wrapped in the ResponseJSON
                // None => not an object
                || Err(&json),
                // Some(json) => check if is a valid ResponseJSON
                |obj| {
                    if obj.is_valid_json() {
                        Ok(
                            ResponseJSON::ok()
                                .http_code(obj["http_code"].as_u64().unwrap() as u16)
                                .data(obj["data"])
                                .finalize()
                        )
                    } else {
                        Err(&*obj as Value)
                    }
                }
            ).unwrap_or_else(|data| ResponseJSON::ok().data(data).finalize() )
    }
}
*/

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
        let mut json = ResponseJSON::ok();
        assert_eq!(json.success, true);
        assert_eq!(json.http_code, 200);
        assert!(json.data.is_null());
        assert_eq!(json.message, None);
        assert_eq!(json.method, None);
        assert_eq!(json.resource, None);

        json.http_code(201).data(&data).method("GET".to_string()).resource("some path".to_string()).message("error message".to_string());
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
        let mut json = ResponseJSON::error();
        assert_eq!(json.success, false);
        assert_eq!(json.http_code, 500);
        assert!(json.data.is_null());
        assert_eq!(json.message, Some("Unexpected error".to_string()));
        assert_eq!(json.method, None);
        assert_eq!(json.resource, None);

        json.http_code(401).data(&data).method("GET".to_string()).resource("some path".to_string()).message("error message".to_string());
        assert_eq!(json.http_code, 401);
        assert_eq!(json.data.as_str(), Some("Some data"));
        assert_eq!(json.message, Some("error message".to_string()));
        assert_eq!(json.method, Some("GET".to_string()));
        assert_eq!(json.resource, Some("some path".to_string()));

        assert_eq!(json.is_valid_json(), true);
        assert_eq!(json.is_ok_json(), false);
        assert_eq!(json.is_error_json(), true);
    }

    // TODO add test `from` Serde JSON

    // TODO add test with POST & GET routes taking a ResponseJSON as param
}   