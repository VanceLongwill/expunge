//! Redact provides a straightfoward macro-based approach for dealing with
//! sensitive values.
//!
//! Keeping track of which values are sensitive (e.g. PII, secrets) is as simple as
//! marking them with the `#[redact]` attribute. Then, when you need a sanitized copy of your data,
//! simply do `let sanitized = data.redact();`. If no other redact behaviour is specified (see `as`
//! & `with`), the field will be replaced with its default value.
//!
//! ### Usage
//!
//! ```rust
//! use redact::Redact;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Clone, Debug, Serialize, Deserialize, Redact)]
//! struct User {
//!   id: i64, // fields without #[redact] annotations are left as is
//!   #[redact(as = "Randy".to_string())]
//!   first_name: String,
//!   #[redact(as = "Lahey".to_string())]
//!   last_name: String,
//!   #[redact(with = sha256::digest)]
//!   date_of_birth: String,
//!   #[redact]
//!   latitude: f64,
//!   #[redact]
//!   longitude: f64,
//!   #[redact(as = "<redacted>".to_string(), zeroize)]
//!   password_hash: String,
//! }
//!
//! let user = User{
//!   id: 101,
//!   first_name: "Ricky".to_string(),
//!   last_name: "LaFleur".to_string(),
//!   date_of_birth: "02/02/1960".to_string(),
//!   latitude: 45.0778,
//!   longitude: 63.546,
//!   password_hash: "2f089e52def4cec8b911883fecdd6d8febe9c9f362d15e3e33feb2c12f07ccc1".to_string(),
//! };
//!
//! let redacted_user = user.redact();
//!
//! let output = serde_json::to_string_pretty(&redacted_user).expect("should serialize");
//!
//! assert_eq!(
//!   r#"{
//!   "id": 101,
//!   "first_name": "Randy",
//!   "last_name": "Lahey",
//!   "date_of_birth": "eeb98c815ae11240b563892c52c8735472bb8259e9a6477e179a9ea26e7a695a",
//!   "latitude": 0.0,
//!   "longitude": 0.0,
//!   "password_hash": "<redacted>"
//!}"#,
//!   output,
//! )
//!
//! ```
//!
//! ### Available attributes
//!
//! | Attribute | Description                                                                                                                                             | Feature   |
//! | ---       | ---                                                                                                                                                     | ---       |
//! | `as`      | provide a value that this field should be set to when redacted. e.g. `Default::default()` or `"<redacted>".to_string()`                                 | -         |
//! | `with`    | provide a function that will be called when redacting this value. It must return the same type as it takes. e.g. hash a `String` with `sha256::digest`. | -         |
//! | `all`     | can be used instead of specifying `#[redact]` on every field/variant in a struct or enum                                                                | -         |
//! | `ignore`  | can be used to skip fields in combination with `all`                                                                                                    | -         |
//! | `zeroize` | zeroize memory for extra security via the [secrecy](https://crates.io/crates/secrecy) & [zeroize](https://crates.io/crates/zeroize) crates              | `zeroize` |
//!
//!

use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
};

pub use redact_derive::*;

pub mod primitives;

#[cfg(feature = "zeroize")]
#[doc(hidden)]
pub use ::zeroize;

#[cfg(feature = "zeroize")]
use secrecy::Secret;
#[cfg(feature = "zeroize")]
use zeroize::{DefaultIsZeroes, Zeroize};

#[cfg(feature = "zeroize")]
#[doc(hidden)]
pub use ::secrecy;

#[cfg(feature = "serde")]
#[doc(hidden)]
pub use ::serde;

/// Trait for recursively redacting values marked as sensitive
pub trait Redact {
    fn redact(self) -> Self
    where
        Self: Sized;
}

impl<T> Redact for Option<T>
where
    T: Redact,
{
    fn redact(self) -> Self
    where
        Self: Sized,
    {
        self.map(Redact::redact)
    }
}

impl<R, E> Redact for Result<R, E>
where
    R: Redact,
    E: Redact,
{
    fn redact(self) -> Self
    where
        Self: Sized,
    {
        match self {
            Ok(v) => Ok(v.redact()),
            Err(e) => Err(e.redact()),
        }
    }
}

/// [Redacted] is a type guard that can be used to ensure that values have been redacted. It is
/// impossible to construct `Redacted<T>` with an unredacted T.
///
/// The
///
/// ### Usage
///
/// ```rust
/// use redact::{Redact, Redacted};
///
/// #[derive(Debug, Redact)]
/// struct PII {
///     #[redact]
///     name: String,
/// };
///
/// let pii = PII { name: "Alice".to_string() };
///
/// do_stuff(pii.into());
///
/// fn do_stuff(pii: Redacted<PII>) {
///     println!("Some redacted pii: {pii:?}");
/// }
/// ```
pub struct Redacted<T>(T);

impl<T> From<T> for Redacted<T>
where
    T: Redact,
{
    fn from(value: T) -> Self {
        Redacted(value.redact())
    }
}

#[allow(dead_code)]
impl<T> Redacted<T> {
    fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Redacted<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Redacted<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> std::fmt::Display for Redacted<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> std::fmt::Debug for Redacted<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Redact for Vec<T>
where
    T: Redact,
{
    fn redact(self) -> Self
    where
        Self: Sized,
    {
        self.into_iter().map(Redact::redact).collect()
    }
}

impl<K, V> Redact for HashMap<K, V>
where
    K: std::hash::Hash + std::cmp::Eq,
    V: Redact,
{
    fn redact(self) -> Self
    where
        Self: Sized,
    {
        self.into_iter().map(|(k, v)| (k, v.redact())).collect()
    }
}

impl<T> Redact for HashSet<T>
where
    T: Redact + std::hash::Hash + std::cmp::Eq,
{
    fn redact(self) -> Self
    where
        Self: Sized,
    {
        self.into_iter().map(Redact::redact).collect()
    }
}

#[cfg(feature = "zeroize")]
impl<T> Redact for Secret<T>
where
    T: DefaultIsZeroes,
    T: Zeroize,
{
    fn redact(self) -> Self
    where
        Self: Sized,
    {
        self
    }
}
