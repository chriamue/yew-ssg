use crate::LocalizedRoutable;
use yew::prelude::*;

/// Properties for the LocalizedLink component
#[derive(Properties, PartialEq)]
pub struct LocalizedLinkProps<R>
where
    R: LocalizedRoutable + 'static,
{
    pub to: R,
    #[prop_or_default]
    pub query: Option<String>,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub anchor_ref: NodeRef,
    #[prop_or_default]
    pub classes: Classes,
    #[prop_or_default]
    pub children: Html,
    #[prop_or_default]
    pub target: Option<AttrValue>,
    #[prop_or_default]
    pub title: Option<AttrValue>,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
}

/// A link component for localized routes
///
/// Generates appropriate href values with language prefixes and
/// provides client-side navigation in the browser.
#[function_component(LocalizedLink)]
pub fn localized_link<R>(props: &LocalizedLinkProps<R>) -> Html
where
    R: LocalizedRoutable + 'static,
{
    let path = props.to.to_path();

    // Add query string if provided
    let href = match &props.query {
        Some(query) if !query.is_empty() => {
            if path.contains('?') {
                format!("{}&{}", path, query)
            } else {
                format!("{}?{}", path, query)
            }
        }
        _ => path,
    };

    // For SSG, combine with base URL
    #[cfg(feature = "ssg")]
    let href = crate::utils::combine_with_base_url(&href);

    let onclick = {
        let on_click = props.onclick.clone();
        let to = props.to.clone();
        let disabled = props.disabled;

        // Only perform client-side navigation in browser context
        #[cfg(not(feature = "ssg"))]
        let navigator = yew_router::hooks::use_navigator().map(|nav| nav.clone());

        Callback::from(move |e: MouseEvent| {
            if !disabled {
                // Call the provided onclick handler
                on_click.emit(e.clone());

                // In the browser, use client-side navigation
                #[cfg(not(feature = "ssg"))]
                if let Some(navigator) = navigator.clone() {
                    // Only handle client-side navigation if not modified click
                    if !e.ctrl_key()
                        && !e.shift_key()
                        && !e.meta_key()
                        && !e.alt_key()
                        && e.button() == 0
                    {
                        e.prevent_default();
                        let to_clone = to.clone();
                        navigator.push(&to_clone);
                    }
                }
            }
        })
    };

    html! {
        <a
            href={href}
            class={props.classes.clone()}
            ref={props.anchor_ref.clone()}
            target={props.target.clone()}
            title={props.title.clone()}
            {onclick}
            disabled={props.disabled}
        >
            { props.children.clone() }
        </a>
    }
}
