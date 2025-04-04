pub use yew_router::*;

// Also specifically re-export the macro
pub use yew_router_macro::Routable;

// Re-export the prelude for convenience
pub mod prelude {
    pub use yew_router::prelude::*;

    // Add SSG-specific items to the prelude if needed
    pub use crate::{get_static_path, is_ssg_mode};
}

// --- SSG Specific Functionality ---
pub fn is_ssg_mode() -> bool {
    cfg!(feature = "ssg")
}

pub fn get_static_path() -> Option<String> {
    option_env!("YEW_SSG_CURRENT_PATH").map(ToString::to_string)
}
