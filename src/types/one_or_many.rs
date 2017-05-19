//! File holding the OneOrMany type and associated tests
//!
//! Author: [Boris](mailto:boris@humanenginuity.com)
//! Version: 1.0
//!
//! ## Release notes
//! - v1.0 : creation

// =======================================================================
// LIBRARY IMPORTS
// =======================================================================
use std::ops::{ Index, IndexMut };

use serde::{ Serialize, Serializer };

// =======================================================================
// STRUCT & TRAIT DEFINITION
// =======================================================================
/// Type to encapsulate 'one or many' values
#[derive(Debug, PartialEq)] 
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

// =======================================================================
// STRUCT & TRAIT IMPLEMENTATION
// =======================================================================
impl<T> OneOrMany<T> {
    /// Return a reference to value (if is OneOrMany::One) or the first value of the vector (if is OneOrMany::Many)
    pub fn value<'v>(&'v self) -> Option<&'v T> {
        match *self {
            OneOrMany::One(ref val) => Some(val),
            OneOrMany::Many(ref vect) => vect.get(0),
        }
    }
    
    /// Return a mutable value (if is OneOrMany::One) or the first value of the vector (if is OneOrMany::Many)
    pub fn value_mut<'v>(&'v mut self) -> Option<&'v mut T> {
        match *self {
            OneOrMany::One(ref mut val) => Some(val),
            OneOrMany::Many(ref mut vect) => vect.get_mut(0),
        }
    }
    
    /// Consume `self` and return the value (if is OneOrMany::One) or the first value of the vector (if is OneOrMany::Many)
    pub fn into_value(self) -> Option<T> {
        match self {
            OneOrMany::One(val) => Some(val),
            OneOrMany::Many(mut vect) => {
                if vect.len() > 0 {
                    Some(vect.remove(0))
                } else {
                    None
                }
            }
        }
    }
    
    /// Consume `self` and return a vector containing all the values from `self` (if OneOrMany::Many) or one value (if OneOrMany::One)
    pub fn into_values(self) -> Vec<T> {
        match self {
            OneOrMany::One(val) => vec![val],
            OneOrMany::Many(vect) => vect,
        }
    }

    /// Returns `true` if `self` is OneOrMany::One
    pub fn is_one(&self) -> bool {
        match *self {
            OneOrMany::One(_) => true,
            _ => false,
        }
    }

    /// Returns `true` if `self` is OneOrMany::Many
    pub fn is_many(&self) -> bool {
        match *self {
            OneOrMany::Many(_) => true,
            _ => false,
        }
    }
}

/// Access an element of this type. Panics if the index is out of .
impl<T> Index<usize> for OneOrMany<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        match *self {
            OneOrMany::One(ref val) => {
                if index != 0 {
                    panic!("index out of bounds: only 'One' value but the index is {}", index);
                }
                val
            },
            OneOrMany::Many(ref vect) => &vect[index],
        }
    }
}

/// Access an element of this type in a mutable context. Panics if the index is out of .
impl<T> IndexMut<usize> for OneOrMany<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        match *self {
            OneOrMany::One(ref mut val) => {
                if index != 0 {
                    panic!("index out of bounds: only 'One' value but the index is {}", index);
                }
                val
            },
            OneOrMany::Many(ref mut vect) => &mut vect[index],
        }
    }
}

/// Implement IntoIterator for OneOrMany
impl<T> IntoIterator for OneOrMany<T> {
    type Item = T;
    type IntoIter = ::std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_values().into_iter()
    }
}

impl<T> Serialize for OneOrMany<T>
    where T: Serialize
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match *self {
            OneOrMany::One(ref val) => serializer.serialize_newtype_variant("OneOrMany", 0, "One", val),
            OneOrMany::Many(ref vec) => serializer.serialize_newtype_variant("OneOrMany", 1, "Many", vec),
        }
    }
}

/// Allow to compare a Vector with an instance from OneOrMany
impl <T: PartialEq<U>, U> PartialEq<Vec<U>> for OneOrMany<T> {
    fn eq(&self, other: &Vec<U>) -> bool {
        match *self {
            OneOrMany::One(ref val) => other.len() == 1 && *val == other[0],
            OneOrMany::Many(ref vect) => {
                let mut index = 0;
                while index < vect.len() {
                    if self[index] != other[index] { return false; };
                    index += 1;
                }
                true
            }
        }
    }
}

impl <T: PartialEq<U>, U> PartialEq<::std::vec::IntoIter<U>> for OneOrMany<T> {
    fn eq(&self, other: &::std::vec::IntoIter<U>) -> bool {
        let other_slice = other.as_slice();
        match *self {
            OneOrMany::One(ref val) => other_slice.len() == 1 && *val == other_slice[0],
            OneOrMany::Many(ref vect) => {
                let mut index = 0;
                while index < vect.len() {
                    if self[index] != other_slice[index] { return false; };
                    index += 1;
                }
                true
            }
        }
    }
}

impl <'a, T> PartialEq<OneOrMany<T>> for Vec<&'a str> 
    where T: PartialEq<&'a str>
{
    fn eq(&self, other: &OneOrMany<T>) -> bool {
        match *other {
            OneOrMany::One(ref val) => self.len() == 1 && val == &self[0],
            OneOrMany::Many(ref vect) => vect == self
        }
    }
}

macro_rules! __impl_partial_eq {
    (Vec < $($args:ty),* $(,)* >) => {
        impl <T> PartialEq<OneOrMany<T>> for Vec<$($args),*> 
            where 
                Vec<T>: PartialEq<Vec<$($args),*>>,
                T: PartialEq<$($args),*>
        {
            fn eq(&self, other: &OneOrMany<T>) -> bool {
                match *other {
                    OneOrMany::One(ref val) => self.len() == 1 && *val == self[0],
                    OneOrMany::Many(ref vect) => vect == self
                }
            }
        }
    }
}

__impl_partial_eq!(Vec<i8>);
__impl_partial_eq!(Vec<i16>);
__impl_partial_eq!(Vec<i32>);
__impl_partial_eq!(Vec<i64>);
__impl_partial_eq!(Vec<u8>);
__impl_partial_eq!(Vec<u16>);
__impl_partial_eq!(Vec<u32>);
__impl_partial_eq!(Vec<u64>);
__impl_partial_eq!(Vec<isize>);
__impl_partial_eq!(Vec<usize>);
__impl_partial_eq!(Vec<f32>);
__impl_partial_eq!(Vec<f64>);
__impl_partial_eq!(Vec<String>);

// =======================================================================
// UNIT TESTS
// =======================================================================
#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::OneOrMany;

    #[test]
    fn OneOrMany_test_one() {
        let x = OneOrMany::One(17);
        assert_eq!(x.is_one(), true);
        assert_eq!(x.is_many(), false);
        assert_eq!(x.value().unwrap(), &17);
        assert_eq!(x[0], 17);
        assert_eq!(x.into_value().unwrap(), 17);

        let mut x = OneOrMany::One(18);
        assert_eq!(x.value_mut().unwrap(), &18);
        assert_eq!(x.into_value().unwrap(), 18);
    }

    #[test]
    fn OneOrMany_test_one_mut() {
        let mut x = OneOrMany::One(17);
        if let Some(y) = x.value_mut() {
            *y = 18;
        }
        assert_eq!(x.value().unwrap(), &18);
        x[0] = 19;
        assert_eq!(x.value().unwrap(), &19);
    }

    #[test]
    fn OneOrMany_test_many() {
        let x = OneOrMany::Many(vec![1, 2, 3]);
        assert_eq!(x.is_one(), false);
        assert_eq!(x.is_many(), true);
        assert_eq!(x.value().unwrap(), &1);
        assert_eq!(x[0], 1);
        assert_eq!(x[1], 2);
        assert_eq!(x[2], 3);
        assert_eq!(x.into_value().unwrap(), 1);

        let mut x = OneOrMany::Many(vec![11, 22, 33]);
        assert_eq!(x.value_mut().unwrap(), &11);
        assert_eq!(x.into_value().unwrap(), 11);
    }

    #[test]
    fn OneOrMany_test_many_mut() {
        let mut x = OneOrMany::Many(vec![1, 2, 3]);
        if let Some(y) = x.value_mut() {
            *y = 18;
        }

        assert_eq!(x.value().unwrap(), &18);
        assert_eq!(x.into_values(), vec![18, 2, 3]);

        let mut x = OneOrMany::Many(vec![1, 2, 3]);
        x[0] = 19;
        x[1] = 20;
        x[2] = 21;
        assert_eq!(x.into_values(), vec![19, 20, 21]);
    }

    #[test]
    #[should_panic(expected = "index out of bounds: only 'One' value but the index is")]
    fn OneOrMany_test_one_index_oob() {
        let x = OneOrMany::One(17);
        x[1];
    }

    #[test]
    #[should_panic(expected = "index out of bounds: the len is 3 but the index is")]
    fn OneOrMany_test_many_index_oob() {
        let x = OneOrMany::Many(vec![1, 2, 3]);
        x[4];
    }

    #[test]
    fn OneOrMany_test_one_into_iter() {
        let mut x = OneOrMany::One(17).into_iter();
        assert_eq!(x.next(), Some(17));
        assert_eq!(x.next(), None);
    }

    #[test]
    fn OneOrMany_test_many_into_iter() {
        let mut x = OneOrMany::Many(vec![1, 2, 3]).into_iter();
        assert_eq!(x.next(), Some(1));
        assert_eq!(x.next(), Some(2));
        assert_eq!(x.next(), Some(3));
        assert_eq!(x.next(), None);
    }

    #[test]
    fn OneOrMany_test_eq() {
        let ox = OneOrMany::One(17);
        let oy = OneOrMany::One(18);

        let mx = OneOrMany::Many(vec![1, 2, 3]);
        let my = OneOrMany::Many(vec![11, 22, 33]);

        assert_eq!(ox == ox, true);
        assert_eq!(mx == mx, true);

        assert_eq!(ox == oy, false);
        assert_eq!(oy == ox, false);
        assert_eq!(ox == mx, false);
        assert_eq!(mx == ox, false);
        assert_eq!(ox == my, false);
        assert_eq!(my == ox, false);
    }

    #[test]
    fn OneOrMany_test_eq_vect() {
        let x = OneOrMany::One(17);
        assert_eq!(x, vec![17]);
        assert_eq!(vec![17], x);

        let x = OneOrMany::Many(vec![1, 2, 3]);
        assert_eq!(x, vec![1, 2, 3]);
        assert_eq!(vec![1, 2, 3], x);

        assert_eq!(OneOrMany::One("test String".to_string()), vec!["test String".to_string()]);
        assert_eq!(vec!["test String".to_string()], OneOrMany::One("test String".to_string()));

        assert_eq!(OneOrMany::One("test String".to_string()), vec!["test String"]);
        assert_eq!(vec!["test String"], OneOrMany::One("test String".to_string()));

        assert_eq!(OneOrMany::One("test &str"), vec!["test &str"]);
        assert_eq!(vec!["test &str"], OneOrMany::One("test &str"));
    }
}