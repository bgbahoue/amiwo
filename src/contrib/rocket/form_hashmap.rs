//! File holding the FormHashMap type and associated tests
//!
//! Author: [Boris](mailto:boris@humanenginuity.com)
//! Version: 1.1

// =======================================================================
// ATTRIBUTES
// =======================================================================
#![allow(dead_code)]

// =======================================================================
// LIBRARY IMPORTS
// =======================================================================
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::io::Read;
use std::marker::PhantomData;

use rocket;
use rocket::{ Request, Data };
use rocket::data::FromData;
use rocket::http::Status;
use rocket::outcome::IntoOutcome;
use rocket::request::{ FromForm, FromFormValue, FormItems };

use types::OneOrMany;

// =======================================================================
// FORMVALUES INNER STRUCT DEFINITION
// =======================================================================
/// Private structure to hold a random number of values
/// providing utility methods to `push()`or `get()` the underlying data
struct FormValues<'v, T: 'v> {
    value: Option<OneOrMany<T>>,
    _phantom: PhantomData<&'v T>,
}

/// Generic structure to hold the value in a key/value pair
/// since we can have an arbitrary number of values associated to a key
impl<'v, T: 'v> FormValues<'v, T> {
    fn new() -> FormValues<'v, T> {
        FormValues {
            value: None,
            _phantom: PhantomData
        }
    }

    /// Set/add a new value
    fn push(&mut self, new_value: T) {
        let old_value = ::std::mem::replace(&mut self.value, None);
        self._phantom = PhantomData;
        self.value = match old_value {
            None => {
                Some(OneOrMany::One(new_value))
            }
            Some(OneOrMany::One(existing_value)) => {
                Some(OneOrMany::Many(vec![existing_value, new_value]))
            }
            Some(OneOrMany::Many(mut vec)) => {
                vec.push(new_value);
                Some(OneOrMany::Many(vec))
            }
        }
    }

    /// Get a reference to this object's value
    fn get(&self) -> Option<&OneOrMany<T>> {
        self.value.as_ref()
    }

    /// Consume this object (since it takes `self`) and move out its content
    fn into_inner(self) -> Option<OneOrMany<T>> {
        self.value
    }
}

impl<'v, T: Debug + 'v> Debug for FormValues<'v, T> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "FormValues {{ values: {:?} }}", self.value)
    }
}

// =======================================================================
// FORMHASHMAP (implement FormData and FromForm)
// =======================================================================

/// A `FromData` type that creates a map of the key/value pairs from a
/// `x-www-form-urlencoded` form `string`.
pub struct FormHashMap<'f> {
    form_string: String,
    form_hash: HashMap<&'f str, FormValues<'f, String>>,
    _phantom: PhantomData<&'f String>,
}

impl<'f> FormHashMap<'f> {
    /// Get a reference for the value (or values) associated with `key`.
    pub fn get(&'f self, key: &str) -> Option<&OneOrMany<String>> {
        // Note: The map method takes the self argument by value, consuming the original
        // `HashMap::get()` returns a reference to the value corresponding to the key
        // so we then consume *that* reference with `map` 
        self.form_hash
            .get(key) // Option<&FormValues<String>> corresponding to the key
            .map(|form_value| form_value.get()) 
            .unwrap() // converted to an Option<&OneOrMany<String>> to 'hide' FormValues
    }

    /// Returns the raw form string that was used to parse the encapsulated
    /// object.
    pub fn raw_form_string(&self) -> &str {
        &self.form_string
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
    fn new(form_string: String) -> Result<Self, String> {
        let long_lived_string: &'f str = unsafe {
            ::std::mem::transmute(form_string.as_str())
        };

        let mut items = FormItems::from(long_lived_string);

        // Handle parsing or decode errors
        let parsing_errors: Vec<_> = items.by_ref()
            .map(|(key, value)| (key, String::from_form_value(value)))
            .filter(|&(_, ref decoded_value)| decoded_value.is_err())
            .collect();

        if !parsing_errors.is_empty() {
            // Parsing errors => fail with invalid result
            return Err(format!("::AMIWO::CONTRIB::ROCKET::FORM_HASMAP::NEW::WARNING Unable to parse form string {}", form_string));
        }
        if !items.completed() {
            warn!("::AMIWO::CONTRIB::ROCKET::FORM_HASMAP::NEW::WARNING Form string {} couldn't be completely parsed", form_string);
        }

        Ok(FormHashMap {
            form_string: form_string,
            form_hash: FormItems::from(long_lived_string).by_ref()
                            .map(|(key, value)| (key, String::from_form_value(value)))
                            .filter(|&(_, ref decoded_value)| decoded_value.is_ok())
                            .fold(
                                HashMap::<&'f str, FormValues<String>>::new(),
                                |mut map, (key, decoded_value)| {
                                    map.entry(key).or_insert(FormValues::new()).push(decoded_value.unwrap());
                                    map
                                }
                            ),
            _phantom: PhantomData,
        })
    }
}

/// Parses a `FormHashMap` from incoming POST/... form data.
///
/// - If the content type of the request data is not
/// `application/x-www-form-urlencoded`, `Forward`s the request.
/// - If the form string is malformed, a `Failure` with status code 
/// `BadRequest` is returned. 
/// - Finally, if reading the incoming stream fails, returns a `Failure` with status code
/// `InternalServerError`.
/// In all failure cases, the raw form string is returned if it was able to be retrieved from the incoming stream.
///
/// All relevant warnings and errors are written to the console
impl<'f> FromData for FormHashMap<'f> {
    type Error = String;

    fn from_data(request: &Request, data: Data) -> rocket::data::Outcome<Self, Self::Error> {
        // TODO add support for application/json
        if !request.content_type().map_or(false, |ct| ct.is_form()) {
            error!("::AMIWO::CONTRIB::ROCKET::FORM_HASMAP::FROM_DATA::WARNING Form data does not have form content type.");
            return rocket::Outcome::Forward(data);
        }

        let size_limit = rocket::config::active()
            .and_then(|c| c.extras.get("limits.application")) // TODO: remove placeholder when upgrading to rocket version > 0.2.6
            // .and_then(|c| c.limits.get("application") // In next version
            .and_then(|limit| limit.as_integer())
            .unwrap_or(32768) as u64;

        let mut buffer = String::new();
        data.open()
            .take(size_limit)
            .read_to_string(&mut buffer)
            .or_else(|err| Err(format!("::AMIWO::CONTRIB::ROCKET::FORM_HASMAP::FROM_DATA::ERROR IO Error: {}", err.description())) )
            .and_then(|_| FormHashMap::new(buffer)) // Note: if ok, read_to_string() returns how many bytes where read 
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
    type Error = (Status, Option<String>);

    fn from_form_items(items: &mut FormItems<'f>) -> Result<Self, Self::Error> {
        FormHashMap::new(items.inner_str().to_string())
            .map(|map| {
                info!("::AMIWO::CONTRIB::ROCKET::FORM_HASMAP::FROM_FORM_ITEMS::INFO Successfully parsed input data => {:?}", map);
                map
            }).map_err(|invalid_string| {
                error!("::AMIWO::CONTRIB::ROCKET::FORM_HASMAP::FROM_FORM_ITEMS::ERROR The request's form string '{}' was malformed.", invalid_string);
                (Status::BadRequest, Some(invalid_string))
            })
    }
}

/// Implement Debug displaying '<internal data holding structure>' from string: <parsed string>'
impl<'f> Debug for FormHashMap<'f> {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(f, "{:?} from string: {:?}", self.form_hash, self.form_string)
    }
}

// =======================================================================
// TESTS
// =======================================================================
#[cfg(test)]
mod tests {
    #![allow(unmounted_route)]
    #![allow(non_snake_case)]

    use super::FormHashMap;
    use types::OneOrMany;

    use rocket;
    use rocket::testing::MockRequest;
    use rocket::http::{ ContentType, Method, Status };

    #[test]
    fn FormHashMap_test_new() {
        let form_string = "a=b1&a=b2&b=c";

        match FormHashMap::new(form_string.to_string()) {
            Ok(map) => {
                assert_eq!(map.get("a"), Some(&OneOrMany::Many(vec!["b1".to_string(), "b2".to_string()])));
                assert_eq!(map.get("b"), Some(&OneOrMany::One("c".to_string())));
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
            assert_eq!(params.get("a"), Some(&OneOrMany::Many(vec!["b1".to_string(), "b2".to_string()])));
            assert_eq!(params.get("b"), Some(&OneOrMany::One("c".to_string())));
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
            assert_eq!(params.get("a"), Some(&OneOrMany::Many(vec!["b1".to_string(), "b2".to_string()])));
            assert_eq!(params.get("b"), Some(&OneOrMany::One("c".to_string())));
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
            assert_eq!(params.get("v"), Some(&OneOrMany::One("4.7.0".to_string())));
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