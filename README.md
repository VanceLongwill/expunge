# Expunge

A crate for expunging/redacting and transforming sensitive fields.

[crates.io](https://crates.io/crates/expunge)

## Basic usage

```rust
use expunge::Expunge;
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Expunge)]
struct User {
  #[expunge(skip)] // skipped fields are not transformed
  id: i64,
  #[expunge(as = "Randy".to_string())]
  first_name: String,
  #[expunge(as = "Lahey".to_string())]
  last_name: String,
  #[expunge(with = sha256::digest)]
  date_of_birth: String,
  latitude: f64,
  longitude: f64,
  #[expunge(as = "<expunged>".to_string(), zeroize)]
  password_hash: String,
}

let user = User{
  id: 101,
  first_name: "Ricky".to_string(),
  last_name: "LaFleur".to_string(),
  date_of_birth: "02/02/1960".to_string(),
  latitude: 45.0778,
  longitude: 63.546,
  password_hash: "2f089e52def4cec8b911883fecdd6d8febe9c9f362d15e3e33feb2c12f07ccc1".to_string(),
};

let expunged_user = user.expunge();

let output = serde_json::to_string_pretty(&expunged_user).expect("should serialize");

assert_eq!(r#"{
  "id": 101,
  "first_name": "Randy",
  "last_name": "Lahey",
  "date_of_birth": "eeb98c815ae11240b563892c52c8735472bb8259e9a6477e179a9ea26e7a695a",
  "latitude": 0.0,
  "longitude": 0.0,
  "password_hash": "<expunged>"
}"#,
  output,
)
```

#### Attributes

##### Container attributes

> attributes that apply to a struct or enum declaration

- 

| Attribute     | Description                                                                                                                                             | Feature   |
| ---           | ---                                                                                                                                                     | ---       |
| `as`          | provide a value that all the fields should be set to when expunged. e.g. `Default::default()` or `"<expunged>".to_string()`                             | -         |
| `with`        | provide a function that will be called when expunging this value. It must return the same type as it takes. e.g. hash a `String` with `sha256::digest`. | -         |
| `skip`        | can be used to skip fields that shouldn't be expunged                                                                                                   | -         |
| `allow_debug` | allows the user to provide their own `Debug` implementation                                                                                             | -         |
| `default`     | shorthand equivalent to `as = Default::default()`                                                                                                       | -         |
| `zeroize`     | zeroize memory for extra security via the [secrecy](https://crates.io/crates/secrecy) & [zeroize](https://crates.io/crates/zeroize) crates              | `zeroize` |
| `slog`        | integrates with [slog](https://crates.io/crates/slog) using [slog-derive](https://crates.io/crates/slog_derive) to automatically expunge fields in logs | `slog`    |

##### Field & variant attributes
 
> attributes that can be applied to a struct field, enum variant or field in an enum variant


| Attribute     | Description                                                                                                                                             | Feature   |
| ---           | ---                                                                                                                                                     | ---       |
| `as`          | provide a value that this field should be set to when expunged. e.g. `Default::default()` or `"<expunged>".to_string()`                                 | -         |
| `with`        | provide a function that will be called when expunging this value. It must return the same type as it takes. e.g. hash a `String` with `sha256::digest`. | -         |
| `skip`        | can be used to skip fields that shouldn't be expunged                                                                                                   | -         |
| `allow_debug` | allows the user to provide their own `Debug` implementation                                                                                             | -         |
| `default`     | shorthand equivalent to `as = Default::default()`                                                                                                       | -         |
| `zeroize`     | zeroize memory for extra security via the [secrecy](https://crates.io/crates/secrecy) & [zeroize](https://crates.io/crates/zeroize) crates              | `zeroize` |
| `slog`        | integrates with [slog](https://crates.io/crates/slog) using [slog-derive](https://crates.io/crates/slog_derive) to automatically expunge fields in logs | `slog`    |

### Logging with `slog`

Expunge provides a painless and foolproof way to log structs that may contain sensitive fields. 
As long as your type implements `serde::Serialize`, the `slog` attribute will derive `slog::SerdeValue`.
Internally the value will be expunged before logging.

#### Example

```rust
use expunge::Expunge;
use serde::{Serialize, Deserialize};
use slog::{info, o, Drain, Logger};
use std::sync::Mutex;

#[derive(Clone, Expunge, Deserialize, Serialize, PartialEq, Eq)] // must implement Serialize
#[expunge(slog)]
#[serde(rename_all = "snake_case")]
enum LocationType {
    #[expunge(as = "<expunged>".to_string())]
    City(String),
    Address {
        #[expunge(as = "line1".to_string())]
        line1: String,
        #[expunge(as = "line2".to_string())]
        line2: String,
    },
}
 
# let mut buf = vec![];
# let drain = Mutex::new(slog_json::Json::default(buf)).fuse();
# let logger = Logger::root(drain, o!());

// Just log as is and it will be automatically expunged

let city = LocationType::City("New York".to_string());
info!(logger, "it should log city"; "location" => city);

let address = LocationType::Address{
    line1: "101 Some street".to_string(),
    line2: "Some Town".to_string(),
};
info!(logger, "it should log address"; "location" => address);

// {"msg":"it should log city","location":{"city":"<expunged>"},"level":"INFO","ts":"2024-02-04T12:55:28.627592Z"}
// {"msg":"it should log address","location":{"address":{"line1":"line1","line2":"line2"}},"level":"INFO","ts":"2024-02-04T12:55:28.627627Z"}
```


## About

Other crates offer similar functionality, but either require types to be changed or 
make it difficult for both the expunged and unexpunged data being used at runtime.

This crate provides a proc_macro that derives the `Expunge` trait for the given type. 
When the `Expunge::expunge` method is called, sensitive fields are transformed/redacted.

- All fields are transformed unless annotated with `#[expunge(skip)]`
- The `Expunge` macro first looks for transformations declared on field/struct attributes i.e. `as` or `with`. 
  If these aren't set then `Expunge` macro will use the `Expunge::expunge` implementation for the type.
  A default implementation for the `Expunge` trait is provided for primitive types and common container types.
  These will be expunged as their default values, unless otherwise specified.
  
Since expunge doesn't require types to be changed, migrating to this crate should be completely frictionless.

This comes with the tradeoff that the user is now responsible for ensuring that `Expunge::expunge` 
has been called as appropriate, this crate includes a type guard `Expunged<T>` 
that can only contain a expunged `T`. Internally constructing `Expunged<T>` calls `Expunge::expunge`, 
so it cannot be initialized with unexpunged data. 

## Similar crates

- [secrecy](https://crates.io/crates/secrecy): Prevents secrets being logged/serialized by wrapping them in a `Secret<T>` type
- [veil](https://crates.io/crates/veil): A proc_macro similar to this crate to implement expunged `std::fmt::Debug` and/or `std::fmt::Display`
- [redact](https://crates.io/crates/redact): Similar to [secrecy](https://docs.rs/secrecy/latest/secrecy/), but without the memory zeroizing
- [redacted](https://crates.io/crates/redacted): Wrappers to control debug formatting of potentially sensitive byte arrays 


### Comparison

| crate                                         | proc_macro | implements Debug | serde support | toggle on/off at runtime | uses original types | slog support |
| ---                                           | ---        | ---              | ---           | ---                      | ---                 | ---          |
| [secrecy](https://crates.io/crates/secrecy)   | ✘          | ✔                | ✔             | ✘                        | ✘                   | ✘            |
| [redact](https://crates.io/crates/redact)     | ✘          | ✔                | ✔             | ✘                        | ✘                   | ✘            |
| [veil](https://crates.io/crates/veil)         | ✔          | ✔                | ✘             | ✘                        | ✘                   | ✘            |
| [redacted](https://crates.io/crates/redacted) | ✘          | ✔                | ✘             | ✘                        | ✘                   | ✘            |
| [expunge](#Expunge)                           | ✔          | ✔                | ✔             | ✔                        | ✔                   | ✔            |


## Contributing

- Ensure that all tests are passing 
   ```sh
   cargo test --all-features
   ```
- Open a PR/issue
