extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Attribute, Data, DataEnum, DataStruct,
    DeriveInput, Expr, Field, Fields, GenericParam, Generics, Index, Meta,
};

#[proc_macro_derive(Expunge, attributes(expunge))]
pub fn expunge_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match try_expunge_derive(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn try_expunge_derive(input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let span = input.span();
    let builder = parse_attributes(span, None, input.attrs)?.unwrap_or_default();
    let slog_enabled = builder.slog;

    let impls = match input.data {
        Data::Struct(s) => derive_struct(s, builder)?,
        Data::Enum(e) => derive_enum(e, builder)?,
        Data::Union(_) => {
            return Err(syn::Error::new(
                input.ident.span(),
                "this trait cannot be derived for unions",
            ))
        }
    };
    let name = input.ident;

    let generics = add_trait_bounds(input.generics);
    let g = generics.clone();
    let (impl_generics, ty_generics, where_clause) = g.split_for_impl();

    let slog_impl = if slog_enabled {
        let generics = add_slog_trait_bounds(generics);
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        quote! {
                impl #impl_generics ::slog::Value for #name #ty_generics #where_clause {
                    fn serialize(
                        &self,
                        record: &::slog::Record,
                        key: ::slog::Key,
                        serializer: &mut dyn ::slog::Serializer,
                    ) -> slog::Result {
                        use ::serde::Serialize;
                        use ::slog_derive::SerdeValue;
                        use ::expunge::slog_debug::is_disabled;

                        #[derive(Clone, Serialize, SerdeValue)]
                        pub struct Wrapped {
                            #[slog]
                            #[serde(flatten)]
                            item: #name,
                        }


                        #[cfg(not(feature = "slog_debug"))]
                        let item = self.clone().expunge();

                        #[cfg(feature = "slog_debug")]
                        let item = if is_disabled() {
                            self.clone()
                        } else {
                            self.clone().expunge()
                        };

                        let wrapped = Wrapped {
                            item,
                        };
                        ::slog::Value::serialize(&wrapped, record, key, serializer)
                    }
                }
        }
    } else {
        TokenStream::default()
    };

    let expanded = quote! {
        #slog_impl

        impl #impl_generics expunge::Expunge for #name #ty_generics #where_clause {
            fn expunge(self) -> Self {
                use ::expunge::*;

                #impls
            }
        }
    };

    Ok(expanded)
}

fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(expunge::Expunge));
        }
    }
    generics
}

fn add_slog_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(::serde::Serialize));
            type_param.bounds.push(parse_quote!(Clone));
        }
    }
    generics
}

#[derive(Debug, Clone, Default)]
struct Builder {
    // an expression to use as the expunged value
    expunge_as: Option<TokenStream>,
    // an function that takes the un-expunged value and returns an expunged value
    expunge_with: Option<TokenStream>,
    // ignore this field
    ignore: bool,
    // expunge all fields in this container
    all: bool,
    // zeroize the memory when expunging (only the current copy)
    zeroize: bool,
    // implement slog::SerdeValue for this type, expunging the value before logging
    slog: bool,
}

impl Builder {
    fn build(self, span: Span, ident: TokenStream) -> Result<TokenStream, syn::Error> {
        let Self {
            expunge_as,
            expunge_with,
            ignore,
            all: _,
            zeroize,
            slog: _,
        } = self;
        if ignore {
            return Ok(TokenStream::default());
        }

        let zeroizer = if zeroize {
            quote! {
                use ::expunge::secrecy::Secret;
                let _ = Secret::new(#ident);
            }
        } else {
            TokenStream::default()
        };

        match (expunge_as, expunge_with) {
            (Some(expunge_as), None) => Ok(quote_spanned! { span =>
                #zeroizer
                #ident = #expunge_as;
            }),
            (None, Some(expunge_with)) => Ok(quote_spanned! { span =>
                #ident = #expunge_with(#ident);
            }),
            (None, None) => Ok(quote_spanned! { span =>
                #ident = #ident.expunge();
            }),
            _ => Err(syn::Error::new(
                span,
                "unsupported combination of attributes",
            )),
        }
    }
}

const WITH: &str = "with";
const AS: &str = "as";
const ALL: &str = "all";
const IGNORE: &str = "ignore";
const ZEROIZE: &str = "zeroize";
const SLOG: &str = "slog";
const DEFAULT: &str = "default";

fn parse_attributes(
    span: Span,
    parent: Option<Builder>,
    attrs: Vec<Attribute>,
) -> Result<Option<Builder>, syn::Error> {
    let attrs: Vec<_> = attrs
        .into_iter()
        .filter(|attr| attr.path().is_ident("expunge"))
        .collect();

    let is_container = parent.is_none();

    match attrs.len() {
        0 => Ok(parent.and_then(|p| if p.all { Some(p) } else { None })),
        1 => {
            let attr = &attrs[0];

            if matches!(attr.meta, Meta::Path(..)) {
                return parent
                    .ok_or(syn::Error::new(
                        attr.meta.span(),
                        "`#[expunge]` can only be used to mark fields & variants".to_string(),
                    ))
                    .map(Some);
            }

            let mut builder = Builder::default();

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident(AS) {
                    if builder.expunge_with.is_some() {
                        return Err(syn::Error::new(
                            meta.path.span(),
                            format!("`{AS}` cannot be combined with `{WITH}`"),
                        ));
                    }
                    let expr: Expr = meta.value()?.parse()?;
                    builder.expunge_as = Some(expr.into_token_stream());
                    Ok(())
                } else if meta.path.is_ident(WITH) {
                    if builder.expunge_as.is_some() {
                        return Err(syn::Error::new(
                            meta.path.span(),
                            format!("`{WITH}` cannot be combined with `{AS}`"),
                        ));
                    }
                    let expr: Expr = meta.value()?.parse()?;
                    builder.expunge_with = Some(expr.into_token_stream());
                    Ok(())
                } else if meta.path.is_ident(IGNORE) {
                    if is_container {
                        return Err(syn::Error::new(
                            meta.path.span(),
                            format!("`{IGNORE}` is not permitted on containers"),
                        ));
                    }
                    builder.ignore = true;
                    Ok(())
                } else if meta.path.is_ident(ALL) {
                    if !is_container {
                        return Err(syn::Error::new(
                                meta.path.span(),
                                format!("`{ALL}` is not permitted on fields or variants, use #[expunge] instead"),
                        ));
                    }
                    builder.all = true;
                    Ok(())
                } else if meta.path.is_ident(ZEROIZE) {
                    if cfg!(feature = "zeroize") {
                        if builder.expunge_with.is_some() {
                            return Err(syn::Error::new(
                                meta.path.span(),
                                format!("`{ZEROIZE}` cannot be combined with `{WITH}`"),
                            ));
                        }
                        if builder.expunge_as.is_none() {
                            return Err(syn::Error::new(
                                meta.path.span(),
                                format!("`{ZEROIZE}` requires that `{AS}` be specified since it consumes the value"),
                            ));
                        }
                        builder.zeroize = true;
                        Ok(())
                    } else {
                        Err(syn::Error::new(
                            meta.path.span(),
                            format!("the `{ZEROIZE}` feature must be enabled"),
                        ))
                    }
                } else if meta.path.is_ident(SLOG) {
                    if cfg!(feature = "slog") {
                        if !is_container {
                            return Err(syn::Error::new(
                                    meta.path.span(),
                                    format!("`{SLOG}` is not permitted on fields or variants"),
                            ));
                        }
                        builder.slog = true;
                        Ok(())
                    } else {
                        Err(syn::Error::new(
                            meta.path.span(),
                            format!("the `{SLOG}` feature must be enabled"),
                        ))
                    }
                } else if meta.path.is_ident(DEFAULT) {
                    builder.expunge_as = Some(quote!{ Default::default() });
                    Ok(())
                } else {
                    Err(syn::Error::new(
                        meta.path.span(),
                        format!("unrecognized option `{:?}`", meta.path),
                    ))
                }
            })?;

            Ok(Some(builder))
        }
        n => Err(syn::Error::new(
            span,
            format!("expected 1 or 0 `expunge` tags, found {n}"),
        )),
    }
}

fn derive_fields(
    is_enum: bool,
    prefix: TokenStream,
    fields: impl IntoIterator<Item = Field>,
    parent: Builder,
) -> Result<TokenStream, syn::Error> {
    fields
        .into_iter()
        .enumerate()
        .map(|(i, field)| {
            let span = field.span();
            let builder = parse_attributes(span, Some(parent.clone()), field.attrs)?
                .map(|f| {
                    let Builder {
                        expunge_as,
                        expunge_with,
                        ignore,
                        all,
                        zeroize,
                        slog,
                    } = f;
                    let (expunge_as, expunge_with) = match (expunge_as, expunge_with) {
                        (Some(ra), None) => (Some(ra), None),
                        (None, Some(rw)) => (None, Some(rw)),
                        (None, None) => (parent.expunge_as.clone(), parent.expunge_with.clone()),
                        (Some(_), Some(_)) => {
                            return Err(syn::Error::new(span, "`as` and `with` cannot be combined"))
                        }
                    };
                    let ignore = ignore || parent.ignore;
                    let all = all || parent.all;
                    let zeroize = zeroize || parent.zeroize;
                    Ok(Builder {
                        expunge_as,
                        expunge_with,
                        ignore,
                        all,
                        zeroize,
                        slog,
                    })
                })
                .transpose()?;

            let builder = if parent.all {
                builder.or(Some(parent.clone()))
            } else {
                builder
            };

            Ok(builder
                .map(|builder| {
                    let ident = match field.ident {
                        Some(named) => {
                            if is_enum {
                                named.into_token_stream()
                            } else {
                                quote! { #prefix.#named }
                            }
                        }
                        None => {
                            if is_enum {
                                Ident::new(&format!("{prefix}{i}"), span).into_token_stream()
                            } else {
                                let index = Index::from(i);
                                quote! { #prefix.#index }
                            }
                        }
                    };

                    builder.build(span, ident)
                })
                .transpose()?
                .unwrap_or(TokenStream::default()))
        })
        .collect()
}

fn get_fields(fields: Fields) -> Option<impl IntoIterator<Item = Field>> {
    match fields {
        Fields::Named(named) => Some(named.named),
        Fields::Unnamed(unnamed) => Some(unnamed.unnamed),
        Fields::Unit => None,
    }
}

fn derive_struct(s: DataStruct, parent: Builder) -> Result<TokenStream, syn::Error> {
    let impls = get_fields(s.fields)
        .map(|fields| derive_fields(false, quote! { next }, fields, parent))
        .transpose()?;

    Ok(quote! {
        let mut next = self;

        #impls

        next
    })
}

fn derive_enum(e: DataEnum, parent: Builder) -> Result<TokenStream, syn::Error> {
    let span = e.enum_token.span();

    let variant_idents = e.variants.iter().map(|variant| &variant.ident);

    let variant_destructures = e.variants.iter().map(|variant| match &variant.fields {
        syn::Fields::Named(syn::FieldsNamed { named, .. }) => {
            let idents = named.iter().map(|field| field.ident.as_ref().unwrap());
            quote! {
                { #(#idents),* }
            }
        }
        syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }) => {
            let args = (0..unnamed.len())
                .map(|i| syn::Ident::new(&format!("arg{i}"), unnamed.span()))
                .map(|ident| quote! { #ident });
            quote! {
                ( #(#args),* )
            }
        }
        syn::Fields::Unit => Default::default(),
    });

    let variant_destructures_mut = e.variants.iter().map(|variant| match &variant.fields {
        syn::Fields::Named(syn::FieldsNamed { named, .. }) => {
            let idents = named
                .iter()
                .map(|field| field.ident.as_ref().unwrap())
                .map(|ident| quote! { mut #ident });
            quote! {
                { #(#idents),* }
            }
        }
        syn::Fields::Unnamed(syn::FieldsUnnamed { unnamed, .. }) => {
            let args = (0..unnamed.len())
                .map(|i| syn::Ident::new(&format!("arg{i}"), unnamed.span()))
                .map(|ident| quote! { mut #ident });
            quote! {
                ( #(#args),* )
            }
        }
        syn::Fields::Unit => Default::default(),
    });

    let variant_bodies: Result<Vec<TokenStream>, syn::Error> = e
        .variants
        .iter()
        .map(|variant| {
            let parent = parse_attributes(span, Some(parent.clone()), variant.attrs.clone())?
                .map(|mut p| {
                    // the `#[expunge]` tag on an enum variant is equivalent to `#[expunge(all)]`
                    p.all = true;
                    p
                })
                .unwrap_or(parent.clone());

            let prefix = if let Fields::Unnamed(..) = &variant.fields {
                quote! { arg }
            } else {
                TokenStream::default()
            };

            get_fields(variant.fields.clone())
                .map(|fields| derive_fields(true, prefix, fields, parent))
                .transpose()
                .map(Option::unwrap_or_default)
        })
        .collect();

    let bodies = variant_bodies?.into_iter();

    Ok(quote_spanned! { span =>
        match self {
                    #(Self::#variant_idents #variant_destructures_mut => {
                        #bodies
                        Self::#variant_idents #variant_destructures
                    },)*
        }
    })
}
