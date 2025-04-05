use crate::about::AboutPage;
use crate::home::HomePage;
use crate::not_found::NotFoundPage;
use crate::readme::ReadmePage;
use crate::route::Route;
use yew::prelude::*;

pub fn switch_route(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <HomePage /> },
        Route::About => html! { <AboutPage /> },
        Route::Readme => html! { <ReadmePage /> },
        Route::NotFound => html! { <NotFoundPage /> },
    }
}
