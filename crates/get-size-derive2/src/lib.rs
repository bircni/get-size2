//! Derives the `GetSize` trait of [`get-size2`](https://docs.rs/get-size2) for structs and enums.
//!
//! This crate is re-exported by `get-size2` and is meant to be used through its `derive` feature.
//! See [`GetSize`](macro@GetSize) for the attribute reference.

use attribute_derive::FromAttr;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

/// The `#[get_size(...)]` attribute of a single field.
#[derive(FromAttr, Default, Debug)]
#[attribute(ident = get_size)]
struct FieldAttribute {
    #[attribute(conflicts = [size_fn, ignore])]
    size: Option<usize>,
    #[attribute(conflicts = [size, ignore])]
    size_fn: Option<syn::Expr>,
    #[attribute(conflicts = [size, size_fn])]
    ignore: bool,
}

/// How the heap size of a single field is determined.
enum FieldSize {
    /// `#[get_size(size = ...)]`: a fixed number of bytes.
    Fixed(usize),
    /// `#[get_size(size_fn = ...)]`: the result of calling a helper with a reference to the field.
    Helper(syn::ExprPath),
    /// `#[get_size(ignore)]`: nothing at all.
    Ignored,
    /// No attribute: the field's own [`GetSize`] implementation.
    Measured,
}

impl FieldSize {
    fn of(field: &syn::Field) -> syn::Result<Self> {
        let attr = FieldAttribute::from_attributes(&field.attrs)
            .map_err(|err| syn::Error::new_spanned(field, err.to_string()))?;

        Ok(if let Some(size) = attr.size {
            Self::Fixed(size)
        } else if let Some(size_fn) = attr.size_fn {
            // Parsing an expression, not a type like path, makes generic arguments require the
            // turbofish, exactly as they do when the helper is called by hand.
            let syn::Expr::Path(size_fn) = size_fn else {
                return Err(syn::Error::new_spanned(
                    size_fn,
                    "`size_fn` expects the name of or the path to a function",
                ));
            };

            Self::Helper(size_fn)
        } else if attr.ignore {
            Self::Ignored
        } else {
            Self::Measured
        })
    }

    /// Whether the generated code has to access the field's value.
    const fn reads_field(&self) -> bool {
        matches!(self, Self::Helper(_) | Self::Measured)
    }
}

/// A field, addressed by its name or by its position for tuple structs and tuple variants.
struct Field {
    member: syn::Member,
    size: FieldSize,
}

impl Field {
    fn collect(fields: &syn::Fields) -> syn::Result<Vec<Self>> {
        fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let member = field.ident.clone().map_or_else(
                    || syn::Member::Unnamed(syn::Index::from(index)),
                    syn::Member::Named,
                );

                Ok(Self {
                    member,
                    size: FieldSize::of(field)?,
                })
            })
            .collect()
    }
}

/// Generates the body which adds up the heap size of `fields` and returns it with the tracker.
///
/// `reference` builds the expression which evaluates to a reference to the given field. Every
/// identifier introduced here is prefixed, so that it can never collide with the name of a field
/// bound by an enum variant pattern.
fn sum_fields(fields: &[Field], reference: impl Fn(usize, &Field) -> TokenStream2) -> TokenStream2 {
    let additions = fields
        .iter()
        .enumerate()
        .filter_map(|(index, field)| match &field.size {
            FieldSize::Fixed(size) => Some(quote! {
                let __get_size_total = __get_size_total + #size;
            }),
            FieldSize::Helper(size_fn) => {
                let reference = reference(index, field);
                Some(quote! {
                    let __get_size_total = __get_size_total + #size_fn(#reference);
                })
            }
            FieldSize::Ignored => None,
            FieldSize::Measured => {
                let reference = reference(index, field);
                Some(quote! {
                    let (__get_size_field, __get_size_tracker) =
                        ::get_size2::GetSize::get_heap_size_with_tracker(#reference, __get_size_tracker);
                    let __get_size_total = __get_size_total + __get_size_field;
                })
            }
        });

    quote! {
        let __get_size_total = 0usize;
        #(#additions)*
        (__get_size_total, __get_size_tracker)
    }
}

/// Generates the match arm measuring a single enum variant.
fn variant_arm(variant: &syn::Variant) -> syn::Result<TokenStream2> {
    let ident = &variant.ident;
    let fields = Field::collect(&variant.fields)?;

    let binding = |index: usize| format_ident!("__get_size_field_{index}");

    // Only the fields which are actually read get bound, the rest is covered by the `..` rest
    // pattern. The struct like pattern works for named, tuple and unit variants alike.
    let bindings = fields
        .iter()
        .enumerate()
        .filter(|(_index, field)| field.size.reads_field())
        .map(|(index, field)| {
            let member = &field.member;
            let binding = binding(index);
            quote! { #member: #binding }
        });

    let body = sum_fields(&fields, |index, _field| {
        let binding = binding(index);
        quote! { #binding }
    });

    Ok(quote! {
        Self::#ident { #(#bindings,)* .. } => { #body }
    })
}

/// Extracts the generics listed by a struct, enum or variant level `#[get_size(ignore(A, B))]`.
fn ignored_generics_of(attr: &syn::Attribute, ignored: &mut Vec<syn::Ident>) -> syn::Result<()> {
    // Skip all attributes which do not belong to us.
    if !attr.path().is_ident("get_size") {
        return Ok(());
    }

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("ignore") && meta.input.peek(syn::token::Paren) {
            return meta.parse_nested_meta(|generic| {
                ignored.push(generic.path.require_ident()?.clone());
                Ok(())
            });
        }

        // Anything else would be accepted without having any effect.
        Err(meta.error(
            "only `ignore(...)` with a list of generic parameters is supported here, \
             `ignore`, `size` and `size_fn` belong on a field",
        ))
    })
}

/// Collects the generics which shall not receive a `GetSize` bound.
///
/// Fields are not searched: their attributes are parsed by [`FieldAttribute`], which only knows
/// the plain `ignore` flag.
fn ignored_generics(ast: &syn::DeriveInput) -> syn::Result<Vec<syn::Ident>> {
    let variants = match &ast.data {
        syn::Data::Enum(data_enum) => Some(&data_enum.variants),
        syn::Data::Struct(_) | syn::Data::Union(_) => None,
    };

    let attrs = ast.attrs.iter().chain(
        variants
            .into_iter()
            .flatten()
            .flat_map(|variant| &variant.attrs),
    );

    let mut ignored = Vec::new();
    for attr in attrs {
        ignored_generics_of(attr, &mut ignored)?;
    }

    // A misspelled name would otherwise silently keep its bound. Const parameters never receive
    // a bound, so listing one is harmless.
    for ident in &ignored {
        let known = ast.generics.params.iter().any(|param| match param {
            syn::GenericParam::Type(param) => param.ident == *ident,
            syn::GenericParam::Const(param) => param.ident == *ident,
            syn::GenericParam::Lifetime(_) => false,
        });

        if !known {
            return Err(syn::Error::new_spanned(
                ident,
                format!(
                    "`{ident}` is not a generic type parameter of `{}`",
                    ast.ident
                ),
            ));
        }
    }

    Ok(ignored)
}

/// Adds a bound `T: GetSize` to every type parameter `T`, unless it is ignored.
fn add_trait_bounds(mut generics: syn::Generics, ignored: &[syn::Ident]) -> syn::Generics {
    for type_param in generics.type_params_mut() {
        if !ignored.contains(&type_param.ident) {
            type_param
                .bounds
                .push(syn::parse_quote!(::get_size2::GetSize));
        }
    }

    generics
}

#[doc = include_str!("./derive.md")]
#[proc_macro_derive(GetSize, attributes(get_size))]
pub fn derive_get_size(input: TokenStream) -> TokenStream {
    match derive_get_size_impl(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn derive_get_size_impl(input: TokenStream) -> syn::Result<TokenStream2> {
    // Construct a representation of Rust code as a syntax tree that we can manipulate.
    let ast: syn::DeriveInput = syn::parse(input)?;

    // Traverse the parsed data to generate the body measuring the fields. Types without any
    // field own no heap memory, so the default implementation of the trait is already correct.
    let body = match &ast.data {
        syn::Data::Struct(data_struct) if data_struct.fields.is_empty() => None,
        syn::Data::Struct(data_struct) => {
            let fields = Field::collect(&data_struct.fields)?;

            Some(sum_fields(&fields, |_index, field| {
                let member = &field.member;
                quote! { &self.#member }
            }))
        }
        syn::Data::Enum(data_enum) if data_enum.variants.is_empty() => None,
        syn::Data::Enum(data_enum) => {
            let arms = data_enum
                .variants
                .iter()
                .map(variant_arm)
                .collect::<syn::Result<Vec<_>>>()?;

            Some(quote! {
                match self {
                    #(#arms)*
                }
            })
        }
        syn::Data::Union(_data_union) => {
            return Err(syn::Error::new_spanned(
                &ast.ident,
                "Deriving GetSize for unions is currently not supported.",
            ));
        }
    };

    let ignored = ignored_generics(&ast)?;

    // The name and the generics of the struct/enum.
    let name = &ast.ident;
    let generics = add_trait_bounds(ast.generics.clone(), &ignored);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let method = body.map(|body| {
        quote! {
            fn get_heap_size_with_tracker<__GetSizeTracker: ::get_size2::GetSizeTracker>(
                &self,
                __get_size_tracker: __GetSizeTracker,
            ) -> (usize, __GetSizeTracker) {
                #body
            }
        }
    });

    Ok(quote! {
        impl #impl_generics ::get_size2::GetSize for #name #ty_generics #where_clause {
            #method
        }
    })
}
