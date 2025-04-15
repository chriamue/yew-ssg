use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput, Expr, LitStr};

/// # LocalizedRoutable Derive Macro
///
/// This macro automatically implements the `LocalizedRoutable` trait for a route enum.
/// It creates a wrapper enum (by default named `Localized<YourEnum>`) that handles
/// both default routes and routes with language prefixes.
///
/// ## Usage
///
/// ```ignore
/// use yew_ssg_router::prelude::*;
/// use strum_macros::EnumIter;
/// use yew_ssg_router_macros::LocalizedRoutable;
///
/// #[derive(Clone, Routable, PartialEq, Debug, EnumIter, Default, LocalizedRoutable)]
/// #[localized(
///     languages = ["en", "de", "fr"],
///     default = "en",
///     wrapper = "LocalizedRoute"
/// )]
/// pub enum Route {
///     #[at("/")]
///     #[default]
///     Home,
///     #[at("/about")]
///     About,
/// }
/// ```
///
/// ## Configuration Options
///
/// The `#[localized(...)]` attribute accepts the following parameters:
///
/// - `languages`: Array of supported language codes (default: `["en"]`)
/// - `default`: Default language code (default: `"en"`)
/// - `wrapper`: Name for the generated wrapper enum (default: `"Localized<YourEnum>"`)
///
/// ## Generated Implementation
///
/// The macro generates:
///
/// 1. A wrapper enum with two variants:
///   - `Default(YourEnum)` - For routes without language prefix
///   - `Localized { lang: String, route: YourEnum }` - For localized routes
///
/// 2. Implementations of `LocalizedRoutable` and `Routable` traits for the wrapper
///
/// 3. IntoEnumIterator implementation for integration with `strum`

#[proc_macro_derive(LocalizedRoutable, attributes(localized))]
pub fn derive_localized_routable(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Extract the name of the base route enum
    let base_route_name = &input.ident;

    // Default values
    let mut wrapper_name = format_ident!("Localized{}", base_route_name);
    let mut languages = vec!["en".to_string()]; // Default language
    let mut default_language = "en".to_string();

    // Look for #[localized(...)] attributes to customize the implementation
    for attr in &input.attrs {
        if attr.path().is_ident("localized") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("languages") {
                    // Parse languages array
                    if let Ok(expr) = meta.value()?.parse::<Expr>() {
                        if let Expr::Array(expr_array) = expr {
                            // Clear default and add parsed languages
                            languages.clear();

                            for expr in expr_array.elems {
                                if let Expr::Lit(lit_expr) = expr {
                                    if let syn::Lit::Str(lit_str) = lit_expr.lit {
                                        languages.push(lit_str.value());
                                    }
                                }
                            }
                        }
                    }
                } else if meta.path.is_ident("default") {
                    // Parse default language
                    if let Ok(lit) = meta.value()?.parse::<LitStr>() {
                        default_language = lit.value();
                    }
                } else if meta.path.is_ident("wrapper") {
                    // Parse wrapper name
                    if let Ok(lit) = meta.value()?.parse::<LitStr>() {
                        wrapper_name = format_ident!("{}", lit.value());
                    }
                }
                Ok(())
            })
            .unwrap_or_else(|err| {
                // Don't fail the whole compilation, just emit the error
                panic!("Error parsing localized attribute: {}", err);
            });
        }
    }

    // Generate string literals for the languages
    let lang_literals = languages.iter().map(|lang| {
        let lang_str = lang.as_str();
        quote! { #lang_str }
    });

    // Generate the default language as string literal
    let default_lang_lit = default_language.as_str();

    // Generate the implementation
    let expanded = quote! {
        // Define the localized route enum
        #[derive(Clone, PartialEq, Debug)]
        pub enum #wrapper_name {
            Default(#base_route_name),
            Localized { lang: String, route: #base_route_name },
        }

        impl Default for #wrapper_name {
            fn default() -> Self {
                // Try to get a default route if the base type implements Default
                #wrapper_name::Default(<#base_route_name as Default>::default())
            }
        }

        impl LocalizedRoutable for #wrapper_name {
            type BaseRoute = #base_route_name;

            fn get_lang(&self) -> Option<String> {
                match self {
                    Self::Default(_) => None,
                    Self::Localized { lang, .. } => Some(Self::validate_lang(lang)),
                }
            }

            fn get_route(&self) -> &Self::BaseRoute {
                match self {
                    Self::Default(route) => route,
                    Self::Localized { route, .. } => route,
                }
            }

            fn from_route(route: Self::BaseRoute, lang: Option<&str>) -> Self {
                match lang {
                    Some(lang) if Self::supported_languages().contains(&lang) => {
                        Self::Localized {
                            lang: lang.to_string(),
                            route,
                        }
                    }
                    _ => Self::Default(route),
                }
            }

            fn validate_lang(lang: &str) -> String {
                if Self::supported_languages().contains(&lang) {
                    lang.to_string()
                } else {
                    Self::default_language().to_string()
                }
            }

            fn supported_languages() -> &'static [&'static str] {
                &[#(#lang_literals),*]
            }

            fn default_language() -> &'static str {
                #default_lang_lit
            }
        }

        impl Routable for #wrapper_name {
            fn from_path(path: &str, params: &std::collections::HashMap<&str, &str>) -> Option<Self> {
                let (lang, remaining_path) = Self::extract_lang_from_path(path);

                if let Some(lang) = lang {
                    // This is a localized route
                    <#base_route_name as Routable>::from_path(&remaining_path, params)
                        .map(|route| Self::Localized { lang, route })
                } else {
                    // This is a default route
                    <#base_route_name as Routable>::from_path(path, params)
                        .map(Self::Default)
                }
            }

            fn to_path(&self) -> String {
                match self {
                    Self::Default(route) => route.to_path(),
                    Self::Localized { lang, route } => {
                        let route_path = route.to_path();
                        if route_path == "/" {
                            format!("/{}", lang)
                        } else {
                            format!("/{}{}", lang, route_path)
                        }
                    }
                }
            }

            fn routes() -> Vec<&'static str> {
                let mut routes = <#base_route_name as Routable>::routes();

                // Add language prefixed routes
                for &lang in Self::supported_languages() {
                    for route in <#base_route_name as Routable>::routes() {
                        let localized_route = if route == "/" {
                            format!("/{}", lang)
                        } else {
                            format!("/{}{}", lang, route)
                        };

                        // Leak to get static lifetime
                        let localized_route: &'static str = Box::leak(localized_route.into_boxed_str());
                        routes.push(localized_route);
                    }
                }

                routes
            }

            fn not_found_route() -> Option<Self> {
                <#base_route_name as Routable>::not_found_route()
                    .map(Self::Default)
            }

            fn recognize(pathname: &str) -> Option<Self> {
                let (lang, remaining_path) = Self::extract_lang_from_path(pathname);

                if let Some(lang) = lang {
                    // This is a localized route
                    <#base_route_name as Routable>::recognize(&remaining_path)
                        .map(|route| Self::Localized { lang, route })
                } else {
                    // This is a default route
                    <#base_route_name as Routable>::recognize(pathname)
                        .map(Self::Default)
                }
            }
        }

        impl IntoEnumIterator for #wrapper_name
        where
            #base_route_name: IntoEnumIterator + Clone,
        {
            type Iterator = LocalizedRouteIter<#base_route_name, Self>;

            fn iter() -> Self::Iterator {
                LocalizedRouteIter::new(Self::supported_languages())
            }
        }
    };

    // Return the generated implementation
    TokenStream::from(expanded)
}
