#!/bin/bash

# Usage: ./gen_primitives.sh > primitives.rs

# Doing this to avoid typing and more macros. There's almost certainly a better way.

types=(
"i8" "i16" "i32" "i64" "i128" "isize"
"u8" "u16" "u32" "u64" "u128" "usize"
"f32" "f64"
"bool"
"()"
"String"
)

echo "use super::Redact;"
echo

for ty in ${types[@]}; do
  cat <<EOF
impl Redact for $ty {
    fn redact(self) -> Self
    where
        Self: Sized,
    {
        Self::default()
    }
}

EOF
done
