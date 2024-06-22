# Container attributes

> attributes that apply to a struct or enum declaration

### `as` 
  
Provide a value that all the fields should be set to when expunged. e.g. `Default::default()` or `"<expunged>".to_string()`

Example:

In this example, all fields will be replaced with the string `"<redacted>"` when expunged.

```rust
{{#include ../../../expunge/tests/book/container_as.rs}}
```

### `default` 

Shorthand for `as = Default::default()`. All fields will be expunged using their `Default::default()` implementations.

Example:

```rust
{{#include ../../../expunge/tests/book/container_default.rs}}
```

### `with` 
  
Expunge all fields using this function.

It must return the same type as it takes. e.g. hash a `String` with `sha256::digest`.

If you own the type, then could also implement `Expunge` directly. 
Using `with`, however, allows you to use different transformations for different fields of the same type.

Example:

In this example, fields will be replaced with their sha256 hashes.

```rust
{{#include ../../../expunge/tests/book/container_with.rs}}
```

### `allow_debug` 

By default, expunge provides its own `Debug` implementation. 
This attribute disables the default implementation, allowing the user to implement or derive their own.
  
Example:

In this example, fields will be replaced with their sha256 hashes.

```rust
{{#include ../../../expunge/tests/book/allow_debug.rs}}
```

### `slog` 

Integrates with slog, see [slog.md](../../slog.md).
