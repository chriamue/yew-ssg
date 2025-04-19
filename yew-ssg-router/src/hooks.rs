use yew::prelude::*;

pub use yew_router::hooks::*;

#[hook]
pub fn use_hash() -> Option<String> {
    #[cfg(not(feature = "ssg"))]
    {
        let location = yew_router::hooks::use_location()?;
        let hash = location.hash().trim_start_matches('#').to_string();
        if hash.is_empty() {
            None
        } else {
            Some(hash)
        }
    }
    #[cfg(feature = "ssg")]
    {
        None
    }
}
