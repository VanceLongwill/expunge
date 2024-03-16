# Expunge

A crate for expunging/redacting and transforming sensitive fields.

[crates.io](https://crates.io/crates/expunge)

## Basic usage

```rust
 use expunge::Expunge;
 use serde::{Serialize, Deserialize};

 #[derive(Clone, Debug, Serialize, Deserialize, Expunge)]
 struct User {
   id: i64, // fields without #[expunge] annotations are left as is
   #[expunge(as = "Randy".to_string())]
   first_name: String,
   #[expunge(as = "Lahey".to_string())]
   last_name: String,
   #[expunge(with = sha256::digest)]
   date_of_birth: String,
   #[expunge]
   latitude: f64,
   #[expunge]
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

 assert_eq!(
   r#"{
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

| Attribute    | Description                                                                                                                                             | Feature   |
| ---          | ---                                                                                                                                                     | ---       |
| `as`         | provide a value that this field should be set to when expunged. e.g. `Default::default()` or `"<expunged>".to_string()`                                 | -         |
| `with`       | provide a function that will be called when expunging this value. It must return the same type as it takes. e.g. hash a `String` with `sha256::digest`. | -         |
| `all`        | can be used instead of specifying `#[expunge]` on every field/variant in a struct or enum                                                               | -         |
| `ignore`     | can be used to skip fields in combination with `all`                                                                                                    | -         |
| `zeroize`    | zeroize memory for extra security via the [secrecy](https://crates.io/crates/secrecy) & [zeroize](https://crates.io/crates/zeroize) crates              | `zeroize` |
| `slog`       | integrates with [slog](https://crates.io/crates/slog) using [slog-derive](https://crates.io/crates/slog_derive) to automatically expunge fields in logs | `slog`    |

### Logging with `slog`

Expunge provides a painless and foolproof way to log structs that may contain sensitive fields. 
As long as your type implements `serde::Serialize`, the `slog` attribute will derive `slog::SerdeValue`.
Internally the value will be expunged before logging.

#### Example

```rust
#[derive(Debug, Clone, Expunge, Deserialize, Serialize, PartialEq, Eq)] // must implement Serialize
#[expunge(slog)]
#[serde(rename_all = "snake_case")]
enum LocationType {
    #[expunge(as = "<expunged>".to_string())]
    City(String),
    #[expunge]
    Address {
        #[expunge(as = "line1".to_string())]
        line1: String,
        #[expunge(as = "line2".to_string())]
        line2: String,
    },
}

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

#### Toggling expunge at runtime

> Requires feature `slog_debug`

When working locally or in non-production environments, it may be useful to see the full values without expunging them.

Expunge provides a way to do this **only when using slog**.

```rust
use expunge::slog_debug::DisabledGuard;

// While _guard is not dropped, slog values will NOT be expunged.
// dropping will revert back to the previous guard (or back to expunging if no previous is set).
let _guard = DisabledGuard::new(true);

info!(logger, "it should log address"; "location" => address);
// {"msg":"it should log city without expunging","location":{"city":"New York"},"level":"INFO","ts":"2024-02-04T12:55:28.627592Z"}

// The most recent guard takes precedence and can be used to revert back to expunging.
// For example, you may want to override a global guard that is not available in the current scope.
let _guard2 = DisabledGuard::new(false);
// {"msg":"it should log city","location":{"city":"<expunged>"},"level":"INFO","ts":"2024-02-04T12:55:28.627592Z"}
```

## About

Other crates offer similar functionality, but either require types to be changed or 
make it difficult for both the expunged and unexpunged data being used at runtime.


This crate provides a proc_macro that implements the `Expunge` trait for the given type. 
Fields annotated with `#[expunge]` are cleared when the `expunge()` method is called, 
yielding back exactly the same type.

Since the same type is returned, introducing this crate should be completely frictionless. 
This comes with the tradeoff that the user is now responsible for ensuring that `expunge()` 
has been called when necessary. To make this more foolproof, this crate includes a type guard `Expunged<T>` 
that can only contain a expunged `T`. Internally constructing `Expunged<T>` calls `expunge()`, 
so it cannot be initialized with unexpunged data.

## Similar crates

- [secrecy](https://crates.io/crates/secrecy): Prevents secrets being logged/serialized by wrapping them in a `Secret<T>` type
- [veil](https://crates.io/crates/veil): A proc_macro similar to this crate to implement expunged `std::fmt::Debug` and/or `std::fmt::Display`
- [redact](https://crates.io/crates/redact): Similar to [secrecy](https://docs.rs/secrecy/latest/secrecy/), but without the memory zeroizing
- [redacted](https://crates.io/crates/redacted): Wrappers to control debug formatting of potentially sensitive byte arrays 


### Comparison

| crate                                         | proc_macro         | implements Display/Debug | serde support      | toggle on/off at runtime | uses original types | slog support       |
| --                                            | -                  | -                        | -                  | -                        | -                   | -                  |
| [secrecy](https://crates.io/crates/secrecy)   | :x:                | :white_check_mark:       | :white_check_mark: | :x:                      | :x:                 | :x:                |
| [redact](https://crates.io/crates/redact)     | :x:                | :white_check_mark:       | :white_check_mark: | :x:                      | :x:                 | :x:                |
| [veil](https://crates.io/crates/veil)         | :white_check_mark: | :white_check_mark:       | :x:                | :x:                      | :x:                 | :x:                |
| [redacted](https://crates.io/crates/redacted) | :x:                | :white_check_mark:       | :x:                | :x:                      | :x:                 | :x:                |
| [expunge](#Expunge)                           | :white_check_mark: | :x:                      | :white_check_mark: | :white_check_mark:       | :white_check_mark:  | :white_check_mark: |


## Contributing

- Ensure that all tests are passing 
   ```sh
   cargo test --all-features
   ```
- Open a PR/issue
