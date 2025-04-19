use crate::i18n::t;
use yew::prelude::*;
use yew_router::prelude::{use_hash, use_language};

const FEATURES: &[(&str, &str)] = &[
    (
        "pre_render",
        "Pre-renders Yew applications to static HTML for improved SEO",
    ),
    (
        "templates",
        "Customizable HTML templates with variable substitution",
    ),
    ("templating", "Advanced attribute-based templating system"),
    ("plugins", "Extensible generator plugin architecture"),
    (
        "seo",
        "Built-in SEO generators (meta tags, Open Graph, Twitter Cards)",
    ),
];

#[function_component(AboutPage)]
pub fn about_page() -> Html {
    let language = use_language();
    let lang = language.lang.as_str();

    // Get the current hash (feature id)
    let hash = use_hash();

    let on_feature_click = Callback::from(move |feature_id: String| {
        if let Some(window) = web_sys::window() {
            let location = window.location();
            let _ = location.set_hash(&feature_id);
        }
    });

    html! {
        <div>
            <h1>{ t("about_page", lang) }</h1>
            <p>{ t("about_description", lang) }</p>
            <h2>{"Core Features"}</h2>
            <ul class="crate-features">
                { for FEATURES.iter().map(|(id, desc)| {
                    let is_active = hash.as_deref() == Some(*id);
                    let class = if is_active { "feature-item active-feature" } else { "feature-item" };
                    let id = id.to_string();
                    html! {
                        <li
                            class={class}
                            onclick={{
                                let on_feature_click = on_feature_click.clone();
                                let id = id.clone();
                                Callback::from(move |_| on_feature_click.emit(id.clone()))
                            }}
                            style={if is_active { "background: #e0f7fa; font-weight: bold;" } else { "" }}
                        >
                            {desc}
                        </li>
                    }
                })}
            </ul>
        </div>
    }
}
