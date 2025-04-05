use crate::static_navigator::StaticNavigatorProvider;
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
    // Wrap with StaticNavigatorProvider
    html! {
        <StaticNavigatorProvider basename={props.basename.clone()}>
            { props.children.clone() }
        </StaticNavigatorProvider>
    }
}
