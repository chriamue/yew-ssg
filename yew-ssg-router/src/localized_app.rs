use crate::language_context::{LanguageContext, LanguageProvider};
use crate::localized_routable::LocalizedRoutable;
use std::fmt::Debug;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct LocalizedAppProps {
    #[prop_or_default]
    pub children: Children,
    #[prop_or_default]
    pub basename: Option<String>,
}

#[function_component(LocalizedApp)]
pub fn localized_app<R>(props: &LocalizedAppProps) -> Html
where
    R: LocalizedRoutable + Debug + 'static,
{
    #[cfg(feature = "ssg")]
    {
        use crate::static_router::StaticRouter;

        // Get the path from environment or thread-local
        let current_path = crate::get_static_path().unwrap_or_else(|| "/".to_string());

        // Extract language from path or use thread-local/env fallback
        let lang = R::recognize(&current_path)
            .and_then(|route| route.get_lang())
            .unwrap_or_else(|| LanguageContext::get_current_lang());

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

#[derive(Properties, PartialEq)]
struct LocalizedRouteProviderProps {
    #[prop_or_default]
    children: Children,
}

#[function_component(LocalizedRouteProvider)]
fn localized_route_provider<R>(props: &LocalizedRouteProviderProps) -> Html
where
    R: LocalizedRoutable + Debug + 'static,
{
    // Get the current path
    let location = use_location().unwrap();
    let current_path = location.path();

    // Extract language from path or use current language
    let lang = R::recognize(current_path)
        .and_then(|route| route.get_lang())
        .unwrap_or_else(|| LanguageContext::get_current_lang());

    html! {
        <LanguageProvider {lang}>
            {props.children.clone()}
        </LanguageProvider>
    }
}
