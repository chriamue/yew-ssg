use crate::generator::Generator;
use crate::generator_collection::GeneratorCollection;
use crate::processor::Processor;
use crate::processors::AttributeSupport;
use log::{debug, info, trace, warn};
use scraper::{ElementRef, Html, Node, Selector};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

/// Processor that handles HTML attribute-based templating using proper HTML parsing
/// This version automatically adapts to generators that implement AttributeSupport
#[derive(Clone)]
pub struct AttributeProcessor {
    /// The prefix used for data attributes (e.g., "data-ssg")
    prefix: String,

    /// Content handler
    content_handler: Option<Arc<dyn Fn(&str) -> String + Send + Sync>>,

    /// Custom attribute handlers
    custom_handlers:
        HashMap<String, Arc<dyn Fn(&str, &HashMap<String, String>) -> String + Send + Sync>>,

    /// Placeholder handlers
    placeholder_handlers: HashMap<String, Arc<dyn Fn(&str) -> String + Send + Sync>>,

    /// Generator information - populated from configure_for_generators
    generator_info: HashMap<String, GeneratorInfo>,
}

#[derive(Clone)]
struct GeneratorInfo {
    name: String,
    supported_attributes: Vec<String>,
}

impl fmt::Debug for AttributeProcessor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AttributeProcessor")
            .field("prefix", &self.prefix)
            .field(
                "custom_handlers",
                &self.custom_handlers.keys().collect::<Vec<_>>(),
            )
            .field("has_content_handler", &self.content_handler.is_some())
            .field(
                "generators",
                &self.generator_info.keys().collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl AttributeProcessor {
    /// Create a new attribute processor with the given prefix
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            content_handler: None,
            custom_handlers: HashMap::new(),
            placeholder_handlers: HashMap::new(),
            generator_info: HashMap::new(),
        }
    }

    /// Register a custom handler for a specific attribute
    pub fn register_attribute_handler<F>(mut self, attr_name: &str, handler: F) -> Self
    where
        F: Fn(&str, &HashMap<String, String>) -> String + Send + Sync + 'static,
    {
        self.custom_handlers
            .insert(attr_name.to_string(), Arc::new(handler));
        self
    }

    /// Register a handler for the content area
    pub fn register_content_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.content_handler = Some(Arc::new(handler));
        self
    }

    /// Register a handler for a placeholder attribute for a specific generator
    pub fn register_placeholder_handler<F>(mut self, generator_name: &str, handler: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        self.placeholder_handlers
            .insert(generator_name.to_string(), Arc::new(handler));
        self
    }

    /// Configure the processor based on available generators
    pub fn configure_for_generators(mut self, generators: &GeneratorCollection) -> Self {
        info!("Configuring attribute processor for generators");

        // Process each generator to extract supported attributes
        for generator in generators.iter() {
            let name = generator.name().to_string();
            let mut supported_attributes = Vec::new();

            // Try to get attribute support information from the generator
            if let Some(typed_generator) = self.try_get_attribute_support(generator) {
                for attr in typed_generator.attributes() {
                    supported_attributes.push(attr.to_string());
                }

                debug!(
                    "Generator '{}' supports attributes: {:?}",
                    name, supported_attributes
                );
            } else {
                // If we couldn't detect attribute support, use the generator name
                debug!(
                    "No attribute support detected for '{}', using name as attribute",
                    name
                );
                supported_attributes.push(name.clone());
            }

            // Store the generator info
            self.generator_info.insert(
                name.clone(),
                GeneratorInfo {
                    name,
                    supported_attributes,
                },
            );
        }

        info!("Configured for {} generators", self.generator_info.len());
        self
    }

    /// Try to extract AttributeSupport from a generator
    fn try_get_attribute_support<'a>(
        &self,
        generator: &'a Box<dyn Generator>,
    ) -> Option<&'a dyn AttributeSupport> {
        // First, try direct casting (unlikely to work with trait objects)
        if let Some(support) = generator
            .as_any()
            .downcast_ref::<Box<dyn AttributeSupport>>()
        {
            return Some(support.as_ref());
        }

        // Try each possible concrete type
        use crate::generators::{
            MetaTagGenerator, OpenGraphGenerator, RobotsMetaGenerator, TitleGenerator,
            TwitterCardGenerator,
        };

        if let Some(g) = generator.as_any().downcast_ref::<MetaTagGenerator>() {
            return Some(g);
        } else if let Some(g) = generator.as_any().downcast_ref::<OpenGraphGenerator>() {
            return Some(g);
        } else if let Some(g) = generator.as_any().downcast_ref::<RobotsMetaGenerator>() {
            return Some(g);
        } else if let Some(g) = generator.as_any().downcast_ref::<TitleGenerator>() {
            return Some(g);
        } else if let Some(g) = generator.as_any().downcast_ref::<TwitterCardGenerator>() {
            return Some(g);
        }

        None
    }

    /// Add default handlers for common attributes
    pub fn with_default_handlers(self) -> Self {
        // Register the default content handler
        let processor =
            self.register_content_handler(|content| format!("<div id=\"app\">{}</div>", content));

        // Add handlers for common attributes
        let mut result = processor;

        // For title
        result = result.register_attribute_handler("title", |value, _metadata| {
            format!("<title>{}</title>", value)
        });

        // For description
        result = result.register_attribute_handler("description", |value, _metadata| {
            format!("<meta name=\"description\" content=\"{}\">", value)
        });

        // For keywords
        result = result.register_attribute_handler("keywords", |value, _metadata| {
            format!("<meta name=\"keywords\" content=\"{}\">", value)
        });

        result
    }

    /// Find elements by their data-* attributes
    fn find_elements_by_attr<'a>(
        &self,
        document: &'a Html,
        attr_name: &str,
    ) -> Vec<ElementRef<'a>> {
        let selector_string = format!("[{}=\"{}\"]", self.prefix, attr_name);
        let mut results = Vec::new();

        if let Ok(selector) = Selector::parse(&selector_string) {
            for element in document.select(&selector) {
                results.push(element);
            }
        } else {
            warn!("Invalid selector: {}", selector_string);
        }

        trace!(
            "Found {} elements for selector '{}'",
            results.len(),
            selector_string
        );
        for (i, element) in results.iter().enumerate() {
            trace!("Element {}: {}", i, element.html());
        }

        results
    }

    /// Find elements by their data-*-placeholder attributes
    fn find_placeholder_elements<'a>(
        &self,
        document: &'a Html,
        generator_name: &str,
    ) -> Vec<ElementRef<'a>> {
        let selector_string = format!("[{}-placeholder=\"{}\"]", self.prefix, generator_name);
        let mut results = Vec::new();

        if let Ok(selector) = Selector::parse(&selector_string) {
            for element in document.select(&selector) {
                results.push(element);
            }
        } else {
            warn!("Invalid selector: {}", selector_string);
        }

        results
    }

    /// Process HTML using the configured handlers and generators
    fn process_html(
        &self,
        html_str: &str,
        metadata: &HashMap<String, String>,
        generator_outputs: &HashMap<String, String>,
        content: &str,
    ) -> Result<String, Box<dyn Error>> {
        // Parse HTML
        let document = Html::parse_document(html_str);

        // Track all elements that need to be processed
        let mut replacements = Vec::new();

        // 1. Process custom attribute handlers for elements with data-ssg="attr_name"
        for (attr_name, handler) in &self.custom_handlers {
            // Create string first, then parse
            let selector_string = format!("[{}=\"{}\"]", self.prefix, attr_name);
            let selector = match Selector::parse(&selector_string) {
                Ok(s) => s,
                Err(e) => {
                    warn!("Invalid selector '{}': {:?}", selector_string, e);
                    continue;
                }
            };

            for element in document.select(&selector) {
                if let Some(value) = metadata.get(attr_name) {
                    debug!("Processing element with {}=\"{}\"", self.prefix, attr_name);
                    let html = element.html();
                    let replacement = handler(value, metadata);
                    replacements.push((html, replacement));
                }
            }
        }

        // 2. Process attribute replacements for elements with data-ssg-a="attr_name"
        {
            // Create string first, then parse
            let attr_selector_string = format!("[{}-a]", self.prefix);
            let selector = match Selector::parse(&attr_selector_string) {
                Ok(s) => s,
                Err(e) => {
                    warn!("Invalid selector '{}': {:?}", attr_selector_string, e);
                    return Ok(html_str.to_string()); // Return original if can't parse
                }
            };

            for element in document.select(&selector) {
                // Check if it has both data-ssg and data-ssg-a attributes
                if let Some(attr_to_change) = element.value().attr(&format!("{}-a", self.prefix)) {
                    if let Some(metadata_key) = element.value().attr(&self.prefix) {
                        if let Some(new_value) = metadata.get(metadata_key) {
                            debug!(
                                "Changing attribute '{}' to '{}' on element {}",
                                attr_to_change,
                                new_value,
                                element.value().name()
                            );

                            // Create a new element with the attribute replaced
                            let mut attrs = Vec::new();
                            for (name, value) in element.value().attrs() {
                                if name == attr_to_change {
                                    // Replace this attribute
                                    attrs.push((name, new_value.as_str()));
                                } else if name == &self.prefix
                                    || name == &format!("{}-a", self.prefix)
                                {
                                    // Skip our special attributes
                                    continue;
                                } else {
                                    // Keep other attributes
                                    attrs.push((name, value));
                                }
                            }

                            // Now replace the entire element
                            let html = element.html();
                            let mut replacement = String::new();
                            replacement.push_str(&format!("<{}", element.value().name()));
                            for (name, value) in attrs {
                                replacement.push_str(&format!(" {}=\"{}\"", name, value));
                            }
                            replacement.push_str(">");

                            // If the element has children, we need to append them
                            if element.children().count() > 0 {
                                for child in element.children() {
                                    if let Some(html) = child.value().as_text() {
                                        replacement.push_str(html);
                                    }
                                }
                                replacement.push_str(&format!("</{}>", element.value().name()));
                            }

                            replacements.push((html, replacement));
                        }
                    }
                }
            }
        }

        // 3. Process generator placeholders
        for (generator_name, output) in generator_outputs {
            // Create string first, then parse
            let placeholder_selector =
                format!("[{}-placeholder=\"{}\"]", self.prefix, generator_name);
            let selector = match Selector::parse(&placeholder_selector) {
                Ok(s) => s,
                Err(e) => {
                    warn!("Invalid selector '{}': {:?}", placeholder_selector, e);
                    continue;
                }
            };

            for element in document.select(&selector) {
                let html = element.html();
                let replacement =
                    if let Some(handler) = self.placeholder_handlers.get(generator_name) {
                        handler(output)
                    } else {
                        output.clone()
                    };
                replacements.push((html, replacement));
            }
        }

        // 4. Process content area
        if let Some(content_handler) = &self.content_handler {
            // Create string first, then parse
            let content_selector = format!("[{}=\"content\"]", self.prefix);
            let selector = match Selector::parse(&content_selector) {
                Ok(s) => s,
                Err(e) => {
                    warn!("Invalid selector '{}': {:?}", content_selector, e);
                    return Ok(html_str.to_string()); // Return original if can't parse
                }
            };

            for element in document.select(&selector) {
                let html = element.html();
                let replacement = content_handler(content);
                replacements.push((html, replacement));
            }
        }

        // Sort replacements by length to replace longer fragments first
        replacements.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        // Apply all replacements to the HTML string
        let mut result = html_str.to_string();
        for (original, replacement) in replacements {
            debug!("Replacing: {} -> {}", original, replacement);
            result = result.replace(&original, &replacement);
        }

        // Remove any remaining data-ssg attributes using regex
        let data_attr_pattern = format!(r#"\s+{}(?:-[a-z]+)?="[^"]*""#, self.prefix);
        if let Ok(re) = regex::Regex::new(&data_attr_pattern) {
            result = re.replace_all(&result, "").to_string();
        }

        debug!("Final processed HTML length: {}", result.len());
        trace!("Final processed HTML: {}", result);

        Ok(result)
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
        self.process_html(html, metadata, generator_outputs, content)
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
        assert!(processed.contains(r#"<meta name="description" content="Replaced description">"#));
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
    fn test_attribute_processor_with_attr_replacement() {
        // Setup
        let processor = AttributeProcessor::new("data-ssg").with_default_handlers();

        // Simplified test HTML with just the meta tag
        let html = r#"<meta name="description" data-ssg="description" data-ssg-a="content" content="Default description">"#;

        let mut metadata = HashMap::new();
        metadata.insert(
            "description".to_string(),
            "Replaced description".to_string(),
        );

        // Execute
        let processed = processor
            .process(html, &metadata, &HashMap::new(), "")
            .unwrap();

        // More specific assertion
        assert!(processed.contains("content=\"Replaced description\""));
        assert!(!processed.contains("data-ssg"));
    }

    #[test]
    fn test_attribute_processor_with_complex_attr_replacement() {
        // Setup
        let processor = AttributeProcessor::new("data-ssg").with_default_handlers();

        // Complex test with multiple attributes and nested elements
        let html = r#"<html lang="en">
        <head>
            <title data-ssg="title">Default Title</title>
            <meta name="description" data-ssg="description" data-ssg-a="content" content="Default description">
            <link rel="canonical" data-ssg="canonical" data-ssg-a="href" href="https://default.com">
        </head>
        <body>
            <div data-ssg="content"></div>
        </body>
        </html>"#;

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Replaced Title".to_string());
        metadata.insert(
            "description".to_string(),
            "Replaced description".to_string(),
        );
        metadata.insert("canonical".to_string(), "https://example.com".to_string());

        // Execute
        let processed = processor
            .process(html, &metadata, &HashMap::new(), "<p>Page content</p>")
            .unwrap();

        // Assertions
        assert!(processed.contains("<title>Replaced Title</title>"));
        assert!(processed.contains("content=\"Replaced description\""));
        assert!(processed.contains("href=\"https://example.com\""));
        assert!(processed.contains("<div id=\"app\"><p>Page content</p></div>"));
        assert!(!processed.contains("data-ssg=")); // Ensure data attributes are removed
    }
}
