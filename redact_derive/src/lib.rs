extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Attribute, Data, DataEnum, DataStruct,
    DeriveInput, Expr, Field, Fields, GenericParam, Generics, Index, Meta,
};

#[proc_macro_derive(Redact, attributes(redact))]
pub fn redact_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match try_redact_derive(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}

fn try_redact_derive(input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let span = input.span();
    let builder = parse_attrs(span, None, input.attrs)?;
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
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let expanded = quote! {
        impl #impl_generics redact::Redact for #name #ty_generics #where_clause {
            fn redact(self) -> Self {
                use ::redact::*;

                #impls
            }
        }
    };

    Ok(expanded)
}

// Add a bound `T: redact::Redact` to every type parameter T.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(redact::Redact));
        }
    }
    generics
}

#[derive(Debug, Clone, Default)]
struct Builder {
    attr_as: Option<TokenStream>,
    attr_with: Option<TokenStream>,
    ignore: bool,
}

impl Builder {
    fn build(self, span: Span, ident: TokenStream) -> Result<TokenStream, syn::Error> {
        let Self {
            attr_as,
            attr_with,
            ignore,
        } = self;
        if ignore {
            return Ok(TokenStream::default());
        }
        match (attr_as, attr_with) {
            (Some(attr_as), None) => Ok(quote_spanned! { span =>
                #ident = #attr_as;
            }),
            (None, Some(attr_with)) => Ok(quote_spanned! { span =>
                #ident = #attr_with(#ident);
            }),
            (None, None) => Ok(quote_spanned! { span =>
                #ident = #ident.redact();
            }),
            _ => Err(syn::Error::new(
                span,
                "unsupported combination of attributes",
            )),
        }
    }
}

fn parse_attrs(
    span: Span,
    parent: Option<Builder>,
    attrs: Vec<Attribute>,
) -> Result<Option<Builder>, syn::Error> {
    let attrs: Vec<_> = attrs
        .into_iter()
        .filter(|attr| attr.path().is_ident("redact"))
        .collect();

    match attrs.len() {
        0 => Ok(parent),
        1 => {
            let attr = &attrs[0];
            let mut builder = Builder::default();

            if matches!(attr.meta, Meta::Path(..)) {
                return Ok(Some(builder));
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("as") {
                    let expr: Expr = meta.value()?.parse()?;
                    builder.attr_as = Some(expr.into_token_stream());
                    Ok(())
                } else if meta.path.is_ident("with") {
                    let expr: Expr = meta.value()?.parse()?;
                    builder.attr_with = Some(expr.into_token_stream());
                    Ok(())
                } else if meta.path.is_ident("ignore") {
                    builder.ignore = true;
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
            format!("expected 1 or 0 `redact` tags, found {n}"),
        )),
    }
}

fn derive_builder(
    fields: impl IntoIterator<Item = Field>,
    parent: Option<Builder>,
) -> Result<Vec<Option<Builder>>, syn::Error> {
    fields
        .into_iter()
        .map(|field| {
            let span = field.span();
            parse_attrs(span, parent.clone(), field.attrs)
        })
        .collect()
}

fn derive_fields(
    is_enum: bool,
    prefix: TokenStream,
    fields: impl IntoIterator<Item = Field>,
    parent: Option<Builder>,
) -> Result<TokenStream, syn::Error> {
    fields
        .into_iter()
        .enumerate()
        .map(|(i, field)| {
            let span = field.span();
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

            match (
                parse_attrs(span, parent.clone(), field.attrs)?,
                parent.clone(),
            ) {
                (Some(builder), _) | (None, Some(builder)) => builder.build(span, ident),
                (None, None) => Ok(TokenStream::default()),
            }
        })
        .collect()
}

fn get_fields(fields: Fields) -> Result<impl IntoIterator<Item = Field>, syn::Error> {
    match fields {
        Fields::Named(named) => Ok(named.named),
        Fields::Unnamed(unnamed) => Ok(unnamed.unnamed),
        unit @ Fields::Unit => Err(syn::Error::new(
            unit.span(),
            "Unit structs are not supported",
        )),
    }
}

fn derive_struct(s: DataStruct, parent: Option<Builder>) -> Result<TokenStream, syn::Error> {
    let impls = derive_fields(false, quote! { next }, get_fields(s.fields)?, parent)?;

    Ok(quote! {
        let mut next = self;

        #impls

        next
    })
}

fn derive_enum(e: DataEnum, parent: Option<Builder>) -> Result<TokenStream, syn::Error> {
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
            let parent = parse_attrs(span, None, variant.attrs.clone())?.or(parent.clone());

            let prefix = match &variant.fields {
                Fields::Named(..) => quote! {},
                Fields::Unnamed(..) => quote! { arg },
                Fields::Unit => TokenStream::default(),
            };

            derive_fields(true, prefix, get_fields(variant.fields.clone())?, parent)
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
