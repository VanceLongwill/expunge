extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse_macro_input, parse_quote, spanned::Spanned, Data, DataEnum, DataStruct, DeriveInput,
    Expr, Fields, GenericParam, Generics, Meta,
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
    let impls = match input.data {
        Data::Struct(s) => derive_struct(s)?,
        Data::Enum(e) => derive_enum(e)?,
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

                let mut next = self;

                #impls

                next
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

fn derive_struct(s: DataStruct) -> Result<TokenStream, syn::Error> {
    let span = s.struct_token.span;
    let fields_named = match s.fields {
        Fields::Named(named) => Ok(named),
        Fields::Unnamed(_) => Err(syn::Error::new(span, "Unnamed fields are not supported")),
        Fields::Unit => Err(syn::Error::new(span, "Unit structs are not supported")),
    }?;

    fields_named
        .named
        .into_iter()
        .enumerate()
        .map(|(i, field)| {
            let span = field.span();
            let ident = field.ident.unwrap();
            // @TODO: support unnamed fields
            //.unwrap_or_else(|| {
            //    let index = Index::from(i);
            //    quote! { self.#index }
            //});
            //
            let attrs: Vec<_> = field
                .attrs
                .into_iter()
                .filter(|attr| attr.path().is_ident("redact"))
                .collect();

            match attrs.len() {
                0 => Ok(TokenStream::default()),
                1 => Ok({
                    let attr = &attrs[0];
                    let span = attr.span();
                    let mut attr_as: Option<TokenStream> = None;
                    let mut attr_with: Option<TokenStream> = None;

                    if matches!(attr.meta, Meta::Path(..)) {
                        return Ok(quote! {
                            next.#ident = next.#ident.redact();
                        });
                    }

                    //let list = attr.meta.require_list()?;
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("as") {
                            let expr: Expr = meta.value()?.parse()?;
                            attr_as = Some(expr.into_token_stream());
                            Ok(())
                        } else if meta.path.is_ident("with") {
                            let expr: Expr = meta.value()?.parse()?;
                            attr_with = Some(expr.into_token_stream());
                            Ok(())
                        } else {
                            Err(syn::Error::new(
                                meta.path.span(),
                                format!("unrecognized option `{:?}`", meta.path),
                            ))
                        }
                    })?;

                    match (attr_as, attr_with) {
                        (Some(attr_as), None) => Ok(quote_spanned! { span =>
                            next.#ident = #attr_as;
                        }),
                        (None, Some(attr_with)) => Ok(quote_spanned! { span =>
                            next.#ident = #attr_with(next.#ident);
                        }),
                        (None, None) => Ok(quote_spanned! { span =>
                            next.#ident = next.#ident.redact();
                        }),
                        _ => Err(syn::Error::new(
                            span,
                            "unsupported combination of attributes",
                        )),
                    }?
                }),
                n => Err(syn::Error::new(
                    span,
                    format!("expected 1 or 0 `redact` tags, found {n}"),
                )),
            }
        })
        .collect()
}

fn derive_enum(e: DataEnum) -> Result<TokenStream, syn::Error> {
    Ok(quote!().into_token_stream())
}
