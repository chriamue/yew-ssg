// yew-ssg-router/tests/localized_routable_tests.rs
use crate::LocalizedRoutable;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use yew_router::Routable;

// Mock types for testing
#[derive(Clone, PartialEq)]
enum MockRoute {
    Home,
    About,
    Profile { id: String },
    NotFound,
}

impl Debug for MockRoute {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Home => write!(f, "Home"),
            Self::About => write!(f, "About"),
            Self::Profile { id } => write!(f, "Profile {{ id: {} }}", id),
            Self::NotFound => write!(f, "NotFound"),
        }
    }
}

impl Routable for MockRoute {
    fn from_path(path: &str, params: &HashMap<&str, &str>) -> Option<Self> {
        match path {
            "/" => Some(Self::Home),
            "/about" => Some(Self::About),
            "/profile" => {
                if let Some(id) = params.get("id") {
                    Some(Self::Profile { id: id.to_string() })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn to_path(&self) -> String {
        match self {
            Self::Home => "/".to_string(),
            Self::About => "/about".to_string(),
            Self::Profile { id } => format!("/profile/{}", id),
            Self::NotFound => "/404".to_string(),
        }
    }

    fn routes() -> Vec<&'static str> {
        vec!["/", "/about", "/profile/:id"]
    }

    fn not_found_route() -> Option<Self> {
        Some(Self::NotFound)
    }

    fn recognize(pathname: &str) -> Option<Self> {
        match pathname {
            "/" => Some(Self::Home),
            "/about" => Some(Self::About),
            p if p.starts_with("/profile/") => {
                let id = p.trim_start_matches("/profile/");
                if !id.is_empty() {
                    Some(Self::Profile { id: id.to_string() })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
enum MockLocalizedRoute {
    Default(MockRoute),
    Localized { lang: String, route: MockRoute },
}

const SUPPORTED_LANGUAGES: &[&str] = &["en", "de", "es", "fr"];

impl LocalizedRoutable for MockLocalizedRoute {
    type BaseRoute = MockRoute;

    fn get_lang(&self) -> Option<String> {
        match self {
            Self::Default(_) => None,
            Self::Localized { lang, .. } => Some(lang.clone()),
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
            Some(lang) if Self::supported_languages().contains(&lang) => Self::Localized {
                lang: lang.to_string(),
                route,
            },
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
        SUPPORTED_LANGUAGES
    }
}

impl Routable for MockLocalizedRoute {
    fn from_path(path: &str, params: &HashMap<&str, &str>) -> Option<Self> {
        // First check if the path starts with a language code
        let parts: Vec<&str> = path.trim_matches('/').split('/').collect();
        if parts.is_empty() {
            // Handle empty path - defaults to root
            return Some(Self::Default(MockRoute::Home));
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

            MockRoute::from_path(&remaining, params).map(|route| Self::Localized { lang, route })
        } else {
            // This is a default route
            MockRoute::from_path(path, params).map(Self::Default)
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
        let mut routes = MockRoute::routes();

        // Add language prefixed routes
        for &lang in SUPPORTED_LANGUAGES {
            for route in MockRoute::routes() {
                let localized_route = if route == "/" {
                    format!("/{}", lang)
                } else {
                    format!("/{}{}", lang, route)
                };

                // Need to convert to static
                let localized_route = Box::leak(localized_route.into_boxed_str());
                routes.push(localized_route);
            }
        }

        routes
    }

    fn not_found_route() -> Option<Self> {
        Some(Self::Default(MockRoute::NotFound))
    }

    fn recognize(pathname: &str) -> Option<Self> {
        let parts: Vec<&str> = pathname.trim_matches('/').split('/').collect();
        if parts.is_empty() {
            return Some(Self::Default(MockRoute::Home));
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

            MockRoute::recognize(&remaining).map(|route| Self::Localized { lang, route })
        } else {
            // This is a default route
            MockRoute::recognize(pathname).map(Self::Default)
        }
    }
}

#[test]
fn test_localized_route_from_path() {
    // Default routes
    assert_eq!(
        MockLocalizedRoute::from_path("/", &HashMap::new()),
        Some(MockLocalizedRoute::Default(MockRoute::Home))
    );

    assert_eq!(
        MockLocalizedRoute::from_path("/about", &HashMap::new()),
        Some(MockLocalizedRoute::Default(MockRoute::About))
    );

    // Localized routes
    assert_eq!(
        MockLocalizedRoute::from_path("/en", &HashMap::new()),
        Some(MockLocalizedRoute::Localized {
            lang: "en".to_string(),
            route: MockRoute::Home
        })
    );

    assert_eq!(
        MockLocalizedRoute::from_path("/de/about", &HashMap::new()),
        Some(MockLocalizedRoute::Localized {
            lang: "de".to_string(),
            route: MockRoute::About
        })
    );

    // Unsupported language - should not match
    assert_eq!(
        MockLocalizedRoute::from_path("/xx/about", &HashMap::new()),
        None
    );
}

#[test]
fn test_localized_route_to_path() {
    // Default routes
    assert_eq!(MockLocalizedRoute::Default(MockRoute::Home).to_path(), "/");

    assert_eq!(
        MockLocalizedRoute::Default(MockRoute::About).to_path(),
        "/about"
    );

    // Localized routes
    assert_eq!(
        MockLocalizedRoute::Localized {
            lang: "en".to_string(),
            route: MockRoute::Home
        }
        .to_path(),
        "/en"
    );

    assert_eq!(
        MockLocalizedRoute::Localized {
            lang: "de".to_string(),
            route: MockRoute::About
        }
        .to_path(),
        "/de/about"
    );
}

#[test]
fn test_extract_lang_from_path() {
    // Default routes
    let (lang, path) = MockLocalizedRoute::extract_lang_from_path("/");
    assert_eq!(lang, None);
    assert_eq!(path, "/");

    let (lang, path) = MockLocalizedRoute::extract_lang_from_path("/about");
    assert_eq!(lang, None);
    assert_eq!(path, "/about");

    // Localized routes
    let (lang, path) = MockLocalizedRoute::extract_lang_from_path("/en");
    assert_eq!(lang, Some("en".to_string()));
    assert_eq!(path, "/");

    let (lang, path) = MockLocalizedRoute::extract_lang_from_path("/de/about");
    assert_eq!(lang, Some("de".to_string()));
    assert_eq!(path, "/about");

    // Unsupported language
    let (lang, path) = MockLocalizedRoute::extract_lang_from_path("/xx/about");
    assert_eq!(lang, None);
    assert_eq!(path, "/xx/about");
}

#[test]
fn test_localize_path() {
    assert_eq!(MockLocalizedRoute::localize_path("/", "en"), "/en");
    assert_eq!(
        MockLocalizedRoute::localize_path("/about", "de"),
        "/de/about"
    );
    assert_eq!(
        MockLocalizedRoute::localize_path("/profile/123", "fr"),
        "/fr/profile/123"
    );
}

#[test]
fn test_validate_lang() {
    // Valid languages
    assert_eq!(MockLocalizedRoute::validate_lang("en"), "en");
    assert_eq!(MockLocalizedRoute::validate_lang("de"), "de");
    assert_eq!(MockLocalizedRoute::validate_lang("es"), "es");
    assert_eq!(MockLocalizedRoute::validate_lang("fr"), "fr");

    // Invalid languages - should return default
    assert_eq!(MockLocalizedRoute::validate_lang("xx"), "en");
    assert_eq!(MockLocalizedRoute::validate_lang(""), "en");
}

#[test]
fn test_get_lang_and_route() {
    // Default route
    let route = MockLocalizedRoute::Default(MockRoute::Home);
    assert_eq!(route.get_lang(), None);
    assert_eq!(route.get_route(), &MockRoute::Home);

    // Localized route
    let route = MockLocalizedRoute::Localized {
        lang: "de".to_string(),
        route: MockRoute::About,
    };
    assert_eq!(route.get_lang(), Some("de".to_string()));
    assert_eq!(route.get_route(), &MockRoute::About);
}

#[test]
fn test_from_route() {
    // With valid language
    assert_eq!(
        MockLocalizedRoute::from_route(MockRoute::Home, Some("de")),
        MockLocalizedRoute::Localized {
            lang: "de".to_string(),
            route: MockRoute::Home,
        }
    );

    // With invalid language
    assert_eq!(
        MockLocalizedRoute::from_route(MockRoute::Home, Some("xx")),
        MockLocalizedRoute::Default(MockRoute::Home)
    );

    // With no language
    assert_eq!(
        MockLocalizedRoute::from_route(MockRoute::Home, None),
        MockLocalizedRoute::Default(MockRoute::Home)
    );
}
