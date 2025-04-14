use crate::i18n::t;
use yew::prelude::*;
use yew_router::prelude::use_language;

#[function_component(AboutPage)]
pub fn about_page() -> Html {
    let language = use_language();
    let lang = language.lang.as_str();

    html! {
        <div>
            <h1>{ t("about_page", lang) }</h1>
            <p>{ t("about_description", lang) }</p>
        </div>
    }
}
