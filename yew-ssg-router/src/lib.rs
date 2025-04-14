pub use yew_router::*;
pub use yew_router_macro::Routable;

mod static_link;
mod static_navigator;
mod static_router;
mod static_switch;
pub mod utils;

mod language_context;
mod localized_link;
mod localized_routable;
mod localized_switch;
mod route_language;

pub use static_link::StaticLink;
pub use static_navigator::{use_navigator, StaticNavigator, StaticNavigatorProvider};
pub use static_router::StaticRouter;
pub use static_switch::StaticSwitch;

pub use language_context::{use_language, LanguageContext, LanguageProvider, TextDirection};
pub use localized_link::{LocalizedLink, LocalizedLinkProps};
pub use localized_routable::LocalizedRoutable;
pub use localized_switch::{LocalizedSwitch, LocalizedSwitchProps};
pub use route_language::{use_route_language, RouteLanguageExtractor};

pub mod prelude {
    pub use crate::{get_static_path, is_ssg_mode};

    // Import necessary types from yew_router without the components we want to replace
    pub use crate::route_language::{use_route_language, RouteLanguageExtractor};
    pub use yew_router::prelude::{use_location, use_route, Location, LocationHandle, Routable};

    // Conditionally import the right components based on feature flag
    #[cfg(feature = "ssg")]
    pub use crate::{
        // Localized router components
        localized_link::LocalizedLink,
        localized_switch::LocalizedSwitch,
        // Static router components
        static_link::StaticLink as Link,
        static_navigator::use_navigator,
        static_router::StaticRouter as BrowserRouter,
        static_switch::StaticSwitch as Switch,
    };

    // When NOT in SSG mode, import the original router components
    #[cfg(not(feature = "ssg"))]
    pub use yew_router::prelude::{use_navigator, BrowserRouter, Link, Switch};

    // When NOT in SSG mode, use the client-side localized components
    #[cfg(not(feature = "ssg"))]
    pub use crate::{localized_link::LocalizedLink, localized_switch::LocalizedSwitch};
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

mod tests;
