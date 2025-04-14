use crate::language_context::LanguageProvider;
use crate::localized_routable::LocalizedRoutable;
use std::fmt::Debug;
use yew::prelude::*;
use yew_router::prelude::*;

/// Properties for the LocalizedApp component
#[derive(Properties, PartialEq)]
pub struct LocalizedAppProps {
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub basename: Option<String>,
}

/// A component that sets up a router with localization support
///
/// This component combines a BrowserRouter with language context,
/// automatically extracting the language from the current route.
#[function_component(LocalizedApp)]
pub fn localized_app<R>(props: &LocalizedAppProps) -> Html
where
    R: LocalizedRoutable + Debug + 'static,
{
    #[cfg(feature = "ssg")]
    {
        // For SSG, we use a slightly different approach
        use crate::static_router::StaticRouter;

        // Get the path from environment
        let current_path = crate::get_static_path().unwrap_or_else(|| "/".to_string());

        // Extract language from path
        let lang = R::recognize(&current_path)
            .and_then(|route| route.get_lang())
            .unwrap_or_else(|| R::default_language().to_string());

        html! {
            <LanguageProvider {lang}>
                <StaticRouter basename={props.basename.clone()}>
                    {props.children.clone()}
                </StaticRouter>
            </LanguageProvider>
        }
    }

    #[cfg(not(feature = "ssg"))]
    {
        html! {
            <BrowserRouter basename={props.basename.clone()}>
                <LocalizedRouteProvider<R>>
                    {props.children.clone()}
                </LocalizedRouteProvider<R>>
            </BrowserRouter>
        }
    }
}

/// Properties for the route language provider
#[derive(Properties, PartialEq)]
struct LocalizedRouteProviderProps {
    #[prop_or_default]
    children: Children,
}

/// Component that extracts language from the current route and provides language context
#[function_component(LocalizedRouteProvider)]
fn localized_route_provider<R>(props: &LocalizedRouteProviderProps) -> Html
where
    R: LocalizedRoutable + Debug + 'static,
{
    // Get the current path
    let location = use_location().unwrap();
    let current_path = location.path();

    // Extract language from path
    let lang = R::recognize(current_path)
        .and_then(|route| route.get_lang())
        .unwrap_or_else(|| R::default_language().to_string());

    html! {
        <LanguageProvider {lang}>
            {props.children.clone()}
        </LanguageProvider>
    }
}
