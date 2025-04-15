use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use yew_router::prelude::*;

pub const SUPPORTED_LANGUAGES: &[&str] = &["en", "de"];
pub const DEFAULT_LANGUAGE: &str = "en";

#[derive(Clone, Routable, PartialEq, Debug, EnumIter, LocalizedRoutable)]
#[localized(languages = ["en", "de"], default = "en", wrapper = "LocalizedRoute")]
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

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;

    #[test]
    fn test_localized_routes() {
        let localized_routes = LocalizedRoute::routes();

        // The count should be: (number of routes) * (1 default + number of languages)
        let expected_count = Route::routes().len() * (1 + SUPPORTED_LANGUAGES.len());
        assert_eq!(localized_routes.len(), expected_count);

        // Check that we have routes for each language
        for &lang in SUPPORTED_LANGUAGES {
            // Check for a home route for each language
            let localized_home = format!("/{}{}", lang, "/");
            let has_localized_home = localized_routes
                .iter()
                .any(|&route| route == localized_home);
            assert!(
                has_localized_home,
                "Missing localized home route for {}",
                lang
            );
        }

        // Test route recognition
        assert_eq!(
            LocalizedRoute::recognize("/").map(|r| r.get_route().clone()),
            Some(Route::Home)
        );

        assert_eq!(
            LocalizedRoute::recognize("/de").map(|r| {
                if let LocalizedRoute::Localized { lang, route } = r {
                    assert_eq!(lang, "de");
                    route
                } else {
                    panic!("Expected localized route, got {:?}", r);
                }
            }),
            Some(Route::Home)
        );

        assert_eq!(
            LocalizedRoute::recognize("/en").map(|r| {
                if let LocalizedRoute::Localized { lang, route } = r {
                    assert_eq!(lang, "en");
                    route
                } else {
                    panic!("Expected localized route, got {:?}", r);
                }
            }),
            Some(Route::Home)
        );
    }

    #[test]
    fn test_localized_route() {
        // Test construction and getters
        let route = LocalizedRoute::from_route(Route::Home, None);
        assert!(matches!(route, LocalizedRoute::Default(_)));
        assert_eq!(route.get_lang(), None);
        assert_eq!(route.get_route(), &Route::Home);

        let route = LocalizedRoute::from_route(Route::Home, Some("de"));
        assert!(matches!(route, LocalizedRoute::Localized { ref lang, .. } if lang == "de"));
        assert_eq!(route.get_lang(), Some("de".to_string()));
        assert_eq!(route.get_route(), &Route::Home);

        let route = LocalizedRoute::from_route(Route::Home, Some("en"));
        assert!(matches!(route, LocalizedRoute::Localized { ref lang, .. } if lang == "en"));
        assert_eq!(route.get_lang(), Some("en".to_string()));
        assert_eq!(route.get_route(), &Route::Home);
    }

    #[test]
    fn test_localized_route_paths() {
        // Test path generation
        assert_eq!(LocalizedRoute::from_route(Route::Home, None).to_path(), "/");

        assert_eq!(
            LocalizedRoute::from_route(Route::Home, Some("en")).to_path(),
            "/en/"
        );

        assert_eq!(
            LocalizedRoute::from_route(Route::Home, Some("de")).to_path(),
            "/de/"
        );

        // Test with different route types
        assert_eq!(
            LocalizedRoute::from_route(
                Route::Crate {
                    id: "123".to_string()
                },
                Some("en")
            )
            .to_path(),
            "/en/crate/123"
        );

        // Note: We don't have a SearchWithQuery route in this example
        // so I'm using Crate instead for this test
        assert_eq!(
            LocalizedRoute::from_route(
                Route::Crate {
                    id: "test".to_string()
                },
                Some("de")
            )
            .to_path(),
            "/de/crate/test"
        );
    }

    #[test]
    fn test_route_recognition() {
        // Test recognition of paths
        let recognized = LocalizedRoute::recognize("/");
        assert!(matches!(
            recognized,
            Some(LocalizedRoute::Default(Route::Home))
        ));

        let recognized = LocalizedRoute::recognize("/de");
        assert!(
            matches!(recognized, Some(LocalizedRoute::Localized { lang, route: Route::Home }) if lang == "de")
        );

        let recognized = LocalizedRoute::recognize("/en/crate/123");
        assert!(matches!(
            recognized,
            Some(LocalizedRoute::Localized {
                lang,
                route: Route::Crate { id }
            }) if lang == "en" && id == "123"
        ));

        // Test invalid language
        let recognized = LocalizedRoute::recognize("/xx/about");
        assert_eq!(recognized, Some(LocalizedRoute::Default(Route::NotFound)));
    }

    #[test]
    fn test_invalid_language_handling() {
        // Test that invalid languages are validated properly
        assert_eq!(LocalizedRoute::validate_lang("xx"), "en");
        assert_eq!(LocalizedRoute::validate_lang("de"), "de");

        // Test from_route with invalid language
        let route = LocalizedRoute::from_route(Route::Home, Some("invalid"));
        assert!(matches!(route, LocalizedRoute::Default(Route::Home)));
    }

    #[test]
    fn test_localized_router_iter() {
        // Test the iterator implementation
        let routes: Vec<LocalizedRoute> = LocalizedRoute::iter().collect();

        // Calculate expected count
        let expected_count = Route::iter().count() * (1 + SUPPORTED_LANGUAGES.len());
        assert_eq!(routes.len(), expected_count);

        // Test that we have both default and localized routes
        let default_routes_count = routes
            .iter()
            .filter(|r| matches!(r, LocalizedRoute::Default(_)))
            .count();
        assert_eq!(default_routes_count, Route::iter().count());

        let localized_routes_count = routes
            .iter()
            .filter(|r| matches!(r, LocalizedRoute::Localized { .. }))
            .count();
        assert_eq!(
            localized_routes_count,
            Route::iter().count() * SUPPORTED_LANGUAGES.len()
        );
    }

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
                .any(|r| matches!(r, LocalizedRoute::Default(r_inner) if *r_inner == route));
            assert!(has_default, "Missing default route: {:?}", route);
        }

        // Check we have all languages for all routes
        for route in Route::iter() {
            for &lang in SUPPORTED_LANGUAGES {
                let has_lang = routes.iter().any(|r| {
                    matches!(r, LocalizedRoute::Localized { lang: l, route: r_inner }
                             if l == lang && *r_inner == route)
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
