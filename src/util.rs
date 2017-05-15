// =======================================================================
// LIBRARY IMPORTS
// =======================================================================
use serde_json;

use std::borrow::Borrow;
use std::hash::Hash;
use std::iter::IntoIterator;

// =======================================================================
// TRAIT DECLARATION
// =======================================================================
pub trait AmiwoUtil {
}

/*
pub trait ContainKeys {
    fn contains_keys<K>(&self, keys: &[K]) -> bool 
        where K: Into<String> + Hash + Ord + Eq;
}
*/

// =======================================================================
// FUNCTIONS DECLARATION
// =======================================================================
// Note: You can create a function that accepts both &[String] and &[&str] using the AsRef trait: see http://stackoverflow.com/questions/41179659/convert-vector-of-string-into-slice-of-str-in-rust
pub fn contains_keys<K>(value: &serde_json::Value, keys: &[K]) -> bool 
    where K: AsRef<str>
{
    value.as_object().map_or(
        false, 
        |obj| keys.iter().all(|ref key| obj.contains_key(key.as_ref()))
    )
}

/*
impl ContainKeys for serde_json::Value {
    fn contains_keys<K>(&self, keys: &[K]) -> bool 
    where
        K: Into<String> + Hash + Ord + Eq,
    {
    self.as_object().map_or(
        false, 
        |obj| keys.iter().all(|ref key| obj.contains_key(key))
    )
    }
}
*/
// =======================================================================
// UNIT TESTS
// =======================================================================
#[cfg(test)]
mod tests {
    use serde_json;

    #[test]
    fn has_properties() {
        let obj: serde_json::Value = serde_json::from_str(r#"{"a": 1, "b": 2}"#).unwrap();
        assert_eq!(super::contains_keys(&obj, &["a", "b"]), true); 
        assert_eq!(super::contains_keys(&obj, &["a", "b", "c"]), false); 
    }
}