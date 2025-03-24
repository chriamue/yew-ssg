use yew::prelude::*;

#[function_component(HomePage)]
pub fn home_page() -> Html {
    html! {
        <div>
            <h1>{"Welcome to Home Page"}</h1>
            <p>{"This is a simple example using yew-ssg"}</p>
        </div>
    }
}
