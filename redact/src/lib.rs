pub use redact_derive::*;

pub mod primitives;
pub use primitives::*;

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
