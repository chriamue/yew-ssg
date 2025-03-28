use strum_macros::EnumIter;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq, Debug, EnumIter)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/about")]
    About,
    #[at("/readme")]
    Readme,
    #[not_found]
    #[at("/404")]
    NotFound,
}
