use log::{info, warn};
use yew::prelude::*;
use yew_router::Routable;

/// Properties for the StaticSwitch component
#[derive(Properties, PartialEq, Clone)]
pub struct StaticSwitchProps<R>
where
    R: Routable + 'static,
{
    pub render: Callback<R, Html>,
}

/// A static version of Switch that works during SSG
#[function_component(StaticSwitch)]
pub fn static_switch<R>(props: &StaticSwitchProps<R>) -> Html
where
    R: Routable + Clone + PartialEq + std::fmt::Debug + 'static,
{
    // Get the current path directly from the environment variable during SSG
    let current_path = crate::get_static_path().unwrap_or_else(|| "/".to_string());

    info!("StaticSwitch: Current path = {}", current_path);

    // Try to match the route
    let matched = R::recognize(&current_path);

    match matched {
        Some(route) => {
            info!("StaticSwitch: Matched route = {:?}", route);
            props.render.emit(route)
        }
        None => {
            warn!("StaticSwitch: No route matched for path = {}", current_path);
            // Try to render the "not found" route
            if let Some(not_found_route) = R::not_found_route() {
                info!(
                    "StaticSwitch: Rendering not_found route = {:?}",
                    not_found_route
                );
                props.render.emit(not_found_route)
            } else {
                warn!("StaticSwitch: No not_found route defined");
                // If no route matched and no not_found route is defined, render nothing
                Html::default()
            }
        }
    }
}
