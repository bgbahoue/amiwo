# Version 0.1.0 (May 15, 2017)
- Created `amiwo::contrib` to hold contribution to other crates (currently rocket); moved `FormHashMap` to that module
- Added `contrib::rocket::ResponseJSON` that can be used as a parameter and response with Rocket

# Version 0.0.1 (May 11, 2017)
- Creation
- Added `types::OneOrMany` enum wraping a single value or vector of value; added `types::rocket::FormHashMap` type deriving [FormData's](https://api.rocket.rs/rocket/data/trait.FromData.html) & [FromForm's](https://api.rocket.rs/rocket/request/trait.FromForm.html) [Rocket](https://rocket.rs) types