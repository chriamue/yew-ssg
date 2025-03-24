use yew::prelude::*;

#[function_component(AboutPage)]
pub fn about_page() -> Html {
    html! {
        <div>
            <h1>{"About Page"}</h1>
            <p>{"This is the about page of our example application"}</p>
        </div>
    }
}
