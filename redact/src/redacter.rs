pub enum RedactOptions<As, With> {
    /// Provide a value that will be used in redacted copies
    As(As),
    /// Dynamically redact values
    With(With),
}

pub struct RedactedBuilder<As, With> {
    opt: RedactOptions<As, With>,
}

pub trait Redacter<T> {
    fn redact(self, value: T) -> T;
}

impl<Inner, As, With> Redacter<Inner> for RedactedBuilder<As, With>
where
    As: Into<Inner>,
    As: Clone,
    With: Redacter<Inner>,
{
    fn redact(self, value: Inner) -> Inner
    where
        Self: Sized,
    {
        match self.opt {
            RedactOptions::As(a) => a.clone().into(),
            RedactOptions::With(with) => with.redact(value),
        }
    }
}

#[derive(Clone, Copy)]
pub struct DefaultRedacter;

impl<T> Redacter<T> for DefaultRedacter
where
    T: Default,
{
    fn redact(self, _value: T) -> T {
        T::default()
    }
}

#[derive(Clone, Copy)]
pub struct HashRedacter;

impl<T> Redacter<T> for HashRedacter
where
    T: AsRef<str>,
    T: From<String>,
{
    fn redact(self, _value: T) -> T {
        T::from("hashed".to_string())
    }
}
