use crate::i18n::t;
use crate::route::{LocalizedRoute, SUPPORTED_LANGUAGES};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
struct LanguageButtonProps {
    lang: &'static str,
    current_lang: String,
    on_click: Callback<&'static str>,
}

#[function_component(LanguageButton)]
fn language_button(props: &LanguageButtonProps) -> Html {
    let lang = props.lang;
    let is_current = props.current_lang == lang;

    // Get the flag emoji and display name for the language
    let (flag, display_name) = match lang {
        "en" => ("🇬🇧", "English"),
        "de" => ("🇩🇪", "Deutsch"),
        _ => ("🌐", lang),
    };

    let on_click = {
        let lang = lang;
        let callback = props.on_click.clone();

        Callback::from(move |_: MouseEvent| {
            callback.emit(lang);
        })
    };

    // Add classes based on current selection
    let class = if is_current {
        "language-button language-button--active"
    } else {
        "language-button"
    };

    html! {
        <button
            type="button"
            class={class}
            onclick={on_click}
            title={display_name}
            aria-pressed={is_current.to_string()}
        >
            <span class="language-button__flag">{flag}</span>
            <span class="language-button__name">{display_name}</span>
        </button>
    }
}

#[function_component(LanguageSelector)]
pub fn language_selector() -> Html {
    let language = use_language();
    let current_lang = language.lang.clone();
    let navigator = use_navigator().unwrap();
    let current_route = use_route::<LocalizedRoute>().unwrap_or_default();

    // Callback for language change
    let on_language_change = {
        let navigator = navigator.clone();
        let current_route = current_route.clone();

        Callback::from(move |new_lang: &'static str| {
            // Get the base route
            let base_route = current_route.get_route().clone();

            // Create a new localized route with the selected language
            let new_route = LocalizedRoute::from_route(base_route, Some(new_lang));

            // Navigate to the new route
            navigator.push(&new_route);
        })
    };

    html! {
        <div class="language-selector">
            <span class="language-selector__label">{t("language", &current_lang)}{": "}</span>
            <div class="language-selector__buttons">
                {
                    SUPPORTED_LANGUAGES.iter().map(|&lang| {
                        html! {
                            <LanguageButton
                                lang={lang}
                                current_lang={current_lang.clone()}
                                on_click={on_language_change.clone()}
                            />
                        }
                    }).collect::<Html>()
                }
            </div>
        </div>
    }
}
