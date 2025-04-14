use std::fmt::Debug;
use yew_router::Routable;

/// Extends `Routable` to support localized routes with language prefixes.
///
/// This trait adds language-specific functionality on top of Yew Router's
/// standard routing system. It allows for routes to be prefixed with language
/// codes (e.g., `/en/home`, `/de/about`) while maintaining compatibility with
/// the core routing system.
pub trait LocalizedRoutable: Routable + Debug {
    /// The base route type without localization
    type BaseRoute: Routable + Debug;

    /// Get the language code if this is a localized route
    ///
    /// Returns `None` for non-localized routes.
    fn get_lang(&self) -> Option<String>;

    /// Get the underlying base route without localization
    fn get_route(&self) -> &Self::BaseRoute;

    /// Create a localized route from a base route and optional language
    ///
    /// If language is `None` or invalid, the default route should be returned.
    fn from_route(route: Self::BaseRoute, lang: Option<&str>) -> Self;

    /// Validate and normalize a language code
    ///
    /// Should return a default language (e.g., "en") if the provided code is invalid.
    fn validate_lang(lang: &str) -> String;

    /// List all supported language codes
    fn supported_languages() -> &'static [&'static str];

    /// Get the default language to use when none is specified
    fn default_language() -> &'static str {
        "en"
    }

    /// Create a localized path from a base path and language
    fn localize_path(path: &str, lang: &str) -> String {
        if path == "/" {
            format!("/{}", lang)
        } else {
            format!("/{}{}", lang, path)
        }
    }

    /// Extract language from a path
    ///
    /// Returns (language, remaining_path)
    fn extract_lang_from_path(path: &str) -> (Option<String>, String) {
        let parts: Vec<&str> = path.trim_matches('/').split('/').collect();

        if parts.is_empty() {
            return (None, "/".to_string());
        }

        let potential_lang = parts[0];
        if Self::supported_languages().contains(&potential_lang) {
            let remaining = if parts.len() > 1 {
                format!("/{}", parts[1..].join("/"))
            } else {
                "/".to_string()
            };
            (Some(potential_lang.to_string()), remaining)
        } else {
            (None, path.to_string())
        }
    }
}
