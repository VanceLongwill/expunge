use std::ops::{Deref, DerefMut};

pub use redact_derive::*;

pub mod primitives;
pub use primitives::*;

#[cfg(feature = "zeroize")]
#[doc(hidden)]
pub use ::zeroize;

#[cfg(feature = "serde")]
#[doc(hidden)]
pub use ::serde;

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
/// impossible to construct Redacted<T> with an unredacted T.
///
/// # Usage
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
