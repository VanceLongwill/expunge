pub use redact_derive::*;

pub trait Redact {
    fn redact(self) -> Self;
}

impl Redact for String {
    fn redact(self) -> Self {
        String::default()
    }
}
