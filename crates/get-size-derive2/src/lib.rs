//! Derives the `GetSize` trait of [`get-size2`](https://docs.rs/get-size2) for structs and enums.
//!
//! This crate is re-exported by `get-size2` and is meant to be used through its `derive` feature.
//! See [`GetSize`](macro@GetSize) for the attribute reference.

use attribute_derive::{Attribute, FromAttr};
use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[derive(FromAttr, Default, Debug)]
#[attribute(ident = get_size)]
struct StructFieldAttribute {
    #[attribute(conflicts = [size_fn, ignore])]
    size: Option<usize>,
    #[attribute(conflicts = [size, ignore])]
    size_fn: Option<syn::Ident>,
    #[attribute(conflicts = [size, size_fn])]
    ignore: bool,
}

fn extract_ignored_generics_list(list: &Vec<syn::Attribute>) -> Vec<syn::PathSegment> {
    let mut collection = Vec::new();

    for attr in list {
        let mut list = extract_ignored_generics(attr);

        collection.append(&mut list);
    }

    collection
}

fn extract_ignored_generics(attr: &syn::Attribute) -> Vec<syn::PathSegment> {
    let mut collection = Vec::new();

    // Skip all attributes which do not belong to us.
    if !attr.meta.path().is_ident("get_size") {
        return collection;
    }

    // Make sure it is a list: #[get_size(...)]
    let Ok(list) = attr.meta.require_list() else {
        return collection;
    };

    // Parse the nested meta: #[get_size(ignore(...))] or #[get_size(ignore)]
    let _ = list.parse_nested_meta(|meta| {
        // Only handle `ignore`
        if !meta.path.is_ident("ignore") {
            return Ok(()); // Skip unrelated
        }

        // Handle the flag case: #[get_size(ignore)]
        if meta.input.is_empty() {
            // Do nothing – valid empty ignore
            return Ok(());
        }

        // Handle the list case: #[get_size(ignore(A, B))]
        meta.parse_nested_meta(|meta| {
            for segment in meta.path.segments {
                collection.push(segment);
            }
            Ok(())
        })?;

        Ok(())
    });

    collection
}

fn collect_all_ignored_generics(ast: &syn::DeriveInput) -> Vec<syn::PathSegment> {
    let mut ignored = extract_ignored_generics_list(&ast.attrs);

    match &ast.data {
        syn::Data::Struct(data_struct) => {
            for field in &data_struct.fields {
                ignored.extend(extract_ignored_generics_list(&field.attrs));
            }
        }
        syn::Data::Enum(data_enum) => {
            for variant in &data_enum.variants {
                ignored.extend(extract_ignored_generics_list(&variant.attrs));
                for field in &variant.fields {
                    ignored.extend(extract_ignored_generics_list(&field.attrs));
                }
            }
        }
        syn::Data::Union(_) => {}
    }

    ignored
}

// Add a bound `T: GetSize` to every type parameter T, unless we ignore it.
fn add_trait_bounds(mut generics: syn::Generics, ignored: &Vec<syn::PathSegment>) -> syn::Generics {
    for param in &mut generics.params {
        if let syn::GenericParam::Type(type_param) = param {
            let mut found = false;
            for ignored in ignored {
                if ignored.ident == type_param.ident {
                    found = true;
                    break;
                }
            }

            if found {
                continue;
            }

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
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error().into(),
    }
}

#[expect(clippy::too_many_lines, reason = "Needs refactoring")]
fn derive_get_size_impl(input: TokenStream) -> syn::Result<TokenStream> {
    // Construct a representation of Rust code as a syntax tree that we can manipulate.
    let ast: syn::DeriveInput = syn::parse(input)?;

    // The name of the struct.
    let name = &ast.ident;

    // Extract all generics we shall ignore.
    // let ignored = extract_ignored_generics_list(&ast.attrs);
    let ignored = collect_all_ignored_generics(&ast);

    // Add a bound `T: GetSize` to every type parameter T.
    let generics = add_trait_bounds(ast.generics, &ignored);

    // Extract the generics of the struct/enum.
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Traverse the parsed data to generate the individual parts of the function.
    match ast.data {
        syn::Data::Enum(data_enum) => {
            if data_enum.variants.is_empty() {
                // Empty enums are easy to implement.
                let generated = quote! {
                    impl ::get_size2::GetSize for #name {}
                };
                return Ok(generated.into());
            }

            let mut cmds = Vec::with_capacity(data_enum.variants.len());

            for variant in data_enum.variants {
                let ident = &variant.ident;

                match &variant.fields {
                    syn::Fields::Unnamed(unnamed_fields) => {
                        let num_fields = unnamed_fields.unnamed.len();

                        let mut field_idents = Vec::with_capacity(num_fields);
                        let mut field_cmds = Vec::with_capacity(num_fields);

                        for (i, field) in unnamed_fields.unnamed.iter().enumerate() {
                            // Parse all relevant attributes.
                            let attr = StructFieldAttribute::from_attributes(&field.attrs)
                                .map_err(|err| syn::Error::new_spanned(field, err.to_string()))?;

                            // Fields handled by `size` or `ignore` are never read, so they are
                            // bound to a wildcard to avoid unused variable warnings.
                            if let Some(size) = attr.size {
                                field_idents.push(quote! { _ });
                                field_cmds.push(quote! {
                                    total += #size;
                                });

                                continue;
                            } else if attr.ignore {
                                field_idents.push(quote! { _ });

                                continue;
                            }

                            let field_ident = format_ident!("v{i}");

                            if let Some(size_fn) = attr.size_fn {
                                field_cmds.push(quote! {
                                    total += #size_fn(#field_ident);
                                });
                            } else {
                                field_cmds.push(quote! {
                                    let (total_add, tracker) = ::get_size2::GetSize::get_heap_size_with_tracker(#field_ident, tracker);
                                    total += total_add;
                                });
                            }

                            field_idents.push(quote! { #field_ident });
                        }

                        cmds.push(quote! {
                            Self::#ident(#(#field_idents,)*) => {
                                let mut total = 0;

                                #(#field_cmds)*;

                                (total, tracker)
                            }
                        });
                    }
                    syn::Fields::Named(named_fields) => {
                        let mut field_idents = Vec::new();
                        let mut field_cmds = Vec::new();
                        let mut skipped_field = false;

                        for field in &named_fields.named {
                            let field_ident = field.ident.as_ref().ok_or_else(|| {
                                syn::Error::new_spanned(field, "Expected named field")
                            })?;

                            let attr = StructFieldAttribute::from_attributes(&field.attrs)
                                .map_err(|err| syn::Error::new_spanned(field, err.to_string()))?;

                            // Fields handled by `size` or `ignore` are never read, so they stay
                            // out of the pattern and are covered by its `..` rest pattern.
                            if let Some(size) = attr.size {
                                skipped_field = true;
                                field_cmds.push(quote! {
                                    total += #size;
                                });

                                continue;
                            } else if attr.ignore {
                                skipped_field = true;

                                continue;
                            }

                            field_idents.push(field_ident);

                            if let Some(size_fn) = attr.size_fn {
                                field_cmds.push(quote! {
                                    total += #size_fn(#field_ident);
                                });
                            } else {
                                field_cmds.push(quote! {
                                    let (total_add, tracker) = ::get_size2::GetSize::get_heap_size_with_tracker(#field_ident, tracker);
                                    total += total_add;
                                });
                            }
                        }

                        let pattern = if skipped_field {
                            quote! { Self::#ident { #(#field_idents,)* .. } }
                        } else {
                            quote! { Self::#ident { #(#field_idents,)* } }
                        };

                        cmds.push(quote! {
                            #pattern => {
                                let mut total = 0;
                                #(#field_cmds)*
                                (total, tracker)
                            }
                        });
                    }

                    syn::Fields::Unit => {
                        cmds.push(quote! {
                            Self::#ident => (0, tracker),
                        });
                    }
                }
            }

            // Build the trait implementation
            let generated = quote! {
                impl #impl_generics ::get_size2::GetSize for #name #ty_generics #where_clause {
                    fn get_heap_size(&self) -> usize {
                        let tracker = ::get_size2::default_tracker();

                        let (total, _) = ::get_size2::GetSize::get_heap_size_with_tracker(self, tracker);

                        total
                    }

                    fn get_heap_size_with_tracker<TRACKER: ::get_size2::GetSizeTracker>(
                        &self,
                        tracker: TRACKER,
                    ) -> (usize, TRACKER) {
                        match self {
                            #(#cmds)*
                        }
                    }
                }
            };
            Ok(generated.into())
        }
        syn::Data::Union(_data_union) => Err(syn::Error::new_spanned(
            name,
            "Deriving GetSize for unions is currently not supported.",
        )),
        syn::Data::Struct(data_struct) => {
            if data_struct.fields.is_empty() {
                // Empty structs are easy to implement.
                let generated = quote! {
                    impl ::get_size2::GetSize for #name {}
                };
                return Ok(generated.into());
            }

            let mut cmds = Vec::with_capacity(data_struct.fields.len());

            let mut unidentified_fields_count = 0; // For newtypes

            for field in &data_struct.fields {
                // Parse all relevant attributes.
                let attr = StructFieldAttribute::from_attributes(&field.attrs)
                    .map_err(|err| syn::Error::new_spanned(field, err.to_string()))?;

                // How this field is accessed: by name, or by position for tuple structs. The
                // positional counter has to advance even when the field is handled by one of the
                // attributes below, otherwise all following fields of a tuple struct would be
                // read at the wrong position.
                let accessor = field.ident.as_ref().map_or_else(
                    || {
                        let index = syn::Index::from(unidentified_fields_count);
                        unidentified_fields_count += 1;
                        quote! { #index }
                    },
                    |ident| quote! { #ident },
                );

                if let Some(size) = attr.size {
                    cmds.push(quote! {
                        total += #size;
                    });

                    continue;
                } else if let Some(size_fn) = attr.size_fn {
                    cmds.push(quote! {
                        total += #size_fn(&self.#accessor);
                    });

                    continue;
                } else if attr.ignore {
                    continue;
                }

                cmds.push(quote! {
                    let (total_add, tracker) = ::get_size2::GetSize::get_heap_size_with_tracker(&self.#accessor, tracker);
                    total += total_add;
                });
            }

            // Build the trait implementation
            let generated = quote! {
                impl #impl_generics ::get_size2::GetSize for #name #ty_generics #where_clause {
                    fn get_heap_size(&self) -> usize {
                        let tracker = ::get_size2::default_tracker();

                        let (total, _) = ::get_size2::GetSize::get_heap_size_with_tracker(self, tracker);

                        total
                    }

                    fn get_heap_size_with_tracker<TRACKER: ::get_size2::GetSizeTracker>(
                        &self,
                        tracker: TRACKER,
                    ) -> (usize, TRACKER) {
                        let mut total = 0;

                        #(#cmds)*;

                        (total, tracker)
                    }
                }
            };
            Ok(generated.into())
        }
    }
}
