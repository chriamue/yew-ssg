use crate::generator::Generator;
use crate::generators::{OpenGraphGenerator, TwitterCardGenerator};
use minijinja::Environment;
use std::collections::HashMap;
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
    global_metadata: HashMap<String, String>,
    route_metadata: HashMap<String, HashMap<String, String>>,
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
            global_metadata: HashMap::new(),
            route_metadata: HashMap::new(),
        }
    }

    /// Create a new static site generator with custom template
    pub fn with_template<S: Into<String>>(
        output_dir: &str,
        template: S,
    ) -> Result<Self, Box<dyn Error>> {
        let mut env = Environment::new();

        let template: String = template.into();
        let template = Box::leak(template.into_boxed_str());
        env.add_template("base", template)?;

        Ok(Self {
            output_dir: output_dir.to_string(),
            template_env: env,
            global_metadata: HashMap::new(),
            route_metadata: HashMap::new(),
        })
    }

    pub fn with_global_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.global_metadata = metadata;
        self
    }

    /// Set metadata for a specific route
    pub fn with_route_metadata(mut self, route: &str, metadata: HashMap<String, String>) -> Self {
        self.route_metadata.insert(route.to_string(), metadata);
        self
    }

    /// Get combined metadata for a specific route
    fn get_metadata_for_route(&self, route_path: &str) -> HashMap<String, String> {
        let mut metadata = self.global_metadata.clone();

        if let Some(route_specific) = self.route_metadata.get(route_path) {
            // Route-specific metadata overrides global metadata
            for (key, value) in route_specific {
                metadata.insert(key.clone(), value.clone());
            }
        }

        metadata
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

            // Empty generator outputs for now (would be filled by actual generators)
            let generator_outputs = HashMap::new();

            // Create HTML document using template
            let html = self.wrap_html(&content, &route_str, &route_path, &generator_outputs)?;

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
    fn wrap_html(
        &self,
        content: &str,
        title: &str,
        path: &str,
        generator_outputs: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        let tmpl = self.template_env.get_template("base")?;

        // Get metadata for this route
        let metadata = self.get_metadata_for_route(path);

        // Create base context
        let mut context_data = HashMap::new();
        context_data.insert("content".to_string(), content.to_string());
        context_data.insert("title".to_string(), title.to_string());
        context_data.insert("path".to_string(), path.to_string());

        // Add description (preferring metadata over default)
        let description = metadata
            .get("description")
            .cloned()
            .unwrap_or_else(|| format!("Page for {}", title));
        context_data.insert("description".to_string(), description);

        // Add other metadata
        for (key, value) in &metadata {
            if !context_data.contains_key(key) {
                context_data.insert(key.clone(), value.clone());
            }
        }

        // Create and add Open Graph tags if the template contains {{ open_graph_tags }}
        let html_template = tmpl.source();
        if html_template.contains("{{ open_graph_tags }}") {
            let og_generator = OpenGraphGenerator {
                site_name: metadata
                    .get("site_name")
                    .cloned()
                    .unwrap_or_else(|| "".to_string()),
                default_image: metadata
                    .get("default_image")
                    .cloned()
                    .unwrap_or_else(|| "".to_string()),
            };

            let og_tags = og_generator
                .generate(path, content, &metadata)
                .unwrap_or_default();
            context_data.insert("open_graph_tags".to_string(), og_tags);
        }

        // Create and add Twitter Card tags if the template contains {{ twitter_card_tags }}
        if html_template.contains("{{ twitter_card_tags }}") {
            let twitter_generator = TwitterCardGenerator {
                twitter_site: metadata.get("twitter_site").cloned(),
                default_card_type: metadata
                    .get("twitter_card_type")
                    .cloned()
                    .unwrap_or_else(|| "summary".to_string()),
            };

            let twitter_tags = twitter_generator
                .generate(path, content, &metadata)
                .unwrap_or_default();
            context_data.insert("twitter_card_tags".to_string(), twitter_tags);
        }

        // Add generator outputs
        for (key, value) in generator_outputs {
            context_data.insert(key.clone(), value.clone());
        }

        // Render the template with the context
        let result = tmpl.render(context_data)?;
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

        let generator_outputs = HashMap::new();

        let result = generator
            .wrap_html("test content", "Test Page", "/test", &generator_outputs)
            .unwrap();

        assert!(result.contains("Test Page"));
        assert!(result.contains("test content"));
        assert!(result.contains("Page for Test Page"));
    }
}
