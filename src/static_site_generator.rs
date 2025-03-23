use std::error::Error;
use std::fmt::Debug;
use std::fs;
use std::sync::Arc;
use strum::IntoEnumIterator;
use yew::prelude::*;
use yew::ServerRenderer;
use yew_router::Routable;

/// Minimal static site generator for Yew applications
pub struct StaticSiteGenerator {
    /// Output directory for generated files
    output_dir: String,
}

/// Wrapper for switch function that implements PartialEq
struct SwitchFn<R: Routable + Clone + 'static>(Arc<dyn Fn(R) -> Html + Send + Sync + 'static>);

impl<R: Routable + Clone + 'static> Clone for SwitchFn<R> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<R: Routable + Clone + 'static> PartialEq for SwitchFn<R> {
    fn eq(&self, _other: &Self) -> bool {
        // Switch functions are always considered equal for our use case
        true
    }
}

impl StaticSiteGenerator {
    /// Create a new static site generator
    pub fn new(output_dir: &str) -> Self {
        Self {
            output_dir: output_dir.to_string(),
        }
    }

    /// Generate static HTML files for the given route type
    pub async fn generate<R, F>(&self, switch_fn: F) -> Result<(), Box<dyn Error>>
    where
        R: Routable + IntoEnumIterator + Clone + PartialEq + Debug + Send + 'static,
        F: Fn(R) -> Html + Clone + Send + Sync + 'static,
    {
        // Create output directory
        fs::create_dir_all(&self.output_dir)?;

        // Wrap the switch function
        let switch_fn = SwitchFn(Arc::new(switch_fn));

        // Use the route discriminants to generate all possible routes
        for route in R::iter() {
            // Get the path for this route
            let route_path = route.to_path();

            // Create the HTML for this route
            let content = self.render_route(&route, switch_fn.clone()).await?;

            // Create a minimal HTML document
            let html = self.wrap_html(&content);

            // Determine file path
            let (dir_path, file_path) = if route_path == "/" {
                (
                    self.output_dir.clone(),
                    format!("{}/index.html", self.output_dir),
                )
            } else {
                let path_component = route_path.trim_start_matches('/').trim_end_matches('/');
                let dir = format!("{}/{}", self.output_dir, path_component);
                (dir.clone(), format!("{}/index.html", dir))
            };

            // Create directory if it doesn't exist
            fs::create_dir_all(&dir_path)?;

            // Write HTML to file
            fs::write(&file_path, html)?;
        }

        Ok(())
    }

    /// Render a route to HTML content
    async fn render_route<R>(
        &self,
        route: &R,
        switch_fn: SwitchFn<R>,
    ) -> Result<String, Box<dyn Error>>
    where
        R: Routable + Clone + PartialEq + Send + 'static,
    {
        // Component to render a route using the switch function
        #[derive(Properties, PartialEq)]
        struct RouteRenderer<R>
        where
            R: Routable + Clone + PartialEq + Send + 'static,
        {
            route: R,
            switch_fn: SwitchFn<R>,
        }

        #[function_component]
        fn RouteRendererComponent<R>(props: &RouteRenderer<R>) -> Html
        where
            R: Routable + Clone + PartialEq + Send + 'static,
        {
            let route = props.route.clone();
            (props.switch_fn.0)(route)
        }

        // Create the renderer
        let route_clone = route.clone();
        let renderer =
            ServerRenderer::<RouteRendererComponent<R>>::with_props(move || RouteRenderer {
                route: route_clone.clone(),
                switch_fn: switch_fn.clone(),
            });

        // Render the component
        let content = renderer.render().await;

        Ok(content)
    }

    /// Wrap content in minimal HTML
    fn wrap_html(&self, content: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <link rel="stylesheet" href="/styles.css">
        <script defer src="/app.js"></script>
    </head>
    <body>
        <div id="app">{}</div>
    </body>
</html>"#,
            content
        )
    }
}
