#![doc = include_str!("./lib.md")]

use attribute_derive::{Attribute, FromAttr};
use proc_macro::TokenStream;
use quote::quote;

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

    // Make sure it is a list.
    let Ok(list) = attr.meta.require_list() else {
        return collection;
    };

    // Parse the nested meta.
    // #[get_size(ignore(A, B))]
    list.parse_nested_meta(|meta| {
        // We only parse the ignore attributes.
        if !meta.path.is_ident("ignore") {
            return Ok(()); // Just skip.
        }

        meta.parse_nested_meta(|meta| {
            for segment in meta.path.segments {
                collection.push(segment);
            }

            Ok(())
        })?;

        Ok(())
    })
    .expect("Could not parse the ignore list.");

    collection
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

#[expect(
    clippy::too_many_lines,
    clippy::missing_panics_doc,
    reason = "Needs refactoring"
)]
#[proc_macro_derive(GetSize, attributes(get_size))]
pub fn derive_get_size(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast: syn::DeriveInput = syn::parse(input).expect("Could not parse tokens");

    // The name of the sruct.
    let name = &ast.ident;

    // Extract all generics we shall ignore.
    let ignored = extract_ignored_generics_list(&ast.attrs);

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
                return generated.into();
            }

            let mut cmds = Vec::with_capacity(data_enum.variants.len());

            for variant in data_enum.variants {
                let ident = &variant.ident;

                match &variant.fields {
                    syn::Fields::Unnamed(unnamed_fields) => {
                        let num_fields = unnamed_fields.unnamed.len();

                        let mut field_idents = Vec::with_capacity(num_fields);
                        for i in 0..num_fields {
                            let field_ident = String::from("v") + &i.to_string();
                            let field_ident = syn::parse_str::<syn::Ident>(&field_ident)
                                .expect("Could not parse string to ident.");

                            field_idents.push(field_ident);
                        }

                        let mut field_cmds = Vec::with_capacity(num_fields);

                        for (i, _field) in unnamed_fields.unnamed.iter().enumerate() {
                            let field_ident = String::from("v") + &i.to_string();
                            let field_ident = syn::parse_str::<syn::Ident>(&field_ident)
                                .expect("Could not parse string to ident.");

                            field_cmds.push(quote! {
                                let (total_add, tracker) = ::get_size2::GetSize::get_heap_size_with_tracker(#field_ident, tracker);
                                total += total_add;
                            });
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
                        let num_fields = named_fields.named.len();

                        let mut field_idents = Vec::with_capacity(num_fields);

                        let mut field_cmds = Vec::with_capacity(num_fields);

                        for field in &named_fields.named {
                            let field_ident =
                                field.ident.as_ref().expect("Could not get field ident.");

                            field_idents.push(field_ident);

                            field_cmds.push(quote! {
                                let (total_add, tracker) = ::get_size2::GetSize::get_heap_size_with_tracker(#field_ident, tracker);
                                total += total_add;
                            });
                        }

                        cmds.push(quote! {
                            Self::#ident{#(#field_idents,)*} => {
                                let mut total = 0;

                                #(#field_cmds)*;

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
                        let tracker = get_size2::StandardTracker::default();

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
            generated.into()
        }
        syn::Data::Union(_data_union) => {
            panic!("Deriving GetSize for unions is currently not supported.")
        }
        syn::Data::Struct(data_struct) => {
            if data_struct.fields.is_empty() {
                // Empty structs are easy to implement.
                let generated = quote! {
                    impl ::get_size2::GetSize for #name {}
                };
                return generated.into();
            }

            let mut cmds = Vec::with_capacity(data_struct.fields.len());

            let mut unidentified_fields_count = 0; // For newtypes

            for field in &data_struct.fields {
                // Parse all relevant attributes.
                let attr = StructFieldAttribute::from_attributes(&field.attrs)
                    .expect("Could not parse attributes.");

                // NOTE There will be no attributes if this is a tuple struct.
                if let Some(size) = attr.size {
                    cmds.push(quote! {
                        total += #size;
                    });

                    continue;
                } else if let Some(size_fn) = attr.size_fn {
                    let ident = field.ident.as_ref().expect("Could not get field ident.");

                    cmds.push(quote! {
                        total += #size_fn(&self.#ident);
                    });

                    continue;
                } else if attr.ignore {
                    continue;
                }

                if let Some(ident) = field.ident.as_ref() {
                    cmds.push(quote! {
                        let (total_add, tracker) = ::get_size2::GetSize::get_heap_size_with_tracker(&self.#ident, tracker);
                        total += total_add;
                    });
                } else {
                    let current_index = syn::Index::from(unidentified_fields_count);
                    cmds.push(quote! {
                        let (total_add, tracker) = ::get_size2::GetSize::get_heap_size_with_tracker(&self.#current_index, tracker);
                        total += total_add;
                    });

                    unidentified_fields_count += 1;
                }
            }

            // Build the trait implementation
            let generated = quote! {
                impl #impl_generics ::get_size2::GetSize for #name #ty_generics #where_clause {
                    fn get_heap_size(&self) -> usize {
                        let tracker = get_size2::StandardTracker::default();

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
            generated.into()
        }
    }
}
