//! Derive macros for [`rtformat`](https://docs.rs/rtformat).

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

/// Derives `rtformat::FormatArg`, adapting to whichever of `Display` and
/// `Debug` the type implements — implementing either one is enough.
///
/// # Routing
///
/// - `{}` renders via `Display` when available, falling back to `Debug`.
/// - `{:?}` and `{:#?}` render via `Debug` (pretty in the `#` case).
/// - Any other format type is rejected with `Err(fmt::Error)` (surfaced as
///   `rtformat::FormatError::UnsupportedFormatType`), as is `{}` on a type
///   implementing neither trait and `{:?}` on a `Display`-only type.
///
/// The dispatch is resolved statically at compile time through
/// inherent-vs-trait method priority, so there is no runtime cost to the
/// adaptation and a missing trait surfaces as a runtime
/// `UnsupportedFormatType` error rather than a compile failure.
///
/// # Generic types
///
/// Every type parameter is additionally bound by `Display + Debug`
/// (serde-style): trait availability on `Self` must be known when the
/// derived impl is checked, which only holds if the parameters provide it.
///
/// # Examples
///
/// ```ignore
/// use core::fmt;
/// use rtformat::{rformat, Format, FormatArg};
///
/// #[derive(Debug, FormatArg)]
/// struct Color(u8, u8, u8);
///
/// impl fmt::Display for Color {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "#{:02x}{:02x}{:02x}", self.0, self.1, self.2)
///     }
/// }
///
/// assert_eq!(rformat!("{}", Color(255, 128, 0)), "#ff8000");
/// assert_eq!(rformat!("{:?}", Color(255, 128, 0)), "Color(255, 128, 0)");
/// ```
#[proc_macro_derive(FormatArg)]
pub fn derive_format_arg(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let mut generics = input.generics.clone();
    for param in generics.type_params_mut() {
        param.bounds.push(syn::parse_quote!(::core::fmt::Display));
        param.bounds.push(syn::parse_quote!(::core::fmt::Debug));
    }
    let (impl_generics, _, _) = generics.split_for_impl();
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();
    let expanded = quote! {
        impl #impl_generics ::rtformat::FormatArg for #name #ty_generics #where_clause {
            fn write(
                &self,
                ty: ::rtformat::FormatType,
                pretty: bool,
                _precision: ::core::option::Option<::core::primitive::usize>,
                f: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::fmt::Result {
                #[allow(unused_imports)]
                use ::rtformat::__private::{DebugFallback as _, DisplayFallback as _};
                let wrap = ::rtformat::__private::Wrap(self);
                match ty {
                    ::rtformat::FormatType::Display => wrap
                        .fmt_display(f)
                        .or_else(|_| wrap.fmt_debug(f))
                        .unwrap_or(::core::result::Result::Err(::core::fmt::Error)),
                    ::rtformat::FormatType::Debug if pretty => wrap
                        .fmt_debug_pretty(f)
                        .unwrap_or(::core::result::Result::Err(::core::fmt::Error)),
                    ::rtformat::FormatType::Debug => wrap
                        .fmt_debug(f)
                        .unwrap_or(::core::result::Result::Err(::core::fmt::Error)),
                    _ => ::core::result::Result::Err(::core::fmt::Error),
                }
            }
        }
    };
    TokenStream::from(expanded)
}
