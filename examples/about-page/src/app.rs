use crate::route::Route;
use crate::switch_route::switch_route;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <nav>
                <ul>
                    <li>
                        <Link<Route> to={Route::Home}>
                            { "Home" }
                        </Link<Route>>
                    </li>
                    <li>
                        <Link<Route> to={Route::About}>
                            { "About" }
                        </Link<Route>>
                    </li>
                    <li>
                        <Link<Route> to={Route::Readme}>
                            { "ReadMe" }
                        </Link<Route>>
                    </li>
                    <li>
                        <Link<Route> to={Route::NotFound}>
                            { "Not Found" }
                        </Link<Route>>
                    </li>
                </ul>
            </nav>
            <main>
                <Switch<Route> render={switch_route} />
            </main>
        </BrowserRouter>
    }
}
