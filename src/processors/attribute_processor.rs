use crate::generator_collection::GeneratorCollection;
use crate::processor::Processor;
use crate::processors::AttributeSupport;
use log::debug;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

/// Processor that handles HTML attribute-based templating
pub struct AttributeProcessor {
    /// The prefix used for data attributes (e.g., "data-ssg")
    prefix: String,
    /// Handlers for specific metadata attributes
    attribute_handlers:
        HashMap<String, Box<dyn Fn(&str, &str, &HashMap<String, String>) -> String + Send + Sync>>,
    /// Handlers for generator placeholder elements
    placeholder_handlers: HashMap<String, Box<dyn Fn(&str, &str) -> String + Send + Sync>>,
    /// Handler for the main content area
    content_handler: Option<Box<dyn Fn(&str, &str) -> String + Send + Sync>>,
}

impl fmt::Debug for AttributeProcessor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AttributeProcessor")
            .field("prefix", &self.prefix)
            .field("attribute_handlers_count", &self.attribute_handlers.len())
            .field(
                "placeholder_handlers_count",
                &self.placeholder_handlers.len(),
            )
            .field("has_content_handler", &self.content_handler.is_some())
            .finish()
    }
}

impl Clone for AttributeProcessor {
    fn clone(&self) -> Self {
        // We can't actually clone the handler functions, so we create a new instance with the same prefix
        AttributeProcessor::new(&self.prefix)
    }
}

impl AttributeProcessor {
    /// Create a new attribute processor with the given prefix
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            attribute_handlers: HashMap::new(),
            placeholder_handlers: HashMap::new(),
            content_handler: None,
        }
    }

    /// Register a handler for a specific attribute
    pub fn register_attribute_handler<F>(mut self, attr_name: &str, handler: F) -> Self
    where
        F: Fn(&str, &str, &HashMap<String, String>) -> String + Send + Sync + 'static,
    {
        self.attribute_handlers
            .insert(attr_name.to_string(), Box::new(handler));
        self
    }

    /// Register a handler for a placeholder attribute for a specific generator
    pub fn register_placeholder_handler<F>(mut self, generator_name: &str, handler: F) -> Self
    where
        F: Fn(&str, &str) -> String + Send + Sync + 'static,
    {
        self.placeholder_handlers
            .insert(generator_name.to_string(), Box::new(handler));
        self
    }

    /// Register a handler for the content area
    pub fn register_content_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str, &str) -> String + Send + Sync + 'static,
    {
        self.content_handler = Some(Box::new(handler));
        self
    }

    /// Add default handlers for common attributes
    pub fn with_default_handlers(self) -> Self {
        let prefix_clone = self.prefix.clone();

        // Title handler
        let title_handler = move |html: &str, value: &str, _: &HashMap<String, String>| {
            let pattern = format!(r#"<title {}="title">[^<]*</title>"#, prefix_clone);
            let re = Regex::new(&pattern).unwrap();
            re.replace_all(html, &format!("<title>{}</title>", value))
                .to_string()
        };

        let prefix_clone = self.prefix.clone();

        // Description meta tag handler
        let desc_handler = move |html: &str, value: &str, _: &HashMap<String, String>| {
            let pattern = format!(
                r#"<meta name="description" {}="description" content="[^"]*""#,
                prefix_clone
            );
            let re = Regex::new(&pattern).unwrap();
            re.replace_all(
                html,
                &format!(r#"<meta name="description" content="{}""#, value),
            )
            .to_string()
        };

        let prefix_clone = self.prefix.clone();

        // Keywords meta tag handler
        let keywords_handler = move |html: &str, value: &str, _: &HashMap<String, String>| {
            let pattern = format!(
                r#"<meta name="keywords" {}="keywords" content="[^"]*""#,
                prefix_clone
            );
            let re = Regex::new(&pattern).unwrap();
            re.replace_all(
                html,
                &format!(r#"<meta name="keywords" content="{}""#, value),
            )
            .to_string()
        };

        let prefix_clone = self.prefix.clone();

        // Default content handler
        let content_handler = move |html: &str, content: &str| {
            let pattern = format!(r#"<div id="app" {}="content">.*?</div>"#, prefix_clone);
            let re = Regex::new(&pattern).unwrap();

            if re.is_match(html) {
                return re
                    .replace_all(html, &format!(r#"<div id="app">{}</div>"#, content))
                    .to_string();
            }

            // Fallback for empty div
            let empty_pattern = format!(r#"<div id="app" {}="content"></div>"#, prefix_clone);
            let empty_re = Regex::new(&empty_pattern).unwrap();

            empty_re
                .replace_all(html, &format!(r#"<div id="app">{}</div>"#, content))
                .to_string()
        };

        self.register_attribute_handler("title", title_handler)
            .register_attribute_handler("description", desc_handler)
            .register_attribute_handler("keywords", keywords_handler)
            .register_content_handler(content_handler)
    }

    /// Configure placeholder handlers based on available generators
    pub fn configure_for_generators(mut self, generators: &GeneratorCollection) -> Self {
        let prefix = Arc::new(self.prefix.clone());

        for generator in generators.iter() {
            // Configure handlers for generator output placeholders
            let generator_name = generator.name().to_string();
            let prefix_clone = Arc::clone(&prefix);

            self =
                self.register_placeholder_handler(&generator_name.clone(), move |html, value| {
                    let pattern = format!(
                        r#"<meta {}-placeholder="{}" [^>]*>"#,
                        prefix_clone, generator_name
                    );
                    let re = Regex::new(&pattern).unwrap();
                    re.replace_all(html, value).to_string()
                });

            // If generator implements AttributeSupport, register attribute handlers
            if let Some(attr_support) = generator
                .as_any()
                .downcast_ref::<Box<dyn AttributeSupport>>()
            {
                for attr_name in attr_support.attributes() {
                    let prefix_clone = Arc::clone(&prefix);

                    self = self.register_attribute_handler(attr_name, move |html, value, _| {
                        let pattern = format!(r#"<[^>]+ {}="{}"[^>]*>"#, prefix_clone, attr_name);
                        let re = Regex::new(&pattern).unwrap();

                        // If pattern matches, replace with generator output or value
                        if re.is_match(html) {
                            // Generate replacement based on attribute
                            // This could be a more complex function that calls back to the generator
                            let replacement = format!("<{}>{}</{}>", attr_name, value, attr_name);
                            return re.replace_all(html, &replacement).to_string();
                        }
                        html.to_string()
                    });
                }
            }
        }

        self
    }
}

impl Processor for AttributeProcessor {
    fn name(&self) -> &'static str {
        "attribute_processor"
    }

    fn process(
        &self,
        html: &str,
        metadata: &HashMap<String, String>,
        generator_outputs: &HashMap<String, String>,
        content: &str,
    ) -> Result<String, Box<dyn Error>> {
        debug!("Processing attributes with metadata: {:?}", metadata);
        let mut result = html.to_string();

        // Process attribute values from metadata
        for (attr_name, handler) in &self.attribute_handlers {
            if let Some(value) = metadata.get(attr_name) {
                let attr_pattern = format!("{}=\"{}\"", self.prefix, attr_name);
                if result.contains(&attr_pattern) {
                    result = handler(&result, value, metadata);
                }
            }
        }

        // Process placeholders for generator outputs
        for (generator_name, handler) in &self.placeholder_handlers {
            if let Some(value) = generator_outputs.get(generator_name) {
                let placeholder_pattern =
                    format!("{}-placeholder=\"{}\"", self.prefix, generator_name);
                if result.contains(&placeholder_pattern) {
                    result = handler(&result, value);
                }
            }
        }

        // Process content area
        if let Some(handler) = &self.content_handler {
            let content_pattern = format!("{}=\"content\"", self.prefix);
            if result.contains(&content_pattern) {
                result = handler(&result, content);
            }
        }

        Ok(result)
    }

    fn clone_box(&self) -> Box<dyn Processor> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::Generator;
    use crate::generator_collection::GeneratorCollection;
    use crate::processors::AttributeSupport;
    use std::any::Any;
    use std::error::Error;

    // Mock Generator for testing
    #[derive(Debug, Clone)]
    struct MockGenerator {
        name: String,
    }

    impl MockGenerator {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    impl Generator for MockGenerator {
        fn name(&self) -> &'static str {
            Box::leak(self.name.clone().into_boxed_str())
        }

        fn generate(
            &self,
            _route: &str,
            _content: &str,
            _metadata: &HashMap<String, String>,
        ) -> Result<String, Box<dyn Error>> {
            Ok(format!("<p>Generated {}</p>", self.name))
        }

        fn clone_box(&self) -> Box<dyn Generator> {
            Box::new(self.clone())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    impl AttributeSupport for MockGenerator {
        fn attributes(&self) -> Vec<&'static str> {
            vec!["mock_attr"]
        }
    }

    #[test]
    fn test_attribute_processor_simple() {
        // Setup
        let processor = AttributeProcessor::new("data-test").with_default_handlers();
        let html = r#"<!DOCTYPE html><html><head><title data-test="title">Default Title</title>
        <meta name="description" data-test="description" content="Default description"></head>
        <body><div id="app" data-test="content"></div></body></html>"#;

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Replaced Title".to_string());
        metadata.insert(
            "description".to_string(),
            "Replaced description".to_string(),
        );

        // Execute
        let processed = processor
            .process(html, &metadata, &HashMap::new(), "<div>New Content</div>")
            .unwrap();

        // Assert
        assert!(processed.contains("<title>Replaced Title</title>"));
        assert!(processed.contains(r#"<meta name="description" content="Replaced description""#));
        assert!(processed.contains("<div>New Content</div>"));
        assert!(!processed.contains("data-test=")); // Ensure data attributes are removed
    }

    #[test]
    fn test_attribute_processor_generator_placeholders() {
        // Setup
        let mut generators = GeneratorCollection::new();
        generators.add(MockGenerator::new("meta"));
        generators.add(MockGenerator::new("og"));

        let processor = AttributeProcessor::new("data-test")
            .with_default_handlers()
            .configure_for_generators(&generators);

        let html = r#"<!DOCTYPE html><html><head><meta data-test-placeholder="meta" content="default">
            <meta data-test-placeholder="og" content="default"></head><body><div id="app" data-test="content"></div></body></html>"#;

        let mut generator_outputs = HashMap::new();
        generator_outputs.insert("meta".to_string(), "<meta value=\"generated\">".to_string());
        generator_outputs.insert("og".to_string(), "<meta value=\"og\">".to_string());

        // Execute
        let processed = processor
            .process(
                html,
                &HashMap::new(),
                &generator_outputs,
                "<p>App Content</p>",
            )
            .unwrap();

        // Assert
        assert!(processed.contains("<meta value=\"generated\">"));
        assert!(processed.contains("<meta value=\"og\">"));
        assert!(!processed.contains("data-test-placeholder="));
    }

    #[test]
    fn test_attribute_processor_mixed() {
        // Setup
        let processor = AttributeProcessor::new("data-custom").register_attribute_handler(
            "custom_attr",
            |html, value, _| {
                let pattern =
                    format!(r#"<div data-custom="custom_attr" data-custom-value="[^"]*""#);
                let re = Regex::new(&pattern).unwrap();
                re.replace_all(
                    html,
                    &format!(
                        r#"<div data-custom="custom_attr" data-custom-value="{}""#,
                        value
                    ),
                )
                .to_string()
            },
        );

        let html = r#"<div data-custom="custom_attr" data-custom-value="original"></div>"#;
        let mut metadata = HashMap::new();
        metadata.insert("custom_attr".to_string(), "new".to_string());

        // Execute
        let result = processor
            .process(html, &metadata, &HashMap::new(), "")
            .unwrap();

        // Assert
        assert_eq!(
            result,
            "<div data-custom=\"custom_attr\" data-custom-value=\"new\"></div>"
        );
    }

    #[test]
    fn test_attribute_processor_no_match() {
        // Setup
        let processor = AttributeProcessor::new("data-test").with_default_handlers();
        let html = r#"<!DOCTYPE html><html><head><title>Default Title</title>
        <meta name="description" content="Default description"></head>
        <body><div id="app"></div></body></html>"#;

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Replaced Title".to_string());
        metadata.insert(
            "description".to_string(),
            "Replaced description".to_string(),
        );

        // Execute
        let processed = processor
            .process(html, &metadata, &HashMap::new(), "<div>New Content</div>")
            .unwrap();

        // Assert - should return the original HTML because there's nothing to replace
        assert_eq!(processed, html);
    }
}
