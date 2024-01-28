# Expunge

A crate for expungeing and transforming sensitive fields.

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
   #[expunge(as = "<expungeed>".to_string(), zeroize)]
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

 let expungeed_user = user.expunge();

 let output = serde_json::to_string_pretty(&expungeed_user).expect("should serialize");

 assert_eq!(
   r#"{
   "id": 101,
   "first_name": "Randy",
   "last_name": "Lahey",
   "date_of_birth": "eeb98c815ae11240b563892c52c8735472bb8259e9a6477e179a9ea26e7a695a",
   "latitude": 0.0,
   "longitude": 0.0,
   "password_hash": "<expungeed>"
}"#,
   output,
 )
```

| Attribute | Description                                                                                                                                             | Feature   |
| ---       | ---                                                                                                                                                     | ---       |
| `as`      | provide a value that this field should be set to when expungeed. e.g. `Default::default()` or `"<expungeed>".to_string()`                                 | -         |
| `with`    | provide a function that will be called when expungeing this value. It must return the same type as it takes. e.g. hash a `String` with `sha256::digest`. | -         |
| `all`     | can be used instead of specifying `#[expunge]` on every field/variant in a struct or enum                                                                | -         |
| `ignore`  | can be used to skip fields in combination with `all`                                                                                                    | -         |
| `zeroize` | zeroize memory for extra security via the [secrecy](https://crates.io/crates/secrecy) & [zeroize](https://crates.io/crates/zeroize) crates              | `zeroize` |

## About

Other crates offer similar functionality, but either require types to be changed or 
make it difficult for both the expungeed and unexpungeed data being used at runtime.

This crate provides a proc_macro that implements the `Expunge` trait for the given type. 
Fields annotated with `#[expunge]` are cleared when the `expunge()` method is called, 
yielding back exactly the same type.

Since the same type is returned, introducing this crate should be completely frictionless. 
This comes with the tradeoff that the user is now responsible for ensuring that `expunge()` 
has been called when necessary. To make this more foolproof, this crate includes a type guard `Expungeed<T>` 
that can only contain a expungeed `T`. Internally constructing `Expungeed<T>` calls `expunge()`, 
so it cannot be initialized with unexpungeed data.

## Similar crates

- [secrecy](https://crates.io/crates/secrecy): Prevents secrets being logged/serialized by wrapping them in a `Secret<T>` type
- [veil](https://crates.io/crates/veil): A proc_macro similar to this crate to implement expungeed `std::fmt::Debug` and/or `std::fmt::Display`
- [expunge](https://crates.io/crates/expunge): Similar to [secrecy](https://docs.rs/secrecy/latest/secrecy/), but without the memory zeroizing
- [expungeed](https://crates.io/crates/expungeed): Wrappers to control debug formatting of potentially sensitive byte arrays 


### Comparison

| crate                                         | proc_macro         | implements Display/Debug | serde support      | toggle on/off at runtime | uses original types |
| --                                            | -                  | -                        | -                  | -                        | -                   |
| [secrecy](https://crates.io/crates/secrecy)   | :x:                | :white_check_mark:       | :white_check_mark: | :x:                      | :x:                 |
| [expunge](https://crates.io/crates/expunge)     | :x:                | :white_check_mark:       | :white_check_mark: | :x:                      | :x:                 |
| [veil](https://crates.io/crates/veil)         | :white_check_mark: | :white_check_mark:       | :x:                | :x:                      | :x:                 |
| [expungeed](https://crates.io/crates/expungeed) | :x:                | :white_check_mark:       | :x:                | :x:                      | :x:                 |
| [expunge](#Expunge)                             | :white_check_mark: | :x:                      | :white_check_mark: | :white_check_mark:       | :white_check_mark:  |


## Contributing


