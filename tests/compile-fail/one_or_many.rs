extern crate amiwo;

use amiwo::types::OneOrMany;

fn test_moved_value_one() {
    let x = OneOrMany::One(17);
    assert_eq!(x.into_value().unwrap(), 17);
    assert_eq!(x.is_one(), true); //~ ERROR use of moved value

    let x = OneOrMany::One(17);
    assert_eq!(x.into_values(), vec![17]);
    assert_eq!(x.is_one(), true); //~ ERROR use of moved value
}

fn test_moved_value_many() {
    let x = OneOrMany::Many(vec![1, 2, 3]);
    assert_eq!(x.into_value().unwrap(), 1);
    assert_eq!(x.is_many(), true); //~ ERROR use of moved value

    let x = OneOrMany::Many(vec![1, 2, 3]);
    assert_eq!(x.into_values(), vec![1, 2, 3]);
    assert_eq!(x.is_many(), true); //~ ERROR use of moved value
}

fn main() {
    test_moved_value_one();
    test_moved_value_many();
}