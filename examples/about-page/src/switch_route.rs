use crate::about::AboutPage;
use crate::home::HomePage;
use crate::route::Route;
use yew::prelude::*;

pub fn switch_route(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <HomePage /> },
        Route::About => html! { <AboutPage /> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}
