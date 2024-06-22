# Field attributes

> attributes that can be applied to a struct field, enum variant or field in an enum variant

### `as` 
  
Provide a value that the given field/variant should be set to when expunged. e.g. `"<expunged>".to_string()`

```rust
{{#include ../../../expunge/tests/book/field_as.rs}}
```

### `default` 

Shorthand for `as = Default::default()`

Example:

```rust
{{#include ../../../expunge/tests/book/field_default.rs}}
```

### `with` 
  
Expunge the field/variant using this function.

It must return the same type as it takes. e.g. hash a `String` with `sha256::digest`

If you own the type, then could also implement `Expunge` directly. 
Using `with`, however, allows you to use different transformations for different fields of the same type.

Example:

```rust
{{#include ../../../expunge/tests/book/field_with.rs}}
```

### `skip`

Skips a field. Fields marked `skip` will be left as-is. This is useful when:
1. You want to preserve fields within a struct that are not sensitive
2. The type cannot be expunged in a meaningful way

```rust
{{#include ../../../expunge/tests/book/field_skip.rs}}
```

### `zeroize`

Zeroize memory for extra security via the [secrecy](https://crates.io/crates/secrecy) & [zeroize](https://crates.io/crates/zeroize) crates.

Example:

```rust
{{#include ../../../expunge/tests/book/field_zeroize.rs}}
```

