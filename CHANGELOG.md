# Version 0.2.0 ()
- Added `GenericError` and changed all method signature to return a `GenericError`
- Added `Pushable` trait and associated implementations for `serde_json::Value`
- Refactored `FormHashMap` using 'serde_json::Value' instead of HashMap => now returns 'Value' instead of 'OneOrMany' types. FormHashMap can now parse 'application/json' data
- Added utility function to `contrib/hyper` to create request and parse Response into ResponseJSON
- Updated for 2017-05-26 nightly.

# Version 0.1.0 (May 19, 2017)
- Created `amiwo::contrib` to hold contribution to other crates (currently rocket); moved `FormHashMap` to that module
- Added `contrib::rocket::ResponseJSON` that can be used as a parameter and response with Rocket

# Version 0.0.1 (May 11, 2017)
- Creation
- Added `types::OneOrMany` enum wraping a single value or vector of value; added `types::rocket::FormHashMap` type deriving [FormData's](https://api.rocket.rs/rocket/data/trait.FromData.html) & [FromForm's](https://api.rocket.rs/rocket/request/trait.FromForm.html) [Rocket](https://rocket.rs) types