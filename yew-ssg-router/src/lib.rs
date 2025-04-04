pub use yew_router::*;
pub use yew_router_macro::Routable;

mod static_link;
mod static_router;
mod static_switch;
pub mod utils;

pub use static_link::StaticLink;
pub use static_router::StaticRouter;
pub use static_switch::StaticSwitch;

pub mod prelude {
    pub use yew_router::prelude::*;

    // Re-export our SSG helper functions
    pub use crate::{get_static_path, is_ssg_mode};

    // When in SSG mode, use our static versions
    #[cfg(feature = "ssg")]
    pub use crate::StaticRouter as BrowserRouter;

    #[cfg(feature = "ssg")]
    pub use crate::StaticLink as Link;

    #[cfg(feature = "ssg")]
    pub use crate::StaticSwitch as Switch;
}

/// Get the current path being rendered during SSG
pub fn get_static_path() -> Option<String> {
    std::env::var("YEW_SSG_CURRENT_PATH").ok()
}

/// Check if the application is running in SSG mode
pub fn is_ssg_mode() -> bool {
    cfg!(feature = "ssg") || std::env::var("YEW_SSG_CURRENT_PATH").is_ok()
}
