# Logging with `slog`

Expunge provides a painless and (relatively) foolproof way to log structs that may contain sensitive fields. 
As long as your type implements `serde::Serialize`, the `slog` attribute will derive `slog::SerdeValue`.
Internally the value will be expunged before logging.

#### Example

```rust
{{#include ../../expunge/tests/book/slog.rs}}
```
