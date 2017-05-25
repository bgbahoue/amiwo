//! File holding the Pushable trait
//!
//! Author: [Boris](mailto:boris@humanenginuity.com)
//! Version: 1.0
//!
//! ## Release notes
//! - v1.0 : creation

// =======================================================================
// LIBRARY IMPORTS
// =======================================================================
use serde_json;
use serde_json::Value;

// =======================================================================
// TRAIT DEFINITION
// =======================================================================
pub trait Pushable<T> {
    fn push(&mut self, value: T) -> &mut Self;
}

// =======================================================================
// TRAIT IMPLEMENTATION
// =======================================================================
/// Implements `Pushable<T>` for `Vec<T>`
impl<T> Pushable<T> for Vec<T> {
    fn push(&mut self, new_value: T) -> &mut Self {
        self.push(new_value);
        self
    }    
}

/// Implements `Pushable` for `serde_json::Value`
///
/// - If `self` is anything but a `Value::Array` => transforms to an Array containing the existing value and appends the new value
/// - If `self` is a `Value::Array` => appends the new value 
impl Pushable<Value> for Value {
    fn push(&mut self, new_value: Value) -> &mut Self {
        let mut vect = Vec::new();

        match *self {
            Value::Array(ref mut existing_vect) => vect.append(existing_vect),
            Value::Null => (), // do nothing
            ref existing_value @ _ => vect.push(existing_value.clone()),
        }
        vect.push(new_value);
        ::std::mem::replace(self, Value::Array(vect));
        self
    }
}

/// Allow to push a String using `serde_json::from_str()`
/// It first try to use `serde_json::from_str()` on the String. If it fails, it pushes a new `Value::String()` instead
impl Pushable<String> for Value {
    fn push(&mut self, new_value: String) -> &mut Self {
        self.push(new_value.as_str())
    }
}

/// Allow to push a &str using `serde_json::from_str()`
/// It first try to use `serde_json::from_str()` on the String. If it fails, it pushes a new `Value::String()` instead
impl<'s> Pushable<&'s str> for Value {
    fn push(&mut self, new_value: &'s str) -> &mut Self {
        let value : Result<Value, _> = serde_json::from_str(new_value);

        if value.is_err() {
            self.push(Value::String(new_value.to_string()));
        } else {
            self.push(value.unwrap());
        }
        self
    }
}

/// Allow to push Result<Value, _> (sugar for pushing `serde_json::from_xxx()`)
/// Panics if conversion failed
impl Pushable<Result<Value, serde_json::Error>> for Value {
    fn push(&mut self, new_value: Result<Value, serde_json::Error>) -> &mut Self {
        if new_value.is_err() {
            panic!("::amiwo::pushable<Result<V, E>>::push::error unable to push invalid value {}", &new_value.unwrap_err());
        } else {
            self.push(new_value.unwrap());
        }
        self
    }
}

// =======================================================================
// UNIT TESTS
// =======================================================================
#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::Pushable;

    #[test]
    fn Pushable_test_value() {
        let mut x = json!("a");
        x.push("b");
        assert_eq!(x, json!(["a", "b"]));

        let mut x = json!(1);
        x.push("b");
        assert_eq!(x, json!([1, "b"]));

        let mut x = json!(true);
        x.push("b");
        assert_eq!(x, json!([true, "b"]));

        let mut x = json!(["a", "b"]);
        x.push("c");
        assert_eq!(x, json!(["a", "b", "c"]));

        let mut x = json!({
            "key1": "value1",
            "key2": "value2"
        });
        x.push("c");
        assert_eq!(x, json!([
            {
                "key1": "value1",
                "key2": "value2"
            }, 
            "c"
        ]));
    }
}