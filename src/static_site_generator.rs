use minijinja::{context, Environment};
use std::error::Error;
use std::fmt::Debug;
use std::fs;
use std::sync::Arc;
use strum::IntoEnumIterator;
use yew::prelude::*;
use yew::ServerRenderer;
use yew_router::Routable;

const DEFAULT_TEMPLATE: &str = r#"<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <link rel="stylesheet" href="/styles.css">
        <script defer src="/app.js"></script>
    </head>
    <body>
        <div id="app">{{ content }}</div>
    </body>
</html>"#;

/// Minimal static site generator for Yew applications
pub struct StaticSiteGenerator {
    /// Output directory for generated files
    output_dir: String,
    /// Template environment
    template_env: Environment<'static>,
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
    /// Create a new static site generator with default template
    pub fn new(output_dir: &str) -> Self {
        let mut env = Environment::new();
        // Store template as a static str
        env.add_template("base", DEFAULT_TEMPLATE)
            .expect("Failed to add default template");

        Self {
            output_dir: output_dir.to_string(),
            template_env: env,
        }
    }

    /// Create a new static site generator with custom template
    pub fn with_template<S: Into<String>>(
        output_dir: &str,
        template: S,
    ) -> Result<Self, Box<dyn Error>> {
        let mut env = Environment::new();
        // Convert template to String and store it in a Box to make it 'static
        let template: String = template.into();
        let template = Box::leak(template.into_boxed_str());
        env.add_template("base", template)?;

        Ok(Self {
            output_dir: output_dir.to_string(),
            template_env: env,
        })
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
            let route_str = format!("{:?}", route);

            // Create the HTML for this route
            let content = self.render_route(&route, switch_fn.clone()).await?;

            // Create HTML document using template
            let html = self.wrap_html(&content, &route_str, &route_path)?;

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

    /// Wrap content in HTML using template
    fn wrap_html(&self, content: &str, title: &str, path: &str) -> Result<String, Box<dyn Error>> {
        let tmpl = self.template_env.get_template("base")?;

        let result = tmpl.render(context! {
            content => content,
            title => title,
            path => path,
            description => format!("Page for {}", title),
        })?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_template() {
        let template = r#"<!DOCTYPE html>
<html>
    <head>
        <title>{{ title }}</title>
        <meta name="description" content="{{ description }}">
    </head>
    <body>
        <main>{{ content }}</main>
    </body>
</html>"#;

        let generator = StaticSiteGenerator::with_template("dist", template).unwrap();
        let result = generator
            .wrap_html("test content", "Test Page", "/test")
            .unwrap();

        assert!(result.contains("Test Page"));
        assert!(result.contains("test content"));
        assert!(result.contains("Page for Test Page"));
    }
}
