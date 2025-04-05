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
    pub use yew_router::prelude::*;

    // When in SSG/SSR mode, use our static versions
    #[cfg(feature = "ssg")]
    pub use crate::{
        static_navigator::use_navigator, StaticLink as Link, StaticRouter as BrowserRouter,
        StaticSwitch as Switch,
    };
}

/// Get the current path being rendered during static generation
pub fn get_static_path() -> Option<String> {
    std::env::var("YEW_SSG_CURRENT_PATH").ok()
}

/// Check if the application is running in static generation mode
pub fn is_ssg_mode() -> bool {
    cfg!(feature = "ssg") || std::env::var("YEW_SSG_CURRENT_PATH").is_ok()
}
