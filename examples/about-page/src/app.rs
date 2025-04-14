use crate::i18n::t;
use crate::language_selector::LanguageSelector;
use crate::route::{LocalizedRoute, Route};
use crate::switch_route::switch_route;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <LocalizedApp<LocalizedRoute>>
            <AppContent />
        </LocalizedApp<LocalizedRoute>>
    }
}

#[function_component(AppContent)]
fn app_content() -> Html {
    // Get current language from context
    let language = use_language();
    let lang = language.lang.as_str();

    html! {
        <>
            <nav>
                <ul>
                    <li>
                        <Link<LocalizedRoute> to={LocalizedRoute::from_route(Route::Home, Some(lang))}>
                            { t("home", lang) }
                        </Link<LocalizedRoute>>
                    </li>
                    <li>
                        <Link<LocalizedRoute> to={LocalizedRoute::from_route(Route::About, Some(lang))}>
                            { t("about", lang) }
                        </Link<LocalizedRoute>>
                    </li>
                    <li>
                        <Link<LocalizedRoute> to={LocalizedRoute::from_route(Route::Readme, Some(lang))}>
                            { t("readme", lang) }
                        </Link<LocalizedRoute>>
                    </li>
                    <li>
                        <Link<LocalizedRoute> to={LocalizedRoute::from_route(Route::Crate { id: "yew-ssg".to_string() }, Some(lang))}>
                            { t("yew_ssg_crate", lang) }
                        </Link<LocalizedRoute>>
                    </li>
                    <li>
                        <Link<LocalizedRoute> to={LocalizedRoute::from_route(Route::Crate { id: "yew-ssg-router".to_string() }, Some(lang))}>
                            { t("yew_ssg_router_crate", lang) }
                        </Link<LocalizedRoute>>
                    </li>
                    <li>
                        <Link<LocalizedRoute> to={LocalizedRoute::from_route(Route::NotFound, Some(lang))}>
                            { t("not_found", lang) }
                        </Link<LocalizedRoute>>
                    </li>
                </ul>
                <LanguageSelector />
            </nav>
            <main>
                <LocalizedSwitch<LocalizedRoute> render={switch_route} />
            </main>
        </>
    }
}
