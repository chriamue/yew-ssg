use crate::config::SsgConfig;
use minijinja::Environment;
use regex::Regex;
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
<html>
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>{{ title | default(path) }}</title>
        {{ meta_tags | default("") }}
        {{ open_graph | default("") }}
        {{ twitter_card | default("") }}
        <link rel="stylesheet" href="/styles.css">
        <script defer src="/app.js"></script>
    </head>
    <body>
        <div id="app">{{ content }}</div>
    </body>
</html>"#;

/// Static site generator for Yew applications.
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
                    Err(e) => eprintln!("Warning: Failed to read template: {}", e),
                }
            } else {
                eprintln!("Warning: Template path {:?} does not exist", path);
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
            eprintln!("Info: Using built-in default HTML template");
        }

        println!("Info: Template initialized from {}", template_source);

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
            println!("Generating route: {}", route_path);

            // 1. Render the Yew component to HTML
            let content = self.render_route(&route, switch_fn.clone()).await?;

            // 2. Get metadata and run generators
            let metadata = self.config.get_metadata_for_route(&route_path);
            let generator_outputs =
                self.config
                    .generators
                    .run_all(&route_path, &content, &metadata)?;

            // 3. Create the final HTML
            let html = self.wrap_html(&content, &route_path, &metadata, &generator_outputs)?;

            // 4. Write to output file
            let (dir_path, file_path) = self.determine_output_path(&route_path);
            fs::create_dir_all(&dir_path)?;
            fs::write(&file_path, html)?;
            println!("  -> Saved to {:?}", file_path);
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

    fn post_process_data_attributes(
        &self,
        html: &str,
        metadata: &HashMap<String, String>,
        generator_outputs: &HashMap<String, String>,
        content: &str,
    ) -> String {
        let mut document = html.to_string();

        // Process title element
        if let Some(title) = metadata.get("title") {
            let re = Regex::new(r#"<title data-ssg="title">.*?</title>"#).unwrap();
            document = re
                .replace_all(&document, &format!("<title>{}</title>", title))
                .to_string();
        }

        // Process meta description
        if let Some(description) = metadata.get("description") {
            let re = Regex::new(r#"<meta name="description" data-ssg="description" content=".*?""#)
                .unwrap();
            document = re
                .replace_all(
                    &document,
                    &format!(r#"<meta name="description" content="{}""#, description),
                )
                .to_string();
        }

        // Process placeholders for generator outputs
        for (key, value) in generator_outputs {
            let pattern = format!(r#"<meta data-ssg-placeholder="{}" .*?>"#, key);
            let re = Regex::new(&pattern).unwrap();
            document = re.replace_all(&document, value).to_string();
        }

        // Handle content div
        let re = Regex::new(r#"<div id="app" data-ssg="content">.*?</div>"#).unwrap();
        document = re
            .replace_all(&document, &format!(r#"<div id="app">{}</div>"#, content))
            .to_string();

        document
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

        // Add primary content and path
        context.insert("content".to_string(), content.to_string());
        context.insert("path".to_string(), path.to_string());

        // Add metadata values
        for (key, value) in metadata {
            context.insert(key.clone(), value.clone());
        }

        // Add generator outputs (overriding metadata with same keys)
        for (key, value) in generator_outputs {
            context.insert(key.clone(), value.clone());
        }

        // Add fallbacks for essential items if missing
        if !context.contains_key("title") {
            context.insert("title".to_string(), format!("Page: {}", path));
            eprintln!("Warning: No title provided for route '{}'", path);
        }

        // Render the template with Minijinja
        let mut html = tmpl.render(context)?;

        // Post-process data-ssg attributes for better trunk compatibility
        html = self.post_process_data_attributes(&html, metadata, generator_outputs, content);

        Ok(html)
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
        <body><main>{{ content }}</main><p>Route: {{ path }}</p></body></html>"#;

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
        let template = r#"<!DOCTYPE html><html><head>{{ title }}{{ meta_tags }}</head>
        <body><main>{{ content }}</main><p>Meta: {{ description }}</p></body></html>"#;

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
    fn test_post_process_data_attributes() {
        // Create a minimal generator instance
        let config = SsgConfigBuilder::new().output_dir("test_dist").build();
        let generator = StaticSiteGenerator::new(config).unwrap();

        // Create test HTML with data-ssg attributes - simpler format to ensure matching works
        let html = r#"<!DOCTYPE html><html><head><title data-ssg="title">Default Title</title><meta name="description" data-ssg="description" content="Default description"><meta data-ssg-placeholder="meta_tags" content="seo"><meta data-ssg-placeholder="open_graph" content="og"></head><body><div id="app" data-ssg="content"></div></body></html>"#;

        // Create test metadata and generator outputs
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Real Title".to_string());
        metadata.insert("description".to_string(), "Real description".to_string());

        let mut generator_outputs = HashMap::new();
        generator_outputs.insert(
            "meta_tags".to_string(),
            "<meta name=\"keywords\" content=\"test,keywords\">".to_string(),
        );
        generator_outputs.insert(
            "open_graph".to_string(),
            "<meta property=\"og:title\" content=\"OG Title\">".to_string(),
        );

        let content = "<p>Generated content</p>";

        // Process the HTML
        let processed =
            generator.post_process_data_attributes(html, &metadata, &generator_outputs, content);

        // Print the processed HTML for debugging
        println!("Processed HTML: {}", processed);

        // Verify replacements were made correctly
        assert!(processed.contains("<title>Real Title</title>"));
        assert!(!processed.contains("data-ssg=\"title\""));

        assert!(processed.contains("<meta name=\"description\" content=\"Real description\""));
        assert!(!processed.contains("data-ssg=\"description\""));

        assert!(processed.contains("<meta name=\"keywords\" content=\"test,keywords\">"));
        assert!(!processed.contains("data-ssg-placeholder=\"meta_tags\""));

        assert!(processed.contains("<meta property=\"og:title\" content=\"OG Title\">"));
        assert!(!processed.contains("data-ssg-placeholder=\"open_graph\""));

        assert!(processed.contains("<div id=\"app\"><p>Generated content</p></div>"));
        assert!(!processed.contains("data-ssg=\"content\""));
    }

    #[test]
    fn test_wrap_html_with_data_attributes() {
        // Template with data-ssg attributes
        let template = r#"<!DOCTYPE html>
<html>
<head>
    <title data-ssg="title">Default Title for Development</title>
    <meta name="description" data-ssg="description" content="Default description">
    <meta data-ssg-placeholder="meta_tags" content="seo">
    <meta data-ssg-placeholder="open_graph" content="og">
</head>
<body>
    <div id="app" data-ssg="content"></div>
</body>
</html>"#;

        // Setup generator with template
        let config = SsgConfigBuilder::new()
            .output_dir("test_dist")
            .default_template_string(template.to_string())
            .build();

        let generator = StaticSiteGenerator::new(config).unwrap();

        // Test data
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "SEO Title".to_string());
        metadata.insert("description".to_string(), "SEO Description".to_string());

        let mut generator_outputs = HashMap::new();
        generator_outputs.insert(
            "meta_tags".to_string(),
            "<meta name=\"keywords\" content=\"test,seo\">".to_string(),
        );
        generator_outputs.insert(
            "open_graph".to_string(),
            "<meta property=\"og:title\" content=\"OG Title\">".to_string(),
        );

        // Execute - this should call both renderer and post-processor
        let result = generator
            .wrap_html(
                "<p>Generated content</p>",
                "/test",
                &metadata,
                &generator_outputs,
            )
            .unwrap();

        // Verify that both templating and post-processing occurred
        assert!(result.contains("<title>SEO Title</title>"));
        assert!(!result.contains("Default Title for Development"));
        assert!(!result.contains("data-ssg=\"title\""));

        assert!(result.contains("<meta name=\"description\" content=\"SEO Description\""));
        assert!(!result.contains("Default description"));

        assert!(result.contains("<meta name=\"keywords\" content=\"test,seo\">"));
        assert!(!result.contains("data-ssg-placeholder=\"meta_tags\""));

        assert!(result.contains("<meta property=\"og:title\" content=\"OG Title\">"));
        assert!(!result.contains("data-ssg-placeholder=\"open_graph\""));

        assert!(result.contains("<div id=\"app\"><p>Generated content</p></div>"));
        assert!(!result.contains("data-ssg=\"content\""));
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
