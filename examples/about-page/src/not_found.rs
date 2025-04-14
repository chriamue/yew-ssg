use crate::i18n::t;
use crate::route::{LocalizedRoute, Route};
use yew::prelude::*;
use yew_router::prelude::use_language;
use yew_router::prelude::*;

#[function_component(NotFoundPage)]
pub fn not_found_page() -> Html {
    let language = use_language();
    let lang = language.lang.as_str();

    // Get the navigator
    let navigator = use_navigator().unwrap_or_else(|| {
        // This should not happen in normal circumstances
        panic!("Navigator not available");
    });

    // Create a callback to navigate back to home
    let go_home = {
        let navigator = navigator.clone();
        let lang = lang.to_string();

        Callback::from(move |_: MouseEvent| {
            let home_route = LocalizedRoute::from_route(Route::Home, Some(&lang));
            navigator.push(&home_route);
        })
    };

    html! {
        <div class="not-found-container">
            <h1>{ t("page_not_found", lang) }</h1>
            <div class="error-details">
                <p>{ t("not_found_message", lang) }</p>
            </div>
            <div class="not-found-actions">
                <button onclick={go_home} class="back-home-button">
                    { t("back_to_home", lang) }
                </button>
            </div>
        </div>
    }
}
