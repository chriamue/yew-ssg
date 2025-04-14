use crate::LocalizedRoutable;
use yew::prelude::*;

/// A switch component specialized for localized routes
#[derive(Properties, PartialEq)]
pub struct LocalizedSwitchProps<R>
where
    R: LocalizedRoutable + 'static,
{
    pub render: Callback<R, Html>,
}

/// Renders the appropriate component based on the current localized route
///
/// This component handles both localized routes (e.g., /en/home) and
/// default routes (e.g., /home), applying the correct language context.
#[function_component(LocalizedSwitch)]
pub fn localized_switch<R>(props: &LocalizedSwitchProps<R>) -> Html
where
    R: LocalizedRoutable + 'static,
{
    // For SSG, get the path from environment
    #[cfg(feature = "ssg")]
    let current_path = { crate::get_static_path().unwrap_or_else(|| "/".to_string()) };

    // For client-side rendering, use the location
    #[cfg(not(feature = "ssg"))]
    let current_path = {
        let location = yew_router::hooks::use_location().unwrap();
        location.path().to_string()
    };

    log::info!("LocalizedSwitch: Current path = {}", current_path);

    // Try to match the route
    let matched = R::recognize(&current_path);

    match matched {
        Some(route) => {
            log::info!("LocalizedSwitch: Matched localized route = {:?}", route);
            props.render.emit(route)
        }
        None => {
            log::warn!(
                "LocalizedSwitch: No route matched for path = {}",
                current_path
            );
            // Try to render the "not found" route
            if let Some(not_found_route) = R::not_found_route() {
                log::info!(
                    "LocalizedSwitch: Rendering not_found route = {:?}",
                    not_found_route
                );
                props.render.emit(not_found_route)
            } else {
                // If no route matched and no not_found route is defined, render nothing
                Html::default()
            }
        }
    }
}
