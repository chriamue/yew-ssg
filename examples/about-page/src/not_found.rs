use crate::route::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(NotFoundPage)]
pub fn not_found_page() -> Html {
    // Get the navigator
    let navigator = use_navigator().unwrap_or_else(|| {
        // This should not happen in normal circumstances
        panic!("Navigator not available");
    });

    // Get the current path from navigator
    let current_path = navigator
        .basename()
        .map(|base| format!("{}", base))
        .unwrap_or_else(|| "/".to_string());

    // Create a callback to navigate back to home
    let go_home = {
        let navigator = navigator.clone();
        Callback::from(move |_: MouseEvent| {
            navigator.push(&Route::Home);
        })
    };

    html! {
        <div class="not-found-container">
            <h1>{"404 - Page Not Found"}</h1>
            <div class="error-details">
                <p>{"We couldn't find what you were looking for."}</p>
                <p class="navigator-info">
                    {"Navigator information: "}
                    <code>{format!("Basename: {}", current_path)}</code>
                </p>
                <p class="navigator-test">
                    {"Testing navigator availability: "}
                    <code>{"Available"}</code>
                </p>
            </div>
            <div class="not-found-actions">
                <button onclick={go_home} class="back-home-button">
                    {"Back to Home (Using Navigator)"}
                </button>
            </div>
        </div>
    }
}
