// =======================================================================
// ATTRIBUTES
// =======================================================================
#![allow(dead_code)]

// =======================================================================
// LIBRARY IMPORTS
// =======================================================================
use std::fmt::Debug;
use std::marker::PhantomData;
use std::io::Read;

use rocket;
use rocket::{ Request, Data };
use rocket::data::FromData;
use rocket::http::{ ContentType, Status };
use rocket::request::{ FromForm, FromFormValue, FormItems };

use std::collections::HashMap;

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

enum FormResult<T> {
    Ok(T),
    Invalid(String)
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
    fn new(form_string: String) -> FormResult<Self> {
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
            error!("::MODEL::ROCKET::FORM_HASMAP::NEW::WARNING Form string {} couldn't be successfully parsed", form_string);
            return FormResult::Invalid(form_string);
        }
        if !items.completed() {
            warn!("::MODEL::ROCKET::FORM_HASMAP::NEW::WARNING Form string {} couldn't be completely parsed", form_string);
            warn!("Items = {:?}", items.collect::<Vec<_>>());
        }

        FormResult::Ok(FormHashMap {
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
/// `application/x-www-form-urlencoded` or `application/json`, `Forward`s the request.
/// - If the form string is malformed, a `Failure` with status code 
/// `BadRequest` is returned. 
/// - Finally, if reading the incoming stream fails, returns a `Failure` with status code
/// `InternalServerError`.
/// In all failure cases, the raw form string is returned if it was able to be retrieved from the incoming stream.
///
/// All relevant warnings and errors are written to the console
impl<'f> FromData for FormHashMap<'f> {
    type Error = Option<String>;

    fn from_data(request: &Request, data: Data) -> rocket::data::Outcome<Self, Self::Error> {
        // TODO add support for application/json
        let form_content_type = ContentType::new("application", "x-www-form-urlencoded");
        if request.content_type() != Some(form_content_type) {
            warn!("::MODEL::ROCKET::FORM_HASMAP::FROM_DATA::WARNING Form data does not have form content type.");
            return rocket::Outcome::Forward(data);
        }

        let mut form_string = String::new();
        let mut stream = data.open().take(32768); 
        if let Err(e) = stream.read_to_string(&mut form_string) {
            error!("::MODEL::ROCKET::FORM_HASMAP::FROM_DATA::ERROR IO Error: {:?}", e);
            rocket::Outcome::Failure((Status::InternalServerError, None))
        } else {
            match FormHashMap::new(form_string) {
                FormResult::Ok(map) => {
                    info!("::MODEL::ROCKET::FORM_HASMAP::FROM_DATA::INFO Successfully parsed input data into {:?}.", map);
                    rocket::Outcome::Success(map)
                },
                FormResult::Invalid(invalid_string) => {
                    error!("::MODEL::ROCKET::FORM_HASMAP::FROM_DATA::ERROR The request's form string '{}' was malformed.", invalid_string);
                    rocket::Outcome::Failure((Status::BadRequest, Some(invalid_string)))
                },
            }
        }
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
        let form_string = items.inner_str().to_string();
        match FormHashMap::new(form_string) {
            FormResult::Ok(map) => {
                info!("::MODEL::ROCKET::FORM_HASMAP::FROM_FORM_ITEMS::INFO Successfully parsed input data => {:?}", map);
                Ok(map)
            },
            FormResult::Invalid(invalid_string) => {
                error!("::MODEL::ROCKET::FORM_HASMAP::FROM_FORM_ITEMS::ERROR The request's form string '{}' was malformed.", invalid_string);
                Err((Status::BadRequest, Some(invalid_string)))
            },
        }
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
mod test {
    #![allow(unmounted_route)]

    use super::FormHashMap;
    use super::FormResult;
    use types::OneOrMany;

    use rocket;
    use rocket::testing::MockRequest;
    use rocket::http::{ ContentType, Method, Status };

    #[test]
    fn test_new() {
        let form_string = "a=b1&a=b2&b=c";

        match FormHashMap::new(form_string.to_string()) {
            FormResult::Ok(map) => {
                println!("Map = {:?}", map);
                println!("Map[{}] = {:?}", "a", map.get("a"));
                assert_eq!(map.get("a"), Some(&OneOrMany::Many(vec!["b1".to_string(), "b2".to_string()])));
                assert_eq!(map.get("b"), Some(&OneOrMany::One("c".to_string())));
            },
            FormResult::Invalid(invalid_string) => {
                panic!("Unable to parse {}", invalid_string);
            }
        }
    }

    #[test]
    fn test_post_route() {
        #[post("/test", data= "<params>")]
        fn test_route(params: FormHashMap) -> &'static str {
            println!("Map = {:?}", params);
            println!("Map[{}] = {:?}", "a", params.get("a"));
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
    fn test_get_route() {
        #[get("/test?<params>")]
        fn test_route(params: FormHashMap) -> &'static str {
            println!("Map = {:?}", params);
            println!("Map[{}] = {:?}", "a", params.get("a"));
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
    fn test_get_qs_with_dot() {
        #[get("/test?<params>")]
        fn test_route(params: FormHashMap) -> &'static str {
            println!("Map = {:?}", params);
            println!("Map[{}] = {:?}", "v", params.get("v"));
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

    // TODO: test json
    /*
    /// let req = MockRequest::new(Post, "/")
    ///     .header(ContentType::JSON)
    ///     .body(r#"{ "key": "value", "array": [1, 2, 3], }"#);
    */

    // TODO: add test lifetime
}