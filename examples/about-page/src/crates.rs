use yew::prelude::*;

#[derive(Clone, Debug, Properties, PartialEq)]
pub struct CratesProps {
    pub id: String,
}

#[function_component(CratesPage)]
pub fn crates_page(props: &CratesProps) -> Html {
    let crate_id = props.id.clone();

    match crate_id.as_str() {
        "yew-ssg" => about_yew_ssg(),
        "yew-ssg-router" => about_yew_router_ssg(),
        _ => unknown_crate(&crate_id),
    }
}

pub fn about_yew_ssg() -> Html {
    html! {
        <div class="crate-container">
            <h1>{"Crate: yew-ssg"}</h1>
            <div class="crate-details">
                <h2>{"Static Site Generator for Yew 🚀"}</h2>
                <p class="crate-description">
                    {"A powerful static site generator (SSG) that pre-renders Yew applications into SEO-friendly HTML. It bridges the gap between single-page applications and search engine optimization."}
                </p>

                <h3>{"Core Features"}</h3>
                <ul class="crate-features">
                    <li>{"Pre-renders Yew applications to static HTML for improved SEO"}</li>
                    <li>{"Customizable HTML templates with variable substitution"}</li>
                    <li>{"Advanced attribute-based templating system"}</li>
                    <li>{"Extensible generator plugin architecture"}</li>
                    <li>{"Built-in SEO generators (meta tags, Open Graph, Twitter Cards)"}</li>
                </ul>

                <h3>{"Technical Details"}</h3>
                <dl class="crate-info">
                    <dt>{"Version"}</dt>
                    <dd>{"0.1.0 (Alpha)"}</dd>

                    <dt>{"Dependencies"}</dt>
                    <dd>{"async-trait, log, lol_html, minijinja, strum, yew"}</dd>

                    <dt>{"Usage"}</dt>
                    <dd>{"Primarily for Yew applications that need SEO benefits of static pre-rendering"}</dd>
                </dl>

                <h3>{"Example Usage"}</h3>
                <pre><code>{r#"
// Configure the SSG
let config = SsgConfigBuilder::new()
    .output_dir("dist")
    .global_metadata(HashMap::from([
        ("site_name".to_string(), "My Awesome Site".to_string()),
    ]))
    .build();

// Initialize the generator
let generator = StaticSiteGenerator::new(config)?;

// Generate static files
generator.generate::<Route, App>().await?;
                "#}</code></pre>
            </div>
        </div>
    }
}

pub fn about_yew_router_ssg() -> Html {
    html! {
        <div class="crate-container">
            <h1>{"Crate: yew-ssg-router"}</h1>
            <div class="crate-details">
                <h2>{"Router Integration for Static Site Generation 🧭"}</h2>
                <p class="crate-description">
                    {"A specialized router implementation for Yew that seamlessly integrates with yew-ssg, providing static pre-rendering capabilities while maintaining full client-side navigation after hydration."}
                </p>

                <h3>{"Core Features"}</h3>
                <ul class="crate-features">
                    <li>{"Drop-in replacement for yew-router with static generation support"}</li>
                    <li>{"Compatible API with standard yew-router components"}</li>
                    <li>{"Static navigation during pre-rendering phase"}</li>
                    <li>{"Full client-side routing after hydration"}</li>
                    <li>{"Feature-flag controlled switching between SSG and CSR modes"}</li>
                </ul>

                <h3>{"Technical Details"}</h3>
                <dl class="crate-info">
                    <dt>{"Version"}</dt>
                    <dd>{"0.1.0 (Alpha)"}</dd>

                    <dt>{"Dependencies"}</dt>
                    <dd>{"yew, yew-router, yew-router-macro"}</dd>

                    <dt>{"Usage"}</dt>
                    <dd>{"For Yew apps that need both static site generation and client-side routing"}</dd>
                </dl>

                <h3>{"Example Usage"}</h3>
                <pre><code>{r#"
// In your Cargo.toml
[dependencies]
yew_router = { package = "yew-ssg-router" }

[features]
ssg = ["yew/ssr", "yew-ssg", "yew_router/ssg"]

// In your code (identical to regular yew-router)
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Link<Route> to={Route::Home}>{"Home"}</Link<Route>>
            <Switch<Route> render={switch_route} />
        </BrowserRouter>
    }
}
                "#}</code></pre>
            </div>
        </div>
    }
}

fn unknown_crate(id: &str) -> Html {
    html! {
        <div class="crate-container">
            <h1>{"Unknown Crate"}</h1>
            <div class="crate-details error-container">
                <p class="error-message">
                    {format!("No information available for crate: '{}'", id)}
                </p>
                <p>{"Please check the URL and try again, or navigate to one of our documented crates:"}</p>
                <ul>
                    <li><a href="/crate/yew-ssg">{"yew-ssg"}</a></li>
                    <li><a href="/crate/yew-ssg-router">{"yew-ssg-router"}</a></li>
                </ul>
            </div>
        </div>
    }
}
