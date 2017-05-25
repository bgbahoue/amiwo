extern crate amiwo;

use amiwo::ResponseJSON;
use amiwo::IsResponseJSON;

fn test_moved_value() {
    let x = ResponseJSON::ok();
    assert_eq!(x.to_string(), "{\"http_code\":200,\"success\":true}".to_string());

    let moved_string = x.into_string();
    assert_eq!(moved_string, "{\"http_code\":200,\"success\":true}".to_string());
    assert_eq!(x.is_valid_json(), false, "Shouldn't be able to execute this"); //~ ERROR use of moved value
}

fn main() {
    test_moved_value();
}