// =======================================================================
// LIBRARY IMPORTS
// =======================================================================
use serde_json;

use std::borrow::Borrow;
use std::hash::Hash;

// =======================================================================
// TRAIT DECLARATION
// =======================================================================
/// Adds a `contains_keys()` method to any Map like object
pub trait ContainsKeys<K> {
    fn contains_keys<Q: ?Sized>(&self, keys: &[&Q]) -> bool 
        where
            K: Borrow<Q>,
            Q: Hash + Eq + Ord;
}

// =======================================================================
// TRAIT IMPLEMENTATION
// =======================================================================
impl ContainsKeys<String> for serde_json::Value {
    fn contains_keys<Q: ?Sized>(&self, keys: &[&Q]) -> bool 
        where
            String: Borrow<Q>,
            Q: Hash + Eq + Ord
    {
        self.as_object().map_or(
            false, 
            |obj| keys.iter().all(|ref key| obj.contains_key(key))
        )
    }
}

impl ContainsKeys<String> for serde_json::map::Map<String, serde_json::Value> {
    fn contains_keys<Q: ?Sized>(&self, keys: &[&Q]) -> bool 
        where
            String: Borrow<Q>,
            Q: Hash + Eq + Ord
    {
        keys.iter().all(|ref key| self.contains_key(key))
    }
}

// =======================================================================
// UNIT TESTS
// =======================================================================
#[cfg(test)]
mod tests {
    use serde_json;
    use super::ContainsKeys;

    #[test]
    fn contains_keys() {
        let obj: serde_json::Value = serde_json::from_str(r#"{"a": 1, "b": 2}"#).unwrap();
        assert_eq!(obj.contains_keys(&["a", "b"]), true); 
        assert_eq!(obj.contains_keys(&["a", "b", "c"]), false);
    }
}