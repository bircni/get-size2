//! Derives the `GetSize` trait of [`get-size2`](https://docs.rs/get-size2) for structs and enums.
//!
//! This crate is re-exported by `get-size2` and is meant to be used through its `derive` feature.
//! See [`GetSize`](macro@GetSize) for the attribute reference.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// How the heap size of a single field is determined.
enum FieldSize {
    /// `#[get_size(size = ...)]`: a fixed number of bytes.
    Fixed(syn::LitInt),
    /// `#[get_size(size_fn = ...)]`: the result of calling a helper with a reference to the field.
    Helper(syn::ExprPath),
    /// `#[get_size(ignore)]`: nothing at all.
    Ignored,
    /// No attribute: the field's own [`GetSize`] implementation.
    Measured,
}

impl FieldSize {
    /// Parses the `#[get_size(...)]` attributes of a field.
    fn of(field: &syn::Field) -> syn::Result<Self> {
        let mut size = None;

        for attr in &field.attrs {
            // Skip all attributes which do not belong to us. A bare `#[get_size]` asks for
            // nothing.
            if !attr.path().is_ident("get_size") || matches!(attr.meta, syn::Meta::Path(_)) {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                let parsed = Self::parse(&meta)?;

                // The three ways to handle a field exclude each other.
                if size.replace(parsed).is_some() {
                    return Err(meta.error(
                        "`size`, `size_fn` and `ignore` are mutually exclusive, \
                         only one of them may be given once",
                    ));
                }

                Ok(())
            })?;
        }

        Ok(size.unwrap_or(Self::Measured))
    }

    /// Parses a single `key`, `key = value` or `key(value)` of a field attribute.
    fn parse(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<Self> {
        if meta.path.is_ident("size") {
            // The literal is checked to be a plain integer, but handed on as written: the
            // compiler then reports one which does not fit the `usize` of the target, which may
            // be narrower than the one of the host this macro runs on.
            return Self::value::<syn::LitInt>(meta)
                .ok()
                // Any other suffix would have to be dropped, turning `0XFF` into 0.
                .filter(|size| matches!(size.suffix(), "" | "usize"))
                .filter(|size| size.base10_parse::<u128>().is_ok())
                .map(Self::Fixed)
                .ok_or_else(|| {
                    meta.error(
                        "`size` expects an integer literal for the number of bytes, without a \
                         suffix or with `usize`, like `size = 1024`",
                    )
                });
        }

        if meta.path.is_ident("size_fn") {
            // Parsed as an expression, not as a type like path, so generic arguments need the
            // turbofish, exactly as they do when the helper is called by hand.
            return Self::value::<syn::ExprPath>(meta)
                .map(Self::Helper)
                .map_err(|_| {
                    meta.error(
                        "`size_fn` expects the name of or the path to a function, with generic \
                         arguments written as `::<T>`",
                    )
                });
        }

        if meta.path.is_ident("ignore") {
            const VALUE: &str = "`ignore` expects no value, or `true` or `false`";

            // A plain `ignore` is the common spelling. `ignore = false` leaves the field to be
            // measured as usual.
            let ignore = if meta.input.peek(syn::Token![=]) {
                Self::value::<syn::LitBool>(meta).map_err(|_| meta.error(VALUE))?
            } else if meta.input.peek(syn::token::Paren) {
                let content;
                syn::parenthesized!(content in meta.input);

                // Anything but a boolean is most likely meant as a list of generics.
                if !content.is_empty() && !content.peek(syn::LitBool) {
                    return Err(meta.error(
                        "a list of generic parameters to ignore belongs on the struct or enum, \
                         a field only takes a plain `ignore`",
                    ));
                }

                content
                    .parse::<syn::LitBool>()
                    .ok()
                    .filter(|_| content.is_empty())
                    .ok_or_else(|| meta.error(VALUE))?
            } else {
                return Ok(Self::Ignored);
            };

            return Ok(if ignore.value {
                Self::Ignored
            } else {
                Self::Measured
            });
        }

        Err(meta.error("the supported field attributes are `size`, `size_fn` and `ignore`"))
    }

    /// Parses the value of a `key = value` or of the equivalent `key(value)`, which has to be
    /// followed by the next key or the end of the attribute.
    fn value<T: syn::parse::Parse>(meta: &syn::meta::ParseNestedMeta<'_>) -> syn::Result<T> {
        if meta.input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in meta.input);
            let value = content.parse()?;

            return if content.is_empty() {
                Ok(value)
            } else {
                Err(content.error("unexpected token"))
            };
        }

        let input = meta.value()?;
        let value = input.parse()?;

        if input.is_empty() || input.peek(syn::Token![,]) {
            Ok(value)
        } else {
            Err(input.error("unexpected token"))
        }
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

/// Names a local variable of the generated code.
///
/// Two things keep it apart from the names of the user. The mixed site span gives it the hygiene
/// of a `macro_rules!` local, so it is neither captured by a field bound in an enum variant
/// pattern nor does it shadow a helper function. That hygiene does not cover items though: a
/// constant or static of the same name would turn the binding into a pattern. The prefix makes
/// such a clash as unlikely as it gets.
fn local(name: &str) -> syn::Ident {
    syn::Ident::new(
        &format!("__get_size_{name}"),
        proc_macro2::Span::mixed_site(),
    )
}

/// Generates the body which adds up the heap size of `fields` and returns it with the tracker.
///
/// `reference` builds the expression which evaluates to a reference to the given field.
fn sum_fields(fields: &[Field], reference: impl Fn(usize, &Field) -> TokenStream2) -> TokenStream2 {
    let total = local("total");
    let tracker = local("tracker");
    let field_size = local("field_size");

    let additions = fields
        .iter()
        .enumerate()
        .filter_map(|(index, field)| match &field.size {
            FieldSize::Fixed(size) => Some(quote! {
                let #total = #total + #size;
            }),
            FieldSize::Helper(size_fn) => {
                let reference = reference(index, field);
                Some(quote! {
                    let #total = #total + #size_fn(#reference);
                })
            }
            FieldSize::Ignored => None,
            FieldSize::Measured => {
                let reference = reference(index, field);
                Some(quote! {
                    let (#field_size, #tracker) =
                        ::get_size2::GetSize::get_heap_size_with_tracker(#reference, #tracker);
                    let #total = #total + #field_size;
                })
            }
        });

    quote! {
        let #total: ::core::primitive::usize = 0;
        #(#additions)*
        (#total, #tracker)
    }
}

/// Generates the match arm measuring a single enum variant.
fn variant_arm(variant: &syn::Variant) -> syn::Result<TokenStream2> {
    let ident = &variant.ident;
    let fields = Field::collect(&variant.fields)?;

    let binding = |index: usize| local(&format!("field_{index}"));

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

    // A bare `#[get_size]` asks for nothing, on a field as much as here.
    if matches!(attr.meta, syn::Meta::Path(_)) {
        return Ok(());
    }

    attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("ignore") && meta.input.peek(syn::token::Paren) {
            // The list may be empty, which generated code can easily produce.
            let content;
            syn::parenthesized!(content in meta.input);
            let generics = content
                .parse_terminated(<syn::Ident as syn::parse::Parse>::parse, syn::Token![,])?;

            ignored.extend(generics);
            return Ok(());
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
/// The list is usually given on the struct or enum itself. One on a variant is accepted as well
/// and applies to the whole enum, as there is only one implementation to bound.
///
/// Fields are not searched: their attributes are parsed by [`FieldSize::of`], which only knows
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
    match expand(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

/// Generates the `GetSize` implementation for the type definition in `input`.
///
/// Works on `proc_macro2` tokens, so it can be called from the unit tests, where the compiler's
/// own token types are not available.
fn expand(input: TokenStream2) -> syn::Result<TokenStream2> {
    // Construct a representation of Rust code as a syntax tree that we can manipulate.
    let ast: syn::DeriveInput = syn::parse2(input)?;

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

    let tracker = local("tracker");
    let method = body.map(|body| {
        quote! {
            fn get_heap_size_with_tracker<__GetSizeTracker: ::get_size2::GetSizeTracker>(
                &self,
                #tracker: __GetSizeTracker,
            ) -> (::core::primitive::usize, __GetSizeTracker) {
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

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::{TokenStream2, expand};

    /// The message of the compile error `input` is rejected with.
    fn error(input: TokenStream2) -> String {
        match expand(input) {
            Ok(tokens) => panic!("expected an error, but got: {tokens}"),
            Err(err) => err.to_string(),
        }
    }

    /// The generated implementation, with all whitespace removed to be independent of how
    /// tokens are printed.
    fn expanded(input: TokenStream2) -> String {
        match expand(input) {
            Ok(tokens) => tokens.to_string().split_whitespace().collect(),
            Err(err) => panic!("expected an implementation, but got the error: {err}"),
        }
    }

    const MISPLACED: &str = "only `ignore(...)` with a list of generic parameters is supported \
                             here, `ignore`, `size` and `size_fn` belong on a field";

    #[test]
    fn rejects_field_attributes_on_containers() {
        for attribute in [
            quote! { ignore },
            quote! { size = 5 },
            quote! { size_fn = helper },
            quote! { bogus },
            quote! { skip(T) },
            quote! { ignore = "T" },
        ] {
            assert_eq!(
                error(quote! { #[get_size(#attribute)] struct Test<T> { value: T } }),
                MISPLACED,
                "struct level `{attribute}`"
            );
            assert_eq!(
                error(quote! { enum Test<T> { #[get_size(#attribute)] Variant(T) } }),
                MISPLACED,
                "variant level `{attribute}`"
            );
        }
    }

    #[test]
    fn rejects_unknown_ignored_generics() {
        assert_eq!(
            error(quote! { #[get_size(ignore(U))] struct Test<T> { value: T } }),
            "`U` is not a generic type parameter of `Test`"
        );
        assert_eq!(
            error(quote! { #[get_size(ignore(T))] struct Test<'T> { value: &'T str } }),
            "`T` is not a generic type parameter of `Test`"
        );
        assert_eq!(
            error(quote! { enum Test<T> { #[get_size(ignore(U))] Variant(T) } }),
            "`U` is not a generic type parameter of `Test`"
        );
    }

    #[test]
    fn rejects_unions() {
        assert_eq!(
            error(quote! { union Test { value: u8 } }),
            "Deriving GetSize for unions is currently not supported."
        );
    }

    #[test]
    fn rejects_conflicting_and_malformed_field_attributes() {
        const EXCLUSIVE: &str = "`size`, `size_fn` and `ignore` are mutually exclusive, only one \
                                 of them may be given once";
        const SIZE: &str = "`size` expects an integer literal for the number of bytes, without \
                            a suffix or with `usize`, like `size = 1024`";
        const IGNORE: &str = "`ignore` expects no value, or `true` or `false`";
        const SIZE_FN: &str = "`size_fn` expects the name of or the path to a function, with \
                               generic arguments written as `::<T>`";

        for (attribute, message) in [
            (quote! { size = 1, ignore }, EXCLUSIVE),
            (quote! { size = 1, size_fn = helper }, EXCLUSIVE),
            (quote! { size_fn = helper, ignore }, EXCLUSIVE),
            (quote! { ignore, ignore }, EXCLUSIVE),
            (quote! { size = 1, size = 1 }, EXCLUSIVE),
            (
                quote! { bogus },
                "the supported field attributes are `size`, `size_fn` and `ignore`",
            ),
            (quote! { size = "one" }, SIZE),
            (quote! { size }, SIZE),
            (quote! { size = -1 }, SIZE),
            (quote! { size = 0XFF }, SIZE),
            (quote! { size = 300u8 }, SIZE),
            (quote! { size = 1.5 }, SIZE),
            (quote! { size = 1u64 }, SIZE),
            (quote! { size = BYTES }, SIZE),
            (quote! { size = 1 + 1 }, SIZE),
            (
                quote! { size = 9999999999999999999999999999999999999999 },
                SIZE,
            ),
            (quote! { size(1, 2) }, SIZE),
            (quote! { size_fn(helper, other) }, SIZE_FN),
            (quote! { size_fn }, SIZE_FN),
            (
                quote! { ignore(T) },
                "a list of generic parameters to ignore belongs on the struct or enum, a field \
                 only takes a plain `ignore`",
            ),
            (quote! { ignore = 1 }, IGNORE),
            (quote! { ignore() }, IGNORE),
            (quote! { ignore(true, false) }, IGNORE),
            (quote! { ignore(true,) }, IGNORE),
            (quote! { size_fn = helper() }, SIZE_FN),
            (quote! { size_fn = helper<u8> }, SIZE_FN),
            (quote! { size_fn = |value| 0 }, SIZE_FN),
            (quote! { size_fn = "helper" }, SIZE_FN),
        ] {
            assert_eq!(
                error(quote! { struct Test { #[get_size(#attribute)] value: u8 } }),
                message,
                "field level `{attribute}`"
            );
        }

        // Spread over several attributes they conflict just the same.
        assert_eq!(
            error(quote! {
                struct Test { #[get_size(size = 1)] #[get_size(ignore)] value: u8 }
            }),
            EXCLUSIVE
        );
    }

    #[test]
    fn accepts_the_lenient_spellings() {
        // A bare attribute and an empty list ask for nothing.
        assert_eq!(
            expanded(quote! { #[get_size] #[get_size()] #[get_size(ignore())] struct Test<T>; }),
            "impl<T:::get_size2::GetSize>::get_size2::GetSizeforTest<T>{}"
        );

        let implementation = expanded(quote! {
            struct Test {
                #[get_size]
                bare: u8,
                #[get_size()]
                empty: u8,
                #[get_size(ignore = false)]
                unignored: u8,
                #[get_size(ignore = true)]
                dropped: u8,
                #[get_size(size = 0x10usize,)]
                fixed: u8,
                // The parenthesised spellings are equivalent.
                #[get_size(size(0b11))]
                parenthesised: u8,
                #[get_size(size_fn(helpers::len))]
                helped: u8,
                #[get_size(ignore(true))]
                skipped: u8,
                #[get_size(ignore(false))]
                retained: u8,
                #[other(attribute)]
                foreign: u8,
            }
        });

        for measured in [
            "&self.bare",
            "&self.empty",
            "&self.unignored",
            "&self.retained",
            "&self.foreign",
        ] {
            assert!(
                implementation.contains(measured),
                "{measured}: {implementation}"
            );
        }
        assert!(!implementation.contains("self.dropped"), "{implementation}");
        assert!(!implementation.contains("self.skipped"), "{implementation}");
        assert!(implementation.contains("+0x10usize;"), "{implementation}");
        assert!(implementation.contains("+0b11;"), "{implementation}");
        assert!(
            implementation.contains("helpers::len(&self.helped)"),
            "{implementation}"
        );
    }

    #[test]
    fn ignored_generics_receive_no_bound() {
        let implementation = expanded(quote! {
            #[get_size(ignore(B, N))]
            struct Test<A, B, const N: usize> {
                first: A,
                #[get_size(ignore)]
                second: [B; N],
            }
        });

        assert!(
            implementation.starts_with(
                "impl<A:::get_size2::GetSize,B,constN:usize>::get_size2::GetSizeforTest<A,B,N>{"
            ),
            "{implementation}"
        );
        assert!(implementation.contains("&self.first"), "{implementation}");
        assert!(!implementation.contains("&self.second"), "{implementation}");
    }

    #[test]
    fn fieldless_types_keep_their_generics() {
        assert_eq!(
            expanded(quote! { struct Marker<const N: usize>; }),
            "impl<constN:usize>::get_size2::GetSizeforMarker<N>{}"
        );
        assert_eq!(
            expanded(quote! { enum Never<T> {} }),
            "impl<T:::get_size2::GetSize>::get_size2::GetSizeforNever<T>{}"
        );
    }

    #[test]
    fn field_attributes_select_how_a_field_is_measured() {
        let implementation = expanded(quote! {
            struct Test {
                measured: String,
                #[get_size(size = 7)]
                fixed: String,
                #[get_size(size_fn = helpers::len::<String>)]
                helped: String,
                #[get_size(ignore)]
                ignored: String,
            }
        });

        assert!(
            implementation.contains("&self.measured"),
            "{implementation}"
        );
        assert!(implementation.contains("+7;"), "{implementation}");
        assert!(
            implementation.contains("helpers::len::<String>(&self.helped)"),
            "{implementation}"
        );
        assert!(!implementation.contains("self.fixed"), "{implementation}");
        assert!(!implementation.contains("self.ignored"), "{implementation}");
    }

    #[test]
    fn variants_only_bind_the_fields_they_read() {
        let implementation = expanded(quote! {
            enum Test {
                Unit,
                Tuple(#[get_size(ignore)] u8, String),
                Named { tracker: String, #[get_size(size = 3)] fixed: u8 },
            }
        });

        assert!(
            implementation.contains("Self::Unit{..}=>"),
            "{implementation}"
        );
        assert!(
            implementation.contains("Self::Tuple{1:__get_size_field_1,..}=>"),
            "{implementation}"
        );
        assert!(
            implementation.contains("Self::Named{tracker:__get_size_field_0,..}=>"),
            "{implementation}"
        );
    }
}
