pub use yew_router::*;
pub use yew_router_macro::Routable;

mod static_link;
mod static_navigator;
mod static_router;
mod static_switch;
pub mod utils;

pub use static_link::StaticLink;
pub use static_navigator::{use_navigator, StaticNavigator, StaticNavigatorProvider};
pub use static_router::StaticRouter;
pub use static_switch::StaticSwitch;

pub mod prelude {
    pub use crate::{get_static_path, is_ssg_mode};

    // Import necessary types from yew_router without the components we want to replace
    pub use yew_router::prelude::{use_location, use_route, Location, LocationHandle, Routable};

    // Conditionally import the right components based on feature flag
    #[cfg(feature = "ssg")]
    pub use crate::{
        static_link::StaticLink as Link, static_navigator::use_navigator,
        static_router::StaticRouter as BrowserRouter, static_switch::StaticSwitch as Switch,
    };

    // Only when NOT in SSG mode, import the original router components
    #[cfg(not(feature = "ssg"))]
    pub use yew_router::prelude::{use_navigator, BrowserRouter, Link, Switch};
}

/// Get the current path being rendered during static generation
pub fn get_static_path() -> Option<String> {
    std::env::var("YEW_SSG_CURRENT_PATH").ok()
}

/// Check if the application is running in static generation mode
pub fn is_ssg_mode() -> bool {
    if cfg!(feature = "ssg") {
        true
    } else {
        false
    }
}
