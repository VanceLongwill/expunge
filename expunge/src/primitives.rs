use super::Expunge;

#[doc(hidden)]
macro_rules! expunge_as_default {
    ($typ:ty) => {
        impl Expunge for $typ {
            fn expunge(self) -> Self
            where
                Self: Sized,
            {
                Self::default()
            }
        }
    };
}

expunge_as_default!(i8);
expunge_as_default!(i16);
expunge_as_default!(i32);
expunge_as_default!(i64);
expunge_as_default!(i128);
expunge_as_default!(isize);
expunge_as_default!(u8);
expunge_as_default!(u16);
expunge_as_default!(u32);
expunge_as_default!(u64);
expunge_as_default!(u128);
expunge_as_default!(usize);
expunge_as_default!(f32);
expunge_as_default!(f64);
expunge_as_default!(bool);
expunge_as_default!(());
expunge_as_default!(String);
expunge_as_default!(&str);
