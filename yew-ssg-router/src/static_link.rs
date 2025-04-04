use std::fmt::Debug;
use yew::prelude::*;
use yew_router::Routable;

/// Properties for a static link
#[derive(Properties, Clone, PartialEq)]
pub struct StaticLinkProps<R>
where
    R: Routable + 'static,
{
    pub to: R,
    #[prop_or_default]
    pub query: Option<()>,
    #[prop_or_default]
    pub state: Option<()>,
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

/// A link component that renders as a regular <a> tag during SSG
#[function_component(StaticLink)]
pub fn static_link<R>(props: &StaticLinkProps<R>) -> Html
where
    R: Routable + PartialEq + Clone + Debug + 'static,
{
    // Get the path from the route
    let path = props.to.to_path();

    // If disabled, return a span
    if props.disabled {
        return html! {
            <span class={props.classes.clone()}>
                { props.children.clone() }
            </span>
        };
    }

    // In SSG mode, render a regular <a> tag
    // In browser, this will be hydrated with client-side navigation
    html! {
        <a
            href={path}
            class={props.classes.clone()}
            ref={props.anchor_ref.clone()}
            target={props.target.clone()}
            title={props.title.clone()}
            onclick={props.onclick.clone()}
        >
            { props.children.clone() }
        </a>
    }
}
