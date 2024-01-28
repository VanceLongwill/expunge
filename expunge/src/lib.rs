//! Expunge provides a straightfoward macro-based approach for dealing with
//! sensitive values.
//!
//! Keeping track of which values are sensitive (e.g. PII, secrets) is as simple as
//! marking them with the `#[expunge]` attribute. Then, when you need a sanitized copy of your data,
//! simply do `let sanitized = data.expunge();`. If no other expunge behaviour is specified (see `as`
//! & `with`), the field will be replaced with its default value.
//!
//! ### Usage
//!
//! ```rust
//! use expunge::Expunge;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Clone, Debug, Serialize, Deserialize, Expunge)]
//! struct User {
//!   id: i64, // fields without #[expunge] annotations are left as is
//!   #[expunge(as = "Randy".to_string())]
//!   first_name: String,
//!   #[expunge(as = "Lahey".to_string())]
//!   last_name: String,
//!   #[expunge(with = sha256::digest)]
//!   date_of_birth: String,
//!   #[expunge]
//!   latitude: f64,
//!   #[expunge]
//!   longitude: f64,
//!   #[expunge(as = "<expungeed>".to_string(), zeroize)]
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
//! let expungeed_user = user.expunge();
//!
//! let output = serde_json::to_string_pretty(&expungeed_user).expect("should serialize");
//!
//! assert_eq!(
//!   r#"{
//!   "id": 101,
//!   "first_name": "Randy",
//!   "last_name": "Lahey",
//!   "date_of_birth": "eeb98c815ae11240b563892c52c8735472bb8259e9a6477e179a9ea26e7a695a",
//!   "latitude": 0.0,
//!   "longitude": 0.0,
//!   "password_hash": "<expungeed>"
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
//! | `as`      | provide a value that this field should be set to when expungeed. e.g. `Default::default()` or `"<expungeed>".to_string()`                                 | -         |
//! | `with`    | provide a function that will be called when expunging this value. It must return the same type as it takes. e.g. hash a `String` with `sha256::digest`. | -         |
//! | `all`     | can be used instead of specifying `#[expunge]` on every field/variant in a struct or enum                                                                | -         |
//! | `ignore`  | can be used to skip fields in combination with `all`                                                                                                    | -         |
//! | `zeroize` | zeroize memory for extra security via the [secrecy](https://crates.io/crates/secrecy) & [zeroize](https://crates.io/crates/zeroize) crates              | `zeroize` |
//!
//!

use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
};

pub use expunge_derive::*;

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

/// Trait for recursively expunging values marked as sensitive
pub trait Expunge {
    fn expunge(self) -> Self
    where
        Self: Sized;
}

impl<T> Expunge for Option<T>
where
    T: Expunge,
{
    fn expunge(self) -> Self
    where
        Self: Sized,
    {
        self.map(Expunge::expunge)
    }
}

impl<R, E> Expunge for Result<R, E>
where
    R: Expunge,
    E: Expunge,
{
    fn expunge(self) -> Self
    where
        Self: Sized,
    {
        match self {
            Ok(v) => Ok(v.expunge()),
            Err(e) => Err(e.expunge()),
        }
    }
}

/// [Expunged] is a type guard that can be used to ensure that values have been expungeed. It is
/// impossible to construct `Expunged<T>` with an unexpungeed T.
///
/// The
///
/// ### Usage
///
/// ```rust
/// use expunge::{Expunge, Expunged};
///
/// #[derive(Debug, Expunge)]
/// struct PII {
///     #[expunge]
///     name: String,
/// };
///
/// let pii = PII { name: "Alice".to_string() };
///
/// do_stuff(pii.into());
///
/// fn do_stuff(pii: Expunged<PII>) {
///     println!("Some expungeed pii: {pii:?}");
/// }
/// ```
pub struct Expunged<T>(T);

impl<T> From<T> for Expunged<T>
where
    T: Expunge,
{
    fn from(value: T) -> Self {
        Expunged(value.expunge())
    }
}

#[allow(dead_code)]
impl<T> Expunged<T> {
    fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Deref for Expunged<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Expunged<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> std::fmt::Display for Expunged<T>
where
    T: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> std::fmt::Debug for Expunged<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Expunge for Vec<T>
where
    T: Expunge,
{
    fn expunge(self) -> Self
    where
        Self: Sized,
    {
        self.into_iter().map(Expunge::expunge).collect()
    }
}

impl<K, V> Expunge for HashMap<K, V>
where
    K: std::hash::Hash + std::cmp::Eq,
    V: Expunge,
{
    fn expunge(self) -> Self
    where
        Self: Sized,
    {
        self.into_iter().map(|(k, v)| (k, v.expunge())).collect()
    }
}

impl<T> Expunge for HashSet<T>
where
    T: Expunge + std::hash::Hash + std::cmp::Eq,
{
    fn expunge(self) -> Self
    where
        Self: Sized,
    {
        self.into_iter().map(Expunge::expunge).collect()
    }
}

#[cfg(feature = "zeroize")]
impl<T> Expunge for Secret<T>
where
    T: DefaultIsZeroes,
    T: Zeroize,
{
    fn expunge(self) -> Self
    where
        Self: Sized,
    {
        self
    }
}
