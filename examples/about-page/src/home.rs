use yew::prelude::*;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    html! {
        <div>
            <h1>{"Welcome to the Home Page!"}</h1>
            <p>{"This is the home page of our website."}</p>
        </div>
    }
}
