use super::Redact;

#[doc(hidden)]
macro_rules! redact_as_default {
    ($typ:ty) => {
        impl Redact for $typ {
            fn redact(self) -> Self
            where
                Self: Sized,
            {
                Self::default()
            }
        }
    };
}

redact_as_default!(i8);
redact_as_default!(i16);
redact_as_default!(i32);
redact_as_default!(i64);
redact_as_default!(i128);
redact_as_default!(isize);
redact_as_default!(u8);
redact_as_default!(u16);
redact_as_default!(u32);
redact_as_default!(u64);
redact_as_default!(u128);
redact_as_default!(usize);
redact_as_default!(f32);
redact_as_default!(f64);
redact_as_default!(bool);
redact_as_default!(());
redact_as_default!(String);
redact_as_default!(&str);
