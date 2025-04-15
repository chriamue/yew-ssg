#![cfg(test)]

use crate::impl_localized_route;
use crate::LocalizedRoutable;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;
use yew_router::Routable;

// Define a test Route enum - make it pub(crate) to resolve visibility issues
#[derive(Clone, Routable, PartialEq, Debug, EnumIter)]
pub enum TestRoute {
    #[at("/")]
    Home,
    #[at("/about")]
    About,
    #[at("/contact")]
    Contact,
    #[not_found]
    #[at("/404")]
    NotFound,
}

// Define languages for testing
pub(crate) const TEST_LANGUAGES: &[&str] = &["en", "fr", "de"];
pub(crate) const TEST_DEFAULT_LANG: &str = "en";

// Implement the localized route using our macro
impl_localized_route!(
    TestRoute,
    TestLocalizedRoute,
    TEST_LANGUAGES,
    TEST_DEFAULT_LANG
);

#[test]
fn test_localized_route_macro() {
    // Test default construction
    let default_route = TestLocalizedRoute::default();
    assert!(matches!(
        default_route,
        TestLocalizedRoute::Default(TestRoute::NotFound)
    ));

    // Test from_route with language
    let localized_route = TestLocalizedRoute::from_route(TestRoute::About, Some("fr"));
    assert!(matches!(
        localized_route,
        TestLocalizedRoute::Localized {
            lang,
            route: TestRoute::About
        } if lang == "fr"
    ));

    // Test from_route with invalid language (should default)
    let invalid_lang_route = TestLocalizedRoute::from_route(TestRoute::Contact, Some("es"));
    assert!(matches!(
        invalid_lang_route,
        TestLocalizedRoute::Default(TestRoute::Contact)
    ));

    // Test path generation
    assert_eq!(TestLocalizedRoute::Default(TestRoute::Home).to_path(), "/");
    assert_eq!(
        TestLocalizedRoute::Localized {
            lang: "fr".to_string(),
            route: TestRoute::Home
        }
        .to_path(),
        "/fr"
    );
    assert_eq!(
        TestLocalizedRoute::Localized {
            lang: "de".to_string(),
            route: TestRoute::About
        }
        .to_path(),
        "/de/about"
    );

    // Test path recognition
    let recognized = TestLocalizedRoute::recognize("/fr/about");
    assert!(matches!(
        recognized,
        Some(TestLocalizedRoute::Localized {
            lang,
            route: TestRoute::About
        }) if lang == "fr"
    ));

    let recognized_root = TestLocalizedRoute::recognize("/de");
    assert!(matches!(
        recognized_root,
        Some(TestLocalizedRoute::Localized {
            lang,
            route: TestRoute::Home
        }) if lang == "de"
    ));

    let recognized_default = TestLocalizedRoute::recognize("/contact");
    assert!(matches!(
        recognized_default,
        Some(TestLocalizedRoute::Default(TestRoute::Contact))
    ));
}

#[test]
fn test_localized_route_iter() {
    // Test iteration over all combinations
    let routes: Vec<TestLocalizedRoute> = TestLocalizedRoute::iter().collect();

    // Expected count: Route variants * (1 Default + supported languages)
    let expected_count = TestRoute::iter().count() * (1 + TEST_LANGUAGES.len());
    assert_eq!(routes.len(), expected_count);

    // Verify we have a Default and Localized version for each route and language
    let has_default_home = routes
        .iter()
        .any(|r| matches!(r, TestLocalizedRoute::Default(TestRoute::Home)));
    assert!(has_default_home);

    let has_fr_about = routes.iter().any(|r| {
        matches!(
            r, TestLocalizedRoute::Localized { lang, route: TestRoute::About } if lang == "fr"
        )
    });
    assert!(has_fr_about);
}
