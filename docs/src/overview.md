# Expunge

A crate for expunging sensitive fields.

> **Expunge**
>   
>       1: to strike out, obliterate, or mark for deletion     
>   
> *"In medieval and Renaissance manuscripts, a series of dots was used to mark mistakes or to label material that should be deleted from a text, and those deletion dots can help you remember the history of expunge. They were known as puncta delentia. The puncta part of the name derives from the Latin verb pungere, which can be translated as "to prick or sting" (and you can imagine that a scribe may have felt stung when their mistakes were so punctuated in a manuscript). Pungere is also an ancestor of expunge, as well as a parent of other dotted, pointed, or stinging terms such as punctuate, compunction, poignant, puncture, and pungent."*    
> 
> Source: [https://www.merriam-webster.com/dictionary/expunge](https://www.merriam-webster.com/dictionary/expunge)


## About

At the core of `Expunge` is the `Expunge` trait, which is used for all transformations.

```rust
pub trait Expunge {
    fn expunge(self) -> Self
    where
        Self: Sized;
}
```

Other crates offer similar functionality, but either require types to be changed or 
make it difficult for both the expunged and unexpunged data being used at runtime.

This crate provides a proc_macro that derives the `Expunge` trait for the given type. 
When the `Expunge::expunge` method is called, sensitive fields are transformed/redacted.

- All fields are transformed unless annotated with `#[expunge(skip)]`
- The `Expunge` macro first looks for transformations declared on field/struct attributes i.e. `as` or `with`. 
  If these aren't set then `Expunge` macro will use the `Expunge::expunge` implementation for the type.
- A default implementation for the `Expunge` trait is provided for primitive types and common container types.
  These will be expunged as their default values, unless otherwise specified.
  
Since expunge doesn't require types to be changed, migrating to this crate should be completely frictionless.

## Similar crates

- [secrecy](https://crates.io/crates/secrecy): Prevents secrets being logged/serialized by wrapping them in a `Secret<T>` type
- [veil](https://crates.io/crates/veil): A proc_macro similar to this crate to implement expunged `std::fmt::Debug` and/or `std::fmt::Display`
- [redact](https://crates.io/crates/redact): Similar to [secrecy](https://docs.rs/secrecy/latest/secrecy/), but without the memory zeroizing
- [redacted](https://crates.io/crates/redacted): Wrappers to control debug formatting of potentially sensitive byte arrays 


### Comparison

| crate                                         | proc_macro | implements Debug | serde support | toggle on/off at runtime | uses original types | slog support |
| ---                                           | ---        | ---              | ---           | ---                      | ---                 | ---          |
| [secrecy](https://crates.io/crates/secrecy)   | ✘          | ✔                | ✔             | ✘                        | ✘                   | ✘            |
| [redact](https://crates.io/crates/redact)     | ✘          | ✔                | ✔             | ✘                        | ✘                   | ✘            |
| [veil](https://crates.io/crates/veil)         | ✔          | ✔                | ✘             | ✘                        | ✘                   | ✘            |
| [redacted](https://crates.io/crates/redacted) | ✘          | ✔                | ✘             | ✘                        | ✘                   | ✘            |
| [expunge](#Expunge)                           | ✔          | ✔                | ✔             | ✔                        | ✔                   | ✔            |

