pub mod redacter;
pub use redact_derive::*;

pub trait Redact {
    fn redact(self) -> Self
    where
        Self: Sized;
}
