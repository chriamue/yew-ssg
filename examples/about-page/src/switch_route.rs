use crate::about::AboutPage;
use crate::crates::CratesPage;
use crate::home::HomePage;
use crate::not_found::NotFoundPage;
use crate::readme::ReadmePage;
use crate::route::{LocalizedRoute, Route};
use yew::prelude::*;
use yew_router::prelude::*;

pub fn switch_route(localized_route: LocalizedRoute) -> Html {
    // Extract the base route from the localized route
    let route = localized_route.get_route();

    match route {
        Route::Home => html! { <HomePage /> },
        Route::About => html! { <AboutPage /> },
        Route::Readme => html! { <ReadmePage /> },
        Route::Crate { id } => html! { <CratesPage id={id.clone()} /> },
        Route::NotFound => html! { <NotFoundPage /> },
    }
}
