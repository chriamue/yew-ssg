use crate::config::SsgConfig;
use crate::processors::{HtmlElementProcessor, TemplateVariableProcessor};
use log::{info, warn};
use minijinja::Environment;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use strum::IntoEnumIterator;
use yew::prelude::*;
use yew::ServerRenderer;
use yew_router::Routable;

const DEFAULT_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>{{ title | default(path) }}</title>
        {{ meta_tags | default("") | safe }}
        {{ open_graph | default("") | safe }}
        {{ twitter_card | default("") | safe }}
        {{ robots_meta | default("") | safe }}
        <link rel="stylesheet" href="/styles.css">
        <script defer src="/app.js"></script>
    </head>
    <body>
        <div id="app">{{ content | safe }}</div>
    </body>
</html>"#;

/// Static site generator for Yew applications.
#[derive(Debug)]
pub struct StaticSiteGenerator {
    /// Configuration for the generator.
    pub config: SsgConfig,
    /// Template environment.
    template_env: Environment<'static>,
}

/// Wrapper for switch function to enable PartialEq for Yew components.
#[derive(Clone)]
struct SwitchFn<R: Routable + Clone + 'static>(Arc<dyn Fn(R) -> Html + Send + Sync + 'static>);

impl<R: Routable + Clone + 'static> PartialEq for SwitchFn<R> {
    fn eq(&self, _other: &Self) -> bool {
        true // All switch functions considered equal
    }
}

impl StaticSiteGenerator {
    /// Create a new static site generator from the provided configuration.
    pub fn new(config: SsgConfig) -> Result<Self, Box<dyn Error>> {
        let mut env = Environment::new();
        let mut template_loaded = false;
        let mut template_source = String::from("built-in default");

        // 1. Try loading from template_path
        if let Some(path) = &config.template_path {
            if path.exists() {
                match fs::read_to_string(path) {
                    Ok(content) => {
                        let static_template = Box::leak(content.into_boxed_str());
                        env.add_template("base", static_template)?;
                        template_loaded = true;
                        template_source = format!("file ({:?})", path);
                    }
                    Err(e) => warn!("Failed to read template: {}", e),
                }
            } else {
                warn!("Template path {:?} does not exist", path);
            }
        }

        // 2. Try using default_template string from config
        if !template_loaded && !config.default_template.is_empty() {
            let static_template = Box::leak(config.default_template.clone().into_boxed_str());
            env.add_template("base", static_template)?;
            template_loaded = true;
            template_source = String::from("config.default_template");
        }

        // 3. Fallback to built-in default template
        if !template_loaded {
            env.add_template("base", DEFAULT_TEMPLATE)?;
            info!("Using built-in default HTML template");
        }

        let mut config = config;
        if config.processors.is_empty() {
            info!("Adding default processors");

            // Add HTML element processor for placeholder elements
            let html_processor = HtmlElementProcessor::new("data-ssg");
            config.processors.add(html_processor);

            // Add template variable processor for {{var}} syntax
            let variable_processor = TemplateVariableProcessor::new();
            config.processors.add(variable_processor);
        }

        Ok(Self {
            config,
            template_env: env,
        })
    }

    /// Generate static HTML files for all routes.
    pub async fn generate<R, F>(&self, switch_fn: F) -> Result<(), Box<dyn Error>>
    where
        R: Routable + IntoEnumIterator + Clone + PartialEq + Debug + Send + 'static,
        F: Fn(R) -> Html + Clone + Send + Sync + 'static,
    {
        fs::create_dir_all(&self.config.output_dir)?;
        let switch_fn = SwitchFn(Arc::new(switch_fn));

        for route in R::iter() {
            let route_path = route.to_path();
            info!("Generating route: {}", route_path);

            // 1. Render the Yew component to HTML
            let content = self.render_route(&route, switch_fn.clone()).await?;

            // 2. Get metadata and run generators
            let metadata = self.config.get_metadata_for_route(&route_path);
            let generator_outputs =
                self.config
                    .generators
                    .run_all(&route_path, &content, &metadata)?;

            // 3. Run processors on the content
            let processed_content = self.config.processors.process_all(
                &content,
                &metadata,
                &generator_outputs,
                &content,
            )?;

            // 4. Create the final HTML
            let html = self.wrap_html(
                &processed_content,
                &route_path,
                &metadata,
                &generator_outputs,
            )?;

            // 5. Write to output file
            let (dir_path, file_path) = self.determine_output_path(&route_path);
            fs::create_dir_all(&dir_path)?;
            fs::write(&file_path, html)?;
            info!("  -> Saved to {:?}", file_path);
        }

        Ok(())
    }

    /// Determine output directory and file path for a route.
    fn determine_output_path(&self, route_path: &str) -> (PathBuf, PathBuf) {
        if route_path == "/" {
            (
                self.config.output_dir.clone(),
                self.config.output_dir.join("index.html"),
            )
        } else {
            let path_component = route_path.trim_start_matches('/');
            let dir = self.config.output_dir.join(path_component);
            (dir.clone(), dir.join("index.html"))
        }
    }

    /// Render a Yew component to HTML using server-side rendering.
    async fn render_route<R>(
        &self,
        route: &R,
        switch_fn: SwitchFn<R>,
    ) -> Result<String, Box<dyn Error>>
    where
        R: Routable + Clone + PartialEq + Send + 'static,
    {
        #[derive(Properties, PartialEq)]
        struct RouteRendererProps<R>
        where
            R: Routable + Clone + PartialEq + Send + 'static,
        {
            route: R,
            switch_fn: SwitchFn<R>,
        }

        #[function_component]
        fn RouteRenderer<R>(props: &RouteRendererProps<R>) -> Html
        where
            R: Routable + Clone + PartialEq + Send + 'static,
        {
            (props.switch_fn.0)(props.route.clone())
        }

        let props = RouteRendererProps {
            route: route.clone(),
            switch_fn,
        };
        let renderer = ServerRenderer::<RouteRenderer<R>>::with_props(move || props);
        Ok(renderer.render().await)
    }

    /// Create the final HTML by combining rendered content, metadata, and generator outputs.
    fn wrap_html(
        &self,
        content: &str,
        path: &str,
        metadata: &HashMap<String, String>,
        generator_outputs: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        let tmpl = self.template_env.get_template("base")?;
        let mut context = HashMap::new();

        // Create a struct to hold our values to ensure they live long enough
        struct TemplateContext {
            values: HashMap<String, String>,
        }

        let mut template_context = TemplateContext {
            values: HashMap::new(),
        };

        // Add primary content and path
        template_context
            .values
            .insert("content".to_string(), content.to_string());
        template_context
            .values
            .insert("path".to_string(), path.to_string());

        // Add metadata values
        for (key, value) in metadata {
            template_context.values.insert(key.clone(), value.clone());
        }

        // Add generator outputs (overriding metadata with same keys)
        for (key, value) in generator_outputs {
            template_context.values.insert(key.clone(), value.clone());
        }

        // Add fallbacks for essential items if missing
        if !template_context.values.contains_key("title") {
            let fallback_title = format!("Page: {}", path);
            template_context
                .values
                .insert("title".to_string(), fallback_title);
            warn!("No title provided for route '{}'", path);
        }

        // Create the context with references to our stored values
        for (key, value) in &template_context.values {
            context.insert(key.as_str(), value.as_str());
        }

        // Render the template with Minijinja
        Ok(tmpl.render(context)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SsgConfigBuilder;

    #[test]
    fn test_wrap_html_with_basic_context_and_metadata() {
        // Setup: Create template with test placeholders
        let template = r#"<!DOCTYPE html><html><head><title>{{ title }}</title>
        <meta name="custom" content="{{ custom_meta }}"></head>
        <body><main>{{ content | safe }}</main><p>Route: {{ path }}</p></body></html>"#;

        // Configure generator with template
        let config = SsgConfigBuilder::new()
            .output_dir("test_dist")
            .default_template_string(template.to_string())
            .build();

        let generator = StaticSiteGenerator::new(config).unwrap();

        // Test data
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Test Title".to_string());
        metadata.insert("custom_meta".to_string(), "Custom Value".to_string());
        let generator_outputs = HashMap::new();
        let content = "<div>Content</div>";
        let path = "/test";

        // Execute
        let result = generator
            .wrap_html(content, path, &metadata, &generator_outputs)
            .unwrap();

        // Verify
        assert!(result.contains("<title>Test Title</title>"));
        assert!(result.contains("<meta name=\"custom\" content=\"Custom Value\">"));
        assert!(result.contains("<div>Content</div>"));
        assert!(result.contains("<p>Route: /test</p>"));
    }

    #[test]
    fn test_wrap_html_with_generator_output_overriding_metadata() {
        // Template with generator placeholders
        let template = r#"<!DOCTYPE html><html><head>{{ title | safe }}{{ meta_tags | safe }}</head>
        <body><main>{{ content | safe }}</main><p>Meta: {{ description }}</p></body></html>"#;

        // Setup generator
        let config = SsgConfigBuilder::new()
            .output_dir("test_dist")
            .default_template_string(template.to_string())
            .build();

        let generator = StaticSiteGenerator::new(config).unwrap();

        // Test data
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Metadata Title".to_string());
        metadata.insert(
            "description".to_string(),
            "Metadata Description".to_string(),
        );

        let mut generator_outputs = HashMap::new();
        generator_outputs.insert(
            "title".to_string(),
            "<title>Generated Title</title>".to_string(),
        );
        generator_outputs.insert(
            "meta_tags".to_string(),
            "<meta name=\"keywords\" content=\"generated\">".to_string(),
        );

        // Execute
        let result = generator
            .wrap_html("<p>Test</p>", "/test", &metadata, &generator_outputs)
            .unwrap();

        // Verify
        assert!(result.contains("<title>Generated Title</title>"));
        assert!(!result.contains("Metadata Title"));
        assert!(result.contains("<meta name=\"keywords\" content=\"generated\">"));
        assert!(result.contains("<p>Meta: Metadata Description</p>"));
    }

    #[test]
    fn test_wrap_html_with_title_fallback() {
        // Create minimal template
        let template = r#"<!DOCTYPE html><html><head><title>{{ title }}</title></head><body>{{ content | safe }}</body></html>"#;

        // Setup generator
        let config = SsgConfigBuilder::new()
            .output_dir("test_dist")
            .default_template_string(template.to_string())
            .build();

        let generator = StaticSiteGenerator::new(config).unwrap();

        // Test with no title in metadata or generator outputs
        let result = generator
            .wrap_html("<p>Test</p>", "/test", &HashMap::new(), &HashMap::new())
            .unwrap();

        // Verify fallback title is used
        assert!(result.contains("<title>Page: /test</title>"));
    }

    #[test]
    fn test_determine_output_path() {
        let config = SsgConfigBuilder::new().output_dir("dist").build();
        let generator = StaticSiteGenerator::new(config).unwrap();

        // Root path
        let (dir, file) = generator.determine_output_path("/");
        assert_eq!(dir, PathBuf::from("dist"));
        assert_eq!(file, PathBuf::from("dist/index.html"));

        // Regular path
        let (dir, file) = generator.determine_output_path("/about");
        assert_eq!(dir, PathBuf::from("dist/about"));
        assert_eq!(file, PathBuf::from("dist/about/index.html"));

        // Nested path
        let (dir, file) = generator.determine_output_path("/blog/post-1");
        assert_eq!(dir, PathBuf::from("dist/blog/post-1"));
        assert_eq!(file, PathBuf::from("dist/blog/post-1/index.html"));
    }
}
