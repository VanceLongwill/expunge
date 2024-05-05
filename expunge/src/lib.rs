#![doc = include_str!("../../README.md")]

use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
};

pub use expunge_derive::*;

pub mod primitives;

/// A collection of utils for common ways to expunge things
pub mod utils;

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

/// [Expunged] is a type guard that can be used to ensure that values have been expunged. It is
/// impossible to construct `Expunged<T>` with an unexpunged T.
///
/// The
///
/// ### Usage
///
/// ```rust
/// use expunge::{Expunge, Expunged};
///
/// #[derive(Debug, Expunge)]
/// #[expunge(allow_debug)]
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
///     println!("Some expunged pii: {pii:?}");
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

impl<T> Expunge for Box<T>
where
    T: Expunge,
{
    fn expunge(self) -> Self
    where
        Self: Sized,
    {
        Box::new((*self).expunge())
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
