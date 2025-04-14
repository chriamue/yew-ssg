use crate::language_context::LanguageProvider;
use crate::localized_routable::LocalizedRoutable;
use yew::prelude::*;

/// Properties for the RouteLanguageExtractor component
#[derive(Properties, PartialEq)]
pub struct RouteLanguageExtractorProps {
    #[prop_or_default]
    pub children: Children,
}

/// Component that extracts language from the current route and provides it via context
#[function_component(RouteLanguageExtractor)]
pub fn route_language_extractor<R: LocalizedRoutable + 'static>(
    props: &RouteLanguageExtractorProps,
) -> Html {
    // For SSG, get the path from environment
    #[cfg(feature = "ssg")]
    let current_path = { crate::get_static_path().unwrap_or_else(|| "/".to_string()) };

    // For client-side rendering, use the location
    #[cfg(not(feature = "ssg"))]
    let current_path = {
        let location = yew_router::hooks::use_location().unwrap();
        location.path().to_string()
    };

    // Try to match the route to extract language
    let lang = R::recognize(&current_path)
        .and_then(|route| route.get_lang())
        .unwrap_or_else(|| R::default_language().to_string());

    html! {
        <LanguageProvider {lang}>
            {props.children.clone()}
        </LanguageProvider>
    }
}

/// Hook to extract the current route's language
#[hook]
pub fn use_route_language<R: LocalizedRoutable + 'static>() -> String {
    // For SSG, get the path from environment
    #[cfg(feature = "ssg")]
    let current_path = { crate::get_static_path().unwrap_or_else(|| "/".to_string()) };

    // For client-side rendering, use the location
    #[cfg(not(feature = "ssg"))]
    let current_path = {
        let location = yew_router::hooks::use_location().unwrap();
        location.path().to_string()
    };

    // Try to match the route to extract language
    R::recognize(&current_path)
        .and_then(|route| route.get_lang())
        .unwrap_or_else(|| R::default_language().to_string())
}
