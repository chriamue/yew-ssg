use yew::prelude::*;

/// Properties for the StaticRouter component used only during SSG
#[derive(Properties, PartialEq, Clone)]
pub struct StaticRouterProps {
    #[prop_or_default]
    pub children: Html,
    #[prop_or_default]
    pub basename: Option<String>,
}

/// A simplified context provider that holds the current SSG path
#[derive(Clone, Debug, PartialEq)]
pub struct SsgPathContext {
    pub path: String,
    pub basename: Option<String>,
}

/// A very simple router that just provides the current path from the SSG environment
#[function_component(StaticRouter)]
pub fn static_router(props: &StaticRouterProps) -> Html {
    // Get the current path from the environment variable
    let current_path = crate::get_static_path().unwrap_or_else(|| "/".to_string());

    // Create a context with the path
    let context = SsgPathContext {
        path: current_path,
        basename: props.basename.clone(),
    };

    // We can't easily recreate yew_router's contexts since they have private fields,
    // so we'll just provide our own simple context for SSG use
    html! {
        <ContextProvider<SsgPathContext> context={context}>
            { props.children.clone() }
        </ContextProvider<SsgPathContext>>
    }
}

/// Get the current SSG path from context
#[hook]
pub fn use_ssg_path() -> Option<SsgPathContext> {
    use_context::<SsgPathContext>()
}
