use crate::config::SsgConfig;
use log::{info, warn};
use minijinja::Environment;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;
use std::fs;
use std::path::PathBuf;
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
    pub template_env: Environment<'static>,
}

impl StaticSiteGenerator {
    /// Create a new static site generator from the provided configuration.
    pub fn new(config: SsgConfig) -> Result<Self, Box<dyn Error>> {
        let mut env = Environment::new();
        let mut template_loaded = false;

        // 1. Try loading from template_path
        if let Some(path) = &config.template_path {
            if path.exists() {
                match fs::read_to_string(path) {
                    Ok(content) => {
                        let static_template = Box::leak(content.into_boxed_str());
                        env.add_template("base", static_template)?;
                        template_loaded = true;
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
        }

        // 3. Fallback to built-in default template
        if !template_loaded {
            env.add_template("base", DEFAULT_TEMPLATE)?;
            info!("Using built-in default HTML template");
        }

        // We no longer need to manually add processors here since they're added in the config by default

        Ok(Self {
            config,
            template_env: env,
        })
    }

    /// Generate static HTML files for all routes.
    pub async fn generate<R, C>(&self) -> Result<(), Box<dyn Error>>
    where
        R: Routable + IntoEnumIterator + Clone + PartialEq + Debug + Send + 'static,
        C: BaseComponent<Properties = ()> + 'static,
    {
        fs::create_dir_all(&self.config.output_dir)?;

        // Get path prefix from environment variable if set
        let path_prefix = std::env::var("YEW_SSG_CURRENT_PATH_PREFIX").unwrap_or_default();
        if !path_prefix.is_empty() {
            info!("Using path prefix: {}", path_prefix);
        }

        for route in R::iter() {
            let route_path = route.to_path();
            info!("Generating route: {}", route_path);

            // 1. Set the YEW_SSG_CURRENT_PATH env var, with prefix if present
            if path_prefix.is_empty() {
                std::env::set_var("YEW_SSG_CURRENT_PATH", &route_path);
            } else {
                // If prefix doesn't have a leading slash, add one
                let prefixed_path = if path_prefix.starts_with('/') {
                    format!("{}{}", path_prefix, route_path)
                } else {
                    format!("/{}{}", path_prefix, route_path)
                };
                std::env::set_var("YEW_SSG_CURRENT_PATH", &prefixed_path);
                info!("  Using prefixed path: {}", prefixed_path);
            }

            // 2. Render the root component
            let content = self.render_base_component::<C>().await?;

            // 3. Clear the YEW_SSG_CURRENT_PATH env var
            std::env::remove_var("YEW_SSG_CURRENT_PATH");

            // 4. Get metadata for the route
            let mut metadata = self.config.get_metadata_for_route(&route_path);
            metadata.insert("path".to_string(), route_path.clone());

            // Add prefix to metadata if present
            if !path_prefix.is_empty() {
                metadata.insert("path_prefix".to_string(), path_prefix.clone());
            }

            // 5. Generate outputs from all generators
            let mut generator_outputs = HashMap::new();

            for generator in &self.config.generators.generators {
                // Generate the main output using the generator's name
                let name = generator.name();
                let result = generator.generate(name, &route_path, &content, &metadata)?;
                generator_outputs.insert(name.to_string(), result);

                // Check if generator supports additional outputs
                if let Some(support) = self.config.generators.try_get_output_support(generator) {
                    // Generate additional outputs
                    for key in support.supported_outputs() {
                        // Skip the main output we already did
                        if key == name {
                            continue;
                        }

                        match generator.generate(key, &route_path, &content, &metadata) {
                            Ok(output) => {
                                generator_outputs.insert(key.to_string(), output);
                            }
                            Err(e) => {
                                warn!("Failed to generate '{}' output: {}", key, e);
                            }
                        }
                    }
                }
            }

            // 6. Run processors on the content
            let processed_content = self.config.processors.process_all(
                &content,
                &metadata,
                &generator_outputs,
                &content,
            )?;

            // 7. Create the final HTML
            let html = self.wrap_html(
                &processed_content,
                &route_path,
                &metadata,
                &generator_outputs,
            )?;

            // 8. Write to output file - use prefixed output path if prefix is set
            let (dir_path, file_path) = if path_prefix.is_empty() {
                self.determine_output_path(&route_path)
            } else {
                // If prefix is set, add it to the output path
                // Ensure prefix doesn't have leading slash for file paths
                let clean_prefix = path_prefix.trim_start_matches('/');

                if route_path == "/" {
                    // Special case for root route
                    let prefixed_dir = self.config.output_dir.join(clean_prefix);
                    (prefixed_dir.clone(), prefixed_dir.join("index.html"))
                } else {
                    // For other routes
                    let path_component = route_path.trim_start_matches('/');
                    let prefixed_path = format!("{}/{}", clean_prefix, path_component);
                    let dir = self.config.output_dir.join(prefixed_path);
                    (dir.clone(), dir.join("index.html"))
                }
            };

            fs::create_dir_all(&dir_path)?;
            fs::write(&file_path, html)?;
            info!("  -> Saved to {:?}", file_path);
        }

        Ok(())
    }

    /// Generate parameterized routes by inferring the route constructor from patterns
    pub async fn generate_parameterized_routes<R, C>(&self) -> Result<(), Box<dyn Error>>
    where
        R: Routable + Clone + PartialEq + Debug + Send + 'static,
        C: BaseComponent<Properties = ()> + 'static,
    {
        info!("Generating parameterized routes from configuration...");

        // Early return if no parameterized routes are defined
        if self.config.route_params.is_empty() {
            info!("No parameterized routes defined in configuration");
            return Ok(());
        }

        // Get path prefix from environment variable if set
        let path_prefix = std::env::var("YEW_SSG_CURRENT_PATH_PREFIX").unwrap_or_default();
        if !path_prefix.is_empty() {
            info!(
                "Using path prefix for parameterized routes: {}",
                path_prefix
            );
        }

        let mut total_generated = 0;

        // For each parameterized route pattern in the config
        for (route_pattern_config, route_params) in &self.config.route_params {
            // Generate all parameter combinations based on the config
            let param_combinations = route_params.generate_param_combinations();
            if param_combinations.is_empty() {
                warn!(
                    "No valid parameter combinations for route pattern: {}",
                    route_pattern_config
                );
                continue;
            }

            info!(
                "Processing {} variants for parameterized route: {}",
                param_combinations.len(),
                route_pattern_config
            );

            // For each parameter combination
            for params in param_combinations {
                // Convert the pattern with placeholders into an actual path with values
                let constructed_path =
                    Self::construct_path_from_pattern(route_pattern_config, &params);

                // Now use the actual Routable trait to recognize the path
                if let Some(route) = R::recognize(&constructed_path) {
                    let route_path = route.to_path();

                    info!(
                        "Generating parameterized route: {} with params: {:?}",
                        route_path, params
                    );
                    total_generated += 1;

                    // 1. Set the YEW_SSG_CURRENT_PATH env var, with prefix if present
                    if path_prefix.is_empty() {
                        std::env::set_var("YEW_SSG_CURRENT_PATH", &route_path);
                    } else {
                        // If prefix doesn't have a leading slash, add one
                        let prefixed_path = if path_prefix.starts_with('/') {
                            format!("{}{}", path_prefix, route_path)
                        } else {
                            format!("/{}{}", path_prefix, route_path)
                        };
                        std::env::set_var("YEW_SSG_CURRENT_PATH", &prefixed_path);
                        info!("  Using prefixed path: {}", prefixed_path);
                    }

                    // Set any route params as environment variables for access during rendering
                    for (key, value) in &params {
                        std::env::set_var(format!("YEW_SSG_PARAM_{}", key), value);
                    }

                    // 2. Render the root component
                    let content = self.render_base_component::<C>().await?;

                    // 3. Clear environment variables
                    std::env::remove_var("YEW_SSG_CURRENT_PATH");
                    for key in params.keys() {
                        std::env::remove_var(format!("YEW_SSG_PARAM_{}", key));
                    }

                    // 4. Get metadata for the route, including parameter-specific metadata
                    let mut metadata = self
                        .config
                        .get_metadata_for_parameterized_route(route_pattern_config, &params);

                    metadata.insert("path".to_string(), route_path.to_string());

                    // Add prefix to metadata if present
                    if !path_prefix.is_empty() {
                        metadata.insert("path_prefix".to_string(), path_prefix.clone());
                    }

                    // 5. Generate outputs from all generators
                    let mut generator_outputs = HashMap::new();

                    for generator in &self.config.generators.generators {
                        // Generate the main output using the generator's name
                        let name = generator.name();
                        let result = generator.generate(name, &route_path, &content, &metadata)?;
                        generator_outputs.insert(name.to_string(), result);

                        // Check if generator supports additional outputs
                        if let Some(support) =
                            self.config.generators.try_get_output_support(generator)
                        {
                            // Generate additional outputs
                            for key in support.supported_outputs() {
                                // Skip the main output we already did
                                if key == name {
                                    continue;
                                }

                                match generator.generate(key, &route_path, &content, &metadata) {
                                    Ok(output) => {
                                        generator_outputs.insert(key.to_string(), output);
                                    }
                                    Err(e) => {
                                        warn!("Failed to generate '{}' output: {}", key, e);
                                    }
                                }
                            }
                        }
                    }

                    // 6. Run processors on the content
                    let processed_content = self.config.processors.process_all(
                        &content,
                        &metadata,
                        &generator_outputs,
                        &content,
                    )?;

                    // 7. Create the final HTML
                    let html = self.wrap_html(
                        &processed_content,
                        &route_path,
                        &metadata,
                        &generator_outputs,
                    )?;

                    // 8. Write to output file - use prefixed output path if prefix is set
                    let (dir_path, file_path) = if path_prefix.is_empty() {
                        self.determine_output_path(&route_path)
                    } else {
                        // If prefix is set, add it to the output path
                        // Ensure prefix doesn't have leading slash for file paths
                        let clean_prefix = path_prefix.trim_start_matches('/');

                        if route_path == "/" {
                            // Special case for root route
                            let prefixed_dir = self.config.output_dir.join(clean_prefix);
                            (prefixed_dir.clone(), prefixed_dir.join("index.html"))
                        } else {
                            // For other routes
                            let path_component = route_path.trim_start_matches('/');
                            let prefixed_path = format!("{}/{}", clean_prefix, path_component);
                            let dir = self.config.output_dir.join(prefixed_path);
                            (dir.clone(), dir.join("index.html"))
                        }
                    };

                    fs::create_dir_all(&dir_path)?;
                    fs::write(&file_path, html)?;
                    info!("  -> Saved to {:?}", file_path);
                } else {
                    warn!(
                        "No route recognized for constructed path: {} (from pattern {})",
                        constructed_path, route_pattern_config
                    );
                }
            }
        }

        info!(
            "Generated {} parameterized route pages in total",
            total_generated
        );

        Ok(())
    }

    // Helper function to construct a path by replacing placeholders with actual values
    fn construct_path_from_pattern(pattern: &str, params: &HashMap<String, String>) -> String {
        let mut result = pattern.to_string();

        for (key, value) in params {
            let placeholder = format!(":{}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }

    pub async fn generate_with_params<R, C, F>(
        &self,
        route_pattern: &str,
        route_builder: F,
    ) -> Result<(), Box<dyn Error>>
    where
        R: Routable + Clone + PartialEq + Debug + Send + 'static,
        C: BaseComponent<Properties = ()> + 'static,
        F: Fn(&HashMap<String, String>) -> R,
    {
        // Check if we have parameters defined for this route
        let route_params = match self.config.route_params.get(route_pattern) {
            Some(params) => params,
            None => {
                warn!("No parameters defined for route pattern: {}", route_pattern);
                return Ok(());
            }
        };

        // Generate all parameter combinations based on the config
        let param_combinations = route_params.generate_param_combinations();
        if param_combinations.is_empty() {
            warn!(
                "No valid parameter combinations for route pattern: {}",
                route_pattern
            );
            return Ok(());
        }

        info!(
            "Generating {} variants for parameterized route: {}",
            param_combinations.len(),
            route_pattern
        );

        // Generate a page for each parameter combination
        for params in param_combinations {
            // Build the route using the provided function
            let route = route_builder(&params);
            let route_path = route.to_path();

            info!(
                "Generating parameterized route: {} with params: {:?}",
                route_path, params
            );

            // 1. Set the YEW_SSG_CURRENT_PATH env var
            std::env::set_var("YEW_SSG_CURRENT_PATH", &route_path);

            // Set any route params as environment variables for access during rendering
            for (key, value) in &params {
                std::env::set_var(format!("YEW_SSG_PARAM_{}", key), value);
            }

            // 2. Render the root component
            let content = self.render_base_component::<C>().await?;

            // 3. Clear environment variables
            std::env::remove_var("YEW_SSG_CURRENT_PATH");
            for key in params.keys() {
                std::env::remove_var(format!("YEW_SSG_PARAM_{}", key));
            }

            // 4. Get metadata for the route, including parameter-specific metadata
            let mut metadata = self
                .config
                .get_metadata_for_parameterized_route(route_pattern, &params);
            metadata.insert("path".to_string(), route_path.to_string());

            // 5. Generate outputs from all generators
            let mut generator_outputs = HashMap::new();

            for generator in &self.config.generators.generators {
                // Generate the main output using the generator's name
                let name = generator.name();
                let result = generator.generate(name, &route_path, &content, &metadata)?;
                generator_outputs.insert(name.to_string(), result);

                // Check if generator supports additional outputs
                if let Some(support) = self.config.generators.try_get_output_support(generator) {
                    // Generate additional outputs
                    for key in support.supported_outputs() {
                        // Skip the main output we already did
                        if key == name {
                            continue;
                        }

                        match generator.generate(key, &route_path, &content, &metadata) {
                            Ok(output) => {
                                generator_outputs.insert(key.to_string(), output);
                            }
                            Err(e) => {
                                warn!("Failed to generate '{}' output: {}", key, e);
                            }
                        }
                    }
                }
            }

            // 6. Run processors on the content
            let processed_content = self.config.processors.process_all(
                &content,
                &metadata,
                &generator_outputs,
                &content,
            )?;

            // 7. Create the final HTML
            let html = self.wrap_html(
                &processed_content,
                &route_path,
                &metadata,
                &generator_outputs,
            )?;

            // 8. Write to output file
            let (dir_path, file_path) = self.determine_output_path(&route_path);
            fs::create_dir_all(&dir_path)?;
            fs::write(&file_path, html)?;
            info!("  -> Saved to {:?}", file_path);
        }

        Ok(())
    }

    /// Generate all pages with dynamic parameters defined in the configuration.
    pub async fn generate_all_parameterized_routes<R, C, F>(
        &self,
        route_builders: &HashMap<&str, F>,
    ) -> Result<(), Box<dyn Error>>
    where
        R: Routable + Clone + PartialEq + Debug + Send + 'static,
        C: BaseComponent<Properties = ()> + 'static,
        F: Fn(&HashMap<String, String>) -> R + Clone, // Add Clone bound
    {
        for (route_pattern, route_builder) in route_builders {
            // Clone the route builder to avoid reference issues
            let builder_clone = route_builder.clone();
            self.generate_with_params::<R, C, _>(route_pattern, builder_clone)
                .await?;
        }
        Ok(())
    }

    /// Helper function to get current parameters during SSG
    pub fn get_current_params() -> HashMap<String, String> {
        let mut params = HashMap::new();
        for (key, value) in std::env::vars() {
            if let Some(param_name) = key.strip_prefix("YEW_SSG_PARAM_") {
                params.insert(param_name.to_string(), value);
            }
        }
        params
    }

    /// Render the root component to HTML using server-side rendering.
    async fn render_base_component<C>(&self) -> Result<String, Box<dyn Error>>
    where
        C: BaseComponent<Properties = ()> + 'static,
    {
        let renderer = ServerRenderer::<C>::new();
        Ok(renderer.render().await)
    }

    /// Determine output directory and file path for a route.
    pub fn determine_output_path(&self, route_path: &str) -> (PathBuf, PathBuf) {
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

    /// Create the final HTML by combining rendered content, metadata, and generator outputs.
    pub fn wrap_html(
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
        let rendered_template = tmpl.render(context)?;

        // Apply processors to the rendered template
        let processed_html = self.config.processors.process_all(
            &rendered_template,
            &metadata,
            &generator_outputs,
            content,
        )?;

        Ok(processed_html)
    }
}

// Add to the bottom of yew-ssg/src/static_site_generator.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SsgConfigBuilder;
    use std::collections::HashMap;

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

    #[test]
    fn test_data_ssg_attribute_replacement_in_pipeline() {
        use crate::processor::Processor;
        use crate::processors::AttributeProcessor;

        // Our test "content" is HTML with data-ssg attributes
        let content_with_attributes = r#"<div data-ssg="content"></div>"#;

        // Set up metadata
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Replaced Title".to_string());
        metadata.insert(
            "description".to_string(),
            "Replaced description".to_string(),
        );

        metadata.insert("content".to_string(), "<p>Page content</p>".to_string());

        // Create a processor and process the content directly
        let attribute_processor = AttributeProcessor::new("data-ssg", None).with_default_handlers();
        let processed_content = attribute_processor
            .process(
                content_with_attributes,
                &metadata,
                &metadata,
                "<p>Page content</p>", // The actual content to insert
            )
            .unwrap();

        println!("Processed content: {}", processed_content);

        // Now check the processed content has the attributes replaced
        assert!(processed_content.contains("<div><p>Page content</p></div>"));
        assert!(!processed_content.contains("data-ssg=")); // Ensure data attributes are removed

        // Now let's test the full pipeline with a MiniJinja template
        let template = r#"<html lang="en">
        <head>
            <title>{{ title }}</title>
            <meta name="description" content="{{ description }}">
        </head>
        <body>
            {{ content | safe }}
        </body>
    </html>"#;

        let config = SsgConfigBuilder::new()
            .output_dir("test_dist")
            .default_template_string(template.to_string())
            .build();

        let ssg = StaticSiteGenerator::new(config).unwrap();

        // The content we pass to wrap_html is already processed by our processor
        let result = ssg
            .wrap_html(&processed_content, "/test-page", &metadata, &metadata)
            .unwrap();

        println!("Wrapped HTML: {}", result);

        // Verify everything is in place
        assert!(result.contains("<title>Replaced Title</title>"));
        assert!(result.contains(r#"<meta name="description" content="Replaced description">"#));
        assert!(result.contains("<p>Page content</p>"));
    }

    #[test]
    fn test_data_ssg_attribute_replacement() {
        use crate::processor::Processor;
        use crate::processors::AttributeProcessor;

        // Create template with data-ssg attribute
        let template = r#"<html lang="en">
        <head>
            <title data-ssg="title">Default Title</title>
            <meta name="description" data-ssg-content="description" content="Default description">
        </head>
        <body>
            <div data-ssg="content"></div>
        </body>
    </html>"#;

        // Set up metadata
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Replaced Title".to_string());
        metadata.insert(
            "description".to_string(),
            "Replaced description".to_string(),
        );

        // Process the template with our AttributeProcessor
        let attribute_processor = AttributeProcessor::new("data-ssg", None).with_default_handlers();
        let processed_template = attribute_processor
            .process(template, &metadata, &metadata, "<p>Page content</p>")
            .unwrap();

        // Verify the processor worked
        assert!(processed_template.contains("<title>Replaced Title</title>"));
        assert!(processed_template.contains("content=\"Replaced description\""));
        assert!(!processed_template.contains("data-ssg="));

        // Now set up the SSG with the processed template
        let config = SsgConfigBuilder::new()
            .output_dir("test_dist")
            .default_template_string(processed_template)
            .build();

        let ssg = StaticSiteGenerator::new(config).unwrap();

        // Process the template through the SSG
        let result = ssg
            .wrap_html(
                "<p>More content</p>",
                "/test-page",
                &metadata,
                &HashMap::new(),
            )
            .unwrap();

        // Debug output
        println!("Final HTML: {}", result);

        // Verify the final HTML contains our replacements
        assert!(result.contains("<title>Replaced Title</title>"));
        assert!(result.contains("content=\"Replaced description\""));
        assert!(!result.contains("data-ssg="));
    }

    #[test]
    fn test_placeholder_processor_integration() {
        use crate::generators::TitleGenerator;

        // Create a simple HTML with a placeholder
        let html = r#"<html>
        <head>
            <div data-ssg-placeholder="title"></div>
        </head>
        <body>
            <div id="content">Test content</div>
        </body>
        </html>"#;

        // Set up a generator collection with a title generator
        let mut generators = crate::generator_collection::GeneratorCollection::new();
        generators.add(TitleGenerator);

        // Create metadata with a title
        let mut metadata = HashMap::new();
        metadata.insert(
            "title".to_string(),
            "<title>My Test Page</title>".to_string(),
        );

        // Now let's create a complete SSG and verify it works end-to-end
        let config = SsgConfigBuilder::new()
            .output_dir("test_dist")
            .default_template_string(html.to_string())
            .build();

        let mut config = config;

        // Add the title generator
        config.generators.add(TitleGenerator);

        // Create the SSG (which will add default processors)
        let ssg = StaticSiteGenerator::new(config).unwrap();

        // Process through the SSG's wrap_html
        let result = ssg
            .wrap_html("", "/test-page", &metadata, &metadata)
            .unwrap();

        // Verify placeholder was replaced
        assert!(result.contains("<title>My Test Page</title>"));
        assert!(!result.contains("data-ssg-placeholder"));
    }

    #[test]
    fn test_canonical_url_handling() {
        use crate::generators::MetaTagGenerator;

        // Create a test template with a placeholder for canonical URL
        let template = r#"<!DOCTYPE html><html><head>
        {{ canonical | default("") | safe }}
        </head><body>{{ content | safe }}</body></html>"#;

        // Set up configuration
        let mut config = SsgConfigBuilder::new()
            .output_dir("test_dist")
            .default_template_string(template.to_string())
            .build();

        // Add meta tag generator
        config.generators.add(MetaTagGenerator {
            default_description: "Default description".to_string(),
            default_keywords: vec!["rust".to_string(), "yew".to_string(), "ssg".to_string()],
        });

        let generator = StaticSiteGenerator::new(config).unwrap();

        // Set up metadata with canonical URL
        let mut metadata = HashMap::new();
        metadata.insert(
            "canonical".to_string(),
            "https://example.com/test".to_string(),
        );

        // Generate and get outputs from generators
        let mut generator_outputs = HashMap::new();
        for generator_box in &generator.config.generators.generators {
            let key = generator_box.name();
            match generator_box.generate(key, "/test", "", &metadata) {
                Ok(output) => {
                    generator_outputs.insert(key.to_string(), output);
                }
                Err(_) => {}
            }

            // Also get the canonical URL specifically
            if let Ok(canonical) = generator_box.generate("canonical", "/test", "", &metadata) {
                generator_outputs.insert("canonical".to_string(), canonical);
            }
        }

        // Process the template
        let result = generator
            .wrap_html("Test content", "/test", &metadata, &generator_outputs)
            .unwrap();

        // Verify the canonical URL is properly formatted
        assert!(result.contains(r#"<link rel="canonical" href="https://example.com/test">"#));

        // Verify there is no nested link tag
        assert!(!result.contains(r#"<link rel="canonical" href="<link"#));
    }

    #[test]
    fn test_canonical_url_in_full_template() {
        use crate::generators::MetaTagGenerator;

        // Create a more realistic template
        let template = r#"<!DOCTYPE html><html><head>
        <title>{{ title | default("Page Title") }}</title>
        {{ meta_tags | default("") | safe }}
        </head><body>{{ content | safe }}</body></html>"#;

        // Set up configuration
        let mut config = SsgConfigBuilder::new()
            .output_dir("test_dist")
            .default_template_string(template.to_string())
            .build();

        // Add meta tag generator
        config.generators.add(MetaTagGenerator {
            default_description: "Default description".to_string(),
            default_keywords: vec!["rust".to_string(), "yew".to_string(), "ssg".to_string()],
        });

        let generator = StaticSiteGenerator::new(config).unwrap();

        // Set up metadata with canonical URL
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Test Page".to_string());
        metadata.insert(
            "canonical".to_string(),
            "https://example.com/page".to_string(),
        );

        // Generate outputs from generators
        let mut generator_outputs = HashMap::new();
        for generator_box in &generator.config.generators.generators {
            let key = generator_box.name();
            if let Ok(output) = generator_box.generate(key, "/test", "", &metadata) {
                generator_outputs.insert(key.to_string(), output);
            }
        }

        // Process the template
        let result = generator
            .wrap_html(
                "<p>Test content</p>",
                "/test",
                &metadata,
                &generator_outputs,
            )
            .unwrap();

        // Print for debugging
        println!("Generated HTML: {}", result);

        // Verify the canonical URL is present and properly formatted
        assert!(result.contains(r#"<link rel="canonical" href="https://example.com/page">"#));

        // Verify there is no nested link tag
        assert!(!result.contains(r#"<link rel="canonical" href="<link"#));
    }

    #[test]
    fn test_canonical_url_attribute_replacement() {
        use crate::generators::MetaTagGenerator;

        // Create a template that already has the canonical link tag with a fixed value
        let template = r#"<!DOCTYPE html><html><head>
        <title>{{ title | default("Page Title") }}</title>
        <link rel="canonical" href="example.com">
        </head><body></body></html>"#;

        // Set up configuration
        let mut config = SsgConfigBuilder::new()
            .output_dir("test_dist")
            .default_template_string(template.to_string())
            .build();

        // Add meta tag generator
        config.generators.add(MetaTagGenerator {
            default_description: "Default description".to_string(),
            default_keywords: vec!["rust".to_string(), "yew".to_string(), "ssg".to_string()],
        });

        let generator = StaticSiteGenerator::new(config).unwrap();

        // Set up metadata with canonical URL
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Test Page".to_string());
        metadata.insert(
            "canonical".to_string(),
            "https://example.com/page".to_string(),
        );

        // Generate outputs from generators
        let mut generator_outputs = HashMap::new();
        for generator_box in &generator.config.generators.generators {
            let key = generator_box.name();
            if let Ok(output) = generator_box.generate(key, "/test", "", &metadata) {
                generator_outputs.insert(key.to_string(), output);
            }
        }

        // Process the template
        let result = generator
            .wrap_html(
                "<p>Test content</p>",
                "/test",
                &metadata,
                &generator_outputs,
            )
            .unwrap();

        // Print for debugging
        println!("Generated HTML with existing canonical tag: {}", result);

        // Verify we have exactly one canonical link tag
        let canonical_count = result.matches(r#"<link rel="canonical""#).count();
        assert_eq!(
            canonical_count, 1,
            "Should have exactly one canonical link tag"
        );

        // Verify the URL in the canonical tag is unchanged (still "example.com")
        // This is because the template has a static value that isn't replaced by default
        assert!(result.contains(r#"<link rel="canonical" href="example.com""#));

        // Verify there's no nested tag
        assert!(!result.contains(r#"href="<link"#));
    }

    #[test]
    fn test_canonical_url_with_parameterized_routes() {
        use crate::config::RouteParams;
        use crate::generators::CanonicalLinkGenerator;

        // Create a test template with a placeholder for canonical links
        let template = r#"<!DOCTYPE html><html><head>
        {{ canonical_links | default("") | safe }}
        </head><body>{{ content | safe }}</body></html>"#;

        // Set up configuration with route parameters
        let mut config = SsgConfigBuilder::new()
            .output_dir("test_dist")
            .default_template_string(template.to_string())
            .build();

        // Add canonical link generator with domain
        config
            .generators
            .add(CanonicalLinkGenerator::with_domain("https://example.com"));

        // Set up route parameters for a crate route
        let mut route_params = RouteParams::new();
        route_params.add_param("id", ["test-crate"]);

        // Add metadata for specific parameter value
        let mut param_metadata = HashMap::new();
        param_metadata.insert("title".to_string(), "Test Crate Details".to_string());
        param_metadata.insert(
            "description".to_string(),
            "Description of test crate".to_string(),
        );

        route_params.add_param_metadata("id", "test-crate", param_metadata);

        // Add the route parameters to the config
        config
            .route_params
            .insert("/crate/:id".to_string(), route_params);

        let generator = StaticSiteGenerator::new(config).unwrap();

        // Simulate generating a parameterized route
        let params = HashMap::from([("id".to_string(), "test-crate".to_string())]);
        let route_pattern = "/crate/:id";
        let route_path = "/crate/test-crate";

        // Get metadata for the route with parameters
        let mut metadata = generator
            .config
            .get_metadata_for_parameterized_route(route_pattern, &params);

        // IMPORTANT: Add path to metadata as would happen in generate_parameterized_routes
        metadata.insert("path".to_string(), route_path.to_string());

        // Generate outputs from generators
        let mut generator_outputs = HashMap::new();
        for generator_box in &generator.config.generators.generators {
            let key = generator_box.name();
            if let Ok(output) = generator_box.generate(key, route_path, "", &metadata) {
                generator_outputs.insert(key.to_string(), output);
            }
        }

        // Process the template
        let result = generator
            .wrap_html(
                "<p>Test content</p>",
                route_path,
                &metadata,
                &generator_outputs,
            )
            .unwrap();

        // Print for debugging
        println!("Generated HTML for parameterized route: {}", result);

        // Verify the canonical URL is present and correctly formed
        assert!(
            result
                .contains(r#"<link rel="canonical" href="https://example.com/crate/test-crate">"#),
            "Canonical URL should be correctly generated for parameterized route"
        );
    }
}
