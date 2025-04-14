use std::collections::HashMap;
use std::iter::FusedIterator;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use yew_router::prelude::*;
use yew_router::LocalizedRoutable;

#[derive(Clone, Routable, PartialEq, Debug, EnumIter)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/about")]
    About,
    #[at("/readme")]
    Readme,
    #[at("/crate/:id")]
    Crate { id: String },
    #[not_found]
    #[at("/404")]
    NotFound,
}

#[derive(Clone, PartialEq, Debug)]
pub enum LocalizedRoute {
    Default(Route),
    Localized { lang: String, route: Route },
}

impl Default for LocalizedRoute {
    fn default() -> Self {
        LocalizedRoute::Default(Route::Home)
    }
}

pub const SUPPORTED_LANGUAGES: &[&str] = &["en", "de"];
pub const DEFAULT_LANGUAGE: &str = "en";

impl LocalizedRoutable for LocalizedRoute {
    type BaseRoute = Route;

    fn get_lang(&self) -> Option<String> {
        match self {
            LocalizedRoute::Default(_) => None,
            LocalizedRoute::Localized { lang, .. } => Some(Self::validate_lang(lang)),
        }
    }

    fn get_route(&self) -> &Self::BaseRoute {
        match self {
            LocalizedRoute::Default(route) => route,
            LocalizedRoute::Localized { route, .. } => route,
        }
    }

    fn from_route(route: Self::BaseRoute, lang: Option<&str>) -> Self {
        match lang {
            Some(lang) if Self::supported_languages().contains(&lang) => {
                LocalizedRoute::Localized {
                    lang: lang.to_string(),
                    route,
                }
            }
            _ => LocalizedRoute::Default(route),
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
        SUPPORTED_LANGUAGES
    }

    fn default_language() -> &'static str {
        DEFAULT_LANGUAGE
    }
}

impl Routable for LocalizedRoute {
    fn from_path(path: &str, params: &HashMap<&str, &str>) -> Option<Self> {
        let parts: Vec<&str> = path.trim_matches('/').split('/').collect();

        if parts.is_empty() {
            return Some(LocalizedRoute::Default(Route::Home));
        }

        let first_part = parts[0];
        if SUPPORTED_LANGUAGES.contains(&first_part) {
            // This is a localized route
            let lang = first_part.to_string();
            let remaining = if parts.len() > 1 {
                format!("/{}", parts[1..].join("/"))
            } else {
                "/".to_string()
            };

            Route::from_path(&remaining, params)
                .map(|route| LocalizedRoute::Localized { lang, route })
        } else {
            // This is a default route
            Route::from_path(path, params).map(LocalizedRoute::Default)
        }
    }

    fn to_path(&self) -> String {
        match self {
            LocalizedRoute::Default(route) => route.to_path(),
            LocalizedRoute::Localized { lang, route } => {
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
        let mut routes = Route::routes();

        // Add language prefixed routes
        for &lang in SUPPORTED_LANGUAGES {
            for route in Route::routes() {
                let localized_route = if route == "/" {
                    format!("/{}", lang)
                } else {
                    format!("/{}{}", lang, route)
                };

                // Leak to get static lifetime
                let localized_route = Box::leak(localized_route.into_boxed_str());
                routes.push(localized_route);
            }
        }

        routes
    }

    fn not_found_route() -> Option<Self> {
        Some(LocalizedRoute::Default(Route::NotFound))
    }

    fn recognize(pathname: &str) -> Option<Self> {
        let parts: Vec<&str> = pathname.trim_matches('/').split('/').collect();
        if parts.is_empty() {
            return Some(LocalizedRoute::Default(Route::Home));
        }

        let first_part = parts[0];
        if SUPPORTED_LANGUAGES.contains(&first_part) {
            // This is a localized route
            let lang = first_part.to_string();
            let remaining = if parts.len() > 1 {
                format!("/{}", parts[1..].join("/"))
            } else {
                "/".to_string()
            };

            Route::recognize(&remaining).map(|route| LocalizedRoute::Localized { lang, route })
        } else {
            // This is a default route
            Route::recognize(pathname).map(LocalizedRoute::Default)
        }
    }
}
#[derive(Clone)]
pub struct LocalizedRouteIter {
    items: Vec<LocalizedRoute>,
    position: usize,
}

impl Iterator for LocalizedRouteIter {
    type Item = LocalizedRoute;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position < self.items.len() {
            let item = self.items[self.position].clone();
            self.position += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.items.len().saturating_sub(self.position);
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for LocalizedRouteIter {
    fn len(&self) -> usize {
        self.items.len().saturating_sub(self.position)
    }
}

impl DoubleEndedIterator for LocalizedRouteIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.position < self.items.len() {
            let last = self.items.len() - 1;
            let item = self.items[last].clone();
            self.items.truncate(last);
            Some(item)
        } else {
            None
        }
    }
}

impl FusedIterator for LocalizedRouteIter {}

// Implement IntoEnumIterator for LocalizedRoute
impl IntoEnumIterator for LocalizedRoute {
    type Iterator = LocalizedRouteIter;

    fn iter() -> Self::Iterator {
        let mut items = Vec::new();

        // Generate all variants: first Default then Localized for each route
        for route in Route::iter() {
            // Add default variant
            items.push(LocalizedRoute::Default(route.clone()));

            // Add localized variants for each language
            for &lang in SUPPORTED_LANGUAGES {
                items.push(LocalizedRoute::Localized {
                    lang: lang.to_string(),
                    route: route.clone(),
                });
            }
        }

        LocalizedRouteIter { items, position: 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localized_route_iterator() {
        let routes: Vec<LocalizedRoute> = LocalizedRoute::iter().collect();

        // Expected count: Route variants * (1 Default + languages count)
        let expected_count = Route::iter().count() * (1 + SUPPORTED_LANGUAGES.len());
        assert_eq!(routes.len(), expected_count);

        // Check first item is Default(Home)
        assert!(matches!(routes[0], LocalizedRoute::Default(Route::Home)));

        // Check we have all default routes
        for route in Route::iter() {
            let has_default = routes
                .iter()
                .any(|r| matches!(r, LocalizedRoute::Default(r_inner) if r_inner == &route));
            assert!(has_default, "Missing default route: {:?}", route);
        }

        // Check we have all languages for all routes
        for route in Route::iter() {
            for &lang in SUPPORTED_LANGUAGES {
                let has_lang = routes.iter().any(|r| {
                    matches!(r, LocalizedRoute::Localized { lang: l, route: r_inner }
                             if l == lang && r_inner == &route)
                });
                assert!(has_lang, "Missing {:?} route for language {}", route, lang);
            }
        }
    }

    #[test]
    fn test_double_ended() {
        let mut iter = LocalizedRoute::iter();
        let first = iter.next();
        let last = iter.next_back();

        assert!(first.is_some());
        assert!(last.is_some());
        assert_ne!(first, last);
    }
}
