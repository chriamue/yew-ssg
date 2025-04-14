use crate::i18n::t;
use yew::prelude::*;
use yew_router::prelude::use_language;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    let language = use_language();
    let lang = language.lang.as_str();

    html! {
        <div>
            <h1>{ t("welcome_to_home", lang) }</h1>
            <p>{ t("simple_example", lang) }</p>
        </div>
    }
}
