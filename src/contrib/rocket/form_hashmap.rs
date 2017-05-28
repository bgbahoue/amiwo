//! File holding the FormHashMap type accepting variable parameters from a Rocket request 
//! exposing a simplified Map type interface to access them
//!
//! Author: [Boris](mailto:boris@humanenginuity.com)
//! Version: 2.0
//!
//! ## Release notes
//! - v2.0 : refactored using serde_json Map & Value
//! - v1.1 : implemented Index trait, renamed old `new()` method into `from_application_data`, added method `from_json_data`
//! - v1.0 : creation

// =======================================================================
// ATTRIBUTES
// =======================================================================
#![allow(dead_code)]

// =======================================================================
// LIBRARY IMPORTS
// =======================================================================
use std::convert::AsRef;
use std::error::Error;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::io::Read;
use std::ops::Index;

use rocket;
use rocket::{ Request, Data };
use rocket::data::FromData;
use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::request::{ FromForm, FromFormValue, FormItems };

use serde_json;
use serde_json::Value;
use serde_json::map::Map;

use error::GenericError;
use traits::Pushable;

// =======================================================================
// STRUCT & TRAIT DEFINITION
// =======================================================================
/// A `FromData` type that creates a map of the key/value pairs from a
/// `x-www-form-urlencoded` or `json` form string 
pub struct FormHashMap<'s> {
    form_string: String,
    map: Map<String, Value>,
    _phantom: PhantomData<&'s str>,
}


// =======================================================================
// IMPLEMENTATION
// =======================================================================

impl<'s> FormHashMap<'s> {
    /// Get a reference for the value (or values) associated with `key`.
    pub fn get<T: AsRef<str>>(&self, key: T) -> Option<&Value> {
        self.map.get(key.as_ref())
    }

    /// Returns the raw form string that was used to parse the encapsulated
    /// object.
    pub fn raw_form_string(&self) -> &str {
        &self.form_string
    }

    /// Build a FormHashMap from application data (i.e. content type application/x-www-form-urlencoded)
    /// Uses Rocket's `FormItems::from<'f>(&'f str)` to parse the form's String
    fn from_application_data(form_string: String) -> Result<Self, GenericError> {
        let long_lived_string: &'s str = unsafe {
            ::std::mem::transmute(form_string.as_str())
        };
        let mut items = FormItems::from(long_lived_string);

        // Handle parsing or decode errors
        let parsing_errors: Vec<_> = items.by_ref()
            .map(|(key, value)| (key, String::from_form_value(value)))
            .filter(|&(_, ref decoded_value)| decoded_value.is_err())
            .collect();

        if !parsing_errors.is_empty() {
            return amiwo_error!( format!("::AMIWO::CONTRIB::ROCKET::FORM_HASHMAP::FROM_APPLICATION_DATA::WARNING Unable to parse form string {} [parsing errors = {:?}]", form_string, parsing_errors) );
        }
        if !items.completed() {
            warn!("::AMIWO::CONTRIB::ROCKET::FORM_HASHMAP::FROM_APPLICATION_DATA::WARNING Form string {} couldn't be completely parsed", form_string);
        }

        Ok(FormHashMap {
            form_string: form_string,
            map: FormItems::from(long_lived_string)
                .map(|(key, value)| (key, String::from_form_value(value)))
                .filter(|&(_, ref decoded_value)| decoded_value.is_ok())
                .fold(
                    Map::new(),
                    |mut map, (key, decoded_value)| {
                        map.entry(key).or_insert(Value::Null).push(Value::String(decoded_value.unwrap()));
                        map
                    }
                ),
            _phantom: PhantomData,
        })
    }

    /// Build a FormHashMap from JSON data (i.e. content type application/json)
    /// Uses serde_json's `serde_json::from_str<'a, T>(&'a str)` to parse the form's String
    fn from_json_data(form_string: String) -> Result<Self, GenericError> {
        let long_lived_string: &'s str = unsafe {
            ::std::mem::transmute(form_string.as_str())
        };
        serde_json::from_str(long_lived_string)
            .or_else(|err| amiwo_error!(
                format!("::AMIWO::CONTRIB::ROCKET::FORM_HASHMAP::FROM_JSON_DATA::ERROR Error parsing string {} > {}", form_string, &err.description()),
                GenericError::Serde(err)
            )).and_then(|value : Value| {
                if value.is_object() {
                    Ok(FormHashMap {
                        form_string: form_string,
                        map: value.as_object().unwrap().clone(),
                        _phantom: PhantomData,
                    })
                } else {
                    amiwo_error!(format!(":AMIWO::CONTRIB::ROCKET::FORM_HASHMAP::FROM_JSON_DATA::ERROR Invalid JSON data {}", form_string))
                }
            })
    }

    // We'd like to have form objects have pointers directly to the form string. 
    // This means that the form string has to live at least as long as the form object. So,
    // to enforce this, we store the form_string along with the form object.
    //
    // So far so good. Now, this means that the form_string can never be
    // deallocated while the object is alive. That implies that the
    // `form_string` value should never be moved away. We can enforce that
    // easily by 1) not making `form_string` public, and 2) not exposing any
    // `&mut self` methods that could modify `form_string`.
    fn new(content_type: &str, form_string: String) -> Result<Self, GenericError> {
        match content_type {
            "application" => FormHashMap::from_application_data(form_string),
            "json" => FormHashMap::from_json_data(form_string),
            _ => amiwo_error!(format!("::AMIWO::CONTRIB::ROCKET::FORM_HASHMAP::NEW::ERROR Unsupported content type {}", content_type)),
        }
    }
}

// =======================================================================
// EXTERNAL TRAITS IMPLEMENTATION
// =======================================================================
/// Parses a `FormHashMap` from incoming POST/... form data.
///
/// - If the content type of the request data is not
/// `application/x-www-form-urlencoded` or `application/json`, `Forward`s the request.
/// - If the form string is malformed, a `Failure` with status code 
/// `BadRequest` is returned. 
/// - Finally, if reading the incoming stream fails, returns a `Failure` with status code
/// `InternalServerError`.
/// In all failure cases, the raw form string is returned if it was able to be retrieved from the incoming stream.
///
/// All relevant warnings and errors are written to the console
impl<'f> FromData for FormHashMap<'f> {
    type Error = GenericError;

    fn from_data(request: &Request, data: Data) -> rocket::data::Outcome<Self, Self::Error> {
        if !request.content_type().map_or(false, |ct| ct.is_form() || ct.is_json()) {
            error!("::AMIWO::CONTRIB::ROCKET::FORM_HASHMAP::FROM_DATA::WARNING Form data does not have application/x-www-form-urlencoded or application/json content type.");
            return rocket::Outcome::Forward(data);
        }

        let content_type = request.content_type().map_or("unsupported content type", |ct| if ct.is_form() { "application" } else { "json" });

        let size_limit = rocket::config::active()
            .and_then(|c| c.extras.get(&("limits.".to_string() + content_type))) // TODO: remove placeholder when upgrading to rocket version > 0.2.6
            // .and_then(|c| c.limits.get("application") // In next version
            .and_then(|limit| limit.as_integer())
            .unwrap_or_else(|| if content_type == "json" { 1<<20 } else { 32768 }) as u64;

        let mut buffer = String::new();
        data.open()
            .take(size_limit)
            .read_to_string(&mut buffer)
            .or_else(|err| amiwo_error!(format!("::AMIWO::CONTRIB::ROCKET::FORM_HASHMAP::FROM_DATA::ERROR IO Error: {}", err.description())) )
            .and_then(|_| FormHashMap::new(content_type, buffer))
            .or_else(|error_message| {
                error!("{}", error_message);
                Err(error_message)
            }).into_outcome() // Note: trait implemented by Rocket FromData for Result<S,E> `fn into_outcome(self, status: Status) -> Outcome<S, E>`
    }
}

/// Parses a `FormHashMap` from incoming GET query strings.
///
/// - If the form string is malformed, a `Failure` with status code 
/// `BadRequest` is returned. 
/// - Finally, if reading the incoming stream fails, returns a `Failure` with status code
/// `InternalServerError`.
/// In all failure cases, the raw form string is returned if it was able to be retrieved from the incoming stream.
///
/// All relevant warnings and errors are written to the console
impl<'f> FromForm<'f> for FormHashMap<'f> {
    /// The raw form string, if it was able to be retrieved from the request.
    type Error = (Status, Option<GenericError>);

    fn from_form_items(items: &mut FormItems<'f>) -> Result<Self, Self::Error> {
        FormHashMap::from_application_data(items.inner_str().to_string())
            .map(|map| {
                info!("::AMIWO::CONTRIB::ROCKET::FORM_HASHMAP::FROM_FORM_ITEMS::INFO Successfully parsed input data => {:?}", map);
                map
            }).map_err(|invalid_string| {
                error!("::AMIWO::CONTRIB::ROCKET::FORM_HASHMAP::FROM_FORM_ITEMS::ERROR The request's form string '{}' was malformed.", invalid_string);
                ( Status::BadRequest, Some(GenericError::Basic(format!("::AMIWO::CONTRIB::ROCKET::FORM_HASHMAP::FROM_FORM_ITEMS::ERROR The request's form string '{}' was malformed.", invalid_string))) )
            })
    }
}

/// Implement Debug displaying '<internal data holding structure>' from string: <parsed string>'
impl<'f> Debug for FormHashMap<'f> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{:?} from string: {:?}", self.map, self.form_string)
    }
}

/// Access an element of this type. Panics if the key is not defined
impl<'a, I: AsRef<str>> Index<I> for FormHashMap<'a> {
    type Output = Value;

    fn index(&self, index: I) -> &Self::Output {
        &self.map[index.as_ref()]
    }
}

// =======================================================================
// UNIT TESTS
// =======================================================================
#[cfg(test)]
mod tests {
    #![allow(unmounted_route)]
    #![allow(non_snake_case)]

    use super::FormHashMap;

    use rocket;
    use rocket::testing::MockRequest;
    use rocket::http::{ ContentType, Method, Status };

    #[test]
    fn FormHashMap_test_new() {
        let form_string = "a=b1&a=b2&b=c";

        match FormHashMap::from_application_data(form_string.to_string()) {
            Ok(map) => {
                assert_eq!(map.get("a"), Some(&json!(["b1", "b2"])));
                assert_eq!(map.get("b"), Some(&json!("c")));
                assert_eq!(map["b"], json!("c"));
                assert_eq!(map["b".to_string()], json!("c"));
            },
            Err(invalid_string) => {
                panic!("Unable to parse {}", invalid_string);
            }
        }
    }

    #[test]
    fn FormHashMap_test_post_route() {
        #[post("/test", data= "<params>")]
        fn test_route(params: FormHashMap) -> &'static str {
            assert_eq!(params.get("a"), Some(&json!(["b1", "b2"])));
            assert_eq!(params.get("b"), Some(&json!("c")));
            "It's working !"
        }

        let rocket = rocket::ignite()
            .mount("/post", routes![test_route]);

        let mut req = MockRequest::new(Method::Post, "/post/test")
            .header(ContentType::Form)
            .body("a=b1&a=b2&b=c");

        let mut response = req.dispatch_with(&rocket);
        let body_str = response.body().and_then(|b| b.into_string());

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(body_str, Some("It's working !".to_string()));
    }

    #[test]
    fn FormHashMap_test_get_route() {
        #[get("/test?<params>")]
        fn test_route(params: FormHashMap) -> &'static str {
            assert_eq!(params.get("a"), Some(&json!(["b1", "b2"])));
            assert_eq!(params.get("b"), Some(&json!("c")));
            "It's working !"
        }

        let rocket = rocket::ignite()
            .mount("/get", routes![test_route]);

        let mut req = MockRequest::new(Method::Get, "/get/test?a=b1&a=b2&b=c");

        let mut response = req.dispatch_with(&rocket);
        let body_str = response.body().and_then(|b| b.into_string());

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(body_str, Some("It's working !".to_string()));
    }

    #[test]
    fn FormHashMap_test_get_qs_with_dot() {
        #[get("/test?<params>")]
        fn test_route(params: FormHashMap) -> &'static str {
            assert_eq!(params.get("v"), Some(&json!("4.7.0")));
            "It's working !"
        }

        let rocket = rocket::ignite()
            .mount("/get", routes![test_route]);

        let mut req = MockRequest::new(Method::Get, "/get/test?v=4.7.0");

        let mut response = req.dispatch_with(&rocket);
        let body_str = response.body().and_then(|b| b.into_string());

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(body_str, Some("It's working !".to_string()));
    }

    // TODO: add test lifetime
}