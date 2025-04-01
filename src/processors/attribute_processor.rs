use crate::generator_collection::GeneratorCollection;
use crate::processor::Processor;
use log::{debug, trace, warn};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

/// Processor that handles attribute-based templating.
///
/// This processor focuses on two main tasks:
/// 1. Replacing content of elements with `data-ssg="key"` using metadata values
/// 2. Updating attributes of elements using `data-ssg="key" data-ssg-a="attribute"`
///
/// The processor is deliberately kept simple and focused for better maintainability.
#[derive(Clone)]
pub struct AttributeProcessor {
    /// The prefix used for data attributes (e.g., "data-ssg")
    prefix: String,

    /// Reference to generators for possible on-demand content generation
    generators: Arc<Option<GeneratorCollection>>,
}

impl fmt::Debug for AttributeProcessor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AttributeProcessor")
            .field("prefix", &self.prefix)
            .finish()
    }
}

impl AttributeProcessor {
    /// Create a new attribute processor with the given prefix
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
            generators: Arc::new(None),
        }
    }

    /// Optionally configure the processor with generators for on-demand generation
    pub fn with_generators(mut self, generators: GeneratorCollection) -> Self {
        self.generators = Arc::new(Some(generators));
        self
    }

    /// Add standard handlers for common content cases
    pub fn with_default_handlers(self) -> Self {
        // No custom handlers to add - we'll handle common cases directly in process_html
        self
    }

    /// Process HTML to handle attribute-based content and attribute replacements
    fn process_html(
        &self,
        html_str: &str,
        metadata: &HashMap<String, String>,
        generator_outputs: &HashMap<String, String>,
        content: &str,
    ) -> Result<String, Box<dyn Error>> {
        let document = Html::parse_document(html_str);
        let mut result = html_str.to_string();
        let mut replacements = Vec::new();

        // 1. Process content replacements (data-ssg="key")
        self.process_content_replacements(&document, metadata, &mut replacements)?;

        // 2. Process attribute replacements (data-ssg="key" data-ssg-a="attr")
        self.process_attribute_replacements(&document, metadata, &mut replacements)?;

        // 3. Handle special case for content area
        self.process_content_area(&document, content, &mut replacements)?;

        // Apply all replacements (sorting by length to avoid nested replacement issues)
        replacements.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        for (original, replacement) in replacements {
            debug!("Replacing: {} -> {}", original, replacement);
            result = result.replace(&original, &replacement);
        }

        // Remove any remaining data-ssg attributes
        self.clean_data_attributes(&mut result)?;

        debug!("Final processed HTML length: {}", result.len());
        trace!("Final processed HTML: {}", result);

        Ok(result)
    }

    /// Process elements with data-ssg="key" to replace their content
    fn process_content_replacements(
        &self,
        document: &Html,
        metadata: &HashMap<String, String>,
        replacements: &mut Vec<(String, String)>,
    ) -> Result<(), Box<dyn Error>> {
        // Create selector for elements with data-ssg attribute
        let selector_string = format!("[{}]:not([{}-a])", self.prefix, self.prefix);
        let selector = match Selector::parse(&selector_string) {
            Ok(s) => s,
            Err(e) => {
                warn!("Invalid selector '{}': {:?}", selector_string, e);
                return Ok(());
            }
        };

        for element in document.select(&selector) {
            if let Some(key) = element.value().attr(&self.prefix) {
                // Skip content element as it's handled separately
                if key == "content" {
                    continue;
                }

                if let Some(value) = metadata.get(key) {
                    debug!(
                        "Processing content replacement for {}=\"{}\"",
                        self.prefix, key
                    );

                    // Get the original element HTML
                    let original = element.html();

                    // Create a replacement with the same tag but new content
                    let tag_name = element.value().name();
                    let mut replacement = String::new();

                    // Preserve all attributes except our data attributes
                    replacement.push_str(&format!("<{}", tag_name));
                    for (attr_name, attr_value) in element.value().attrs() {
                        if !attr_name.starts_with(&self.prefix) {
                            replacement.push_str(&format!(" {}=\"{}\"", attr_name, attr_value));
                        }
                    }
                    replacement.push('>');

                    // Handle special case for meta tags and similar elements
                    if key == "title" {
                        replacement.push_str(&format!("{}", value));
                    } else if key == "description" && tag_name == "meta" {
                        // Do nothing - we'll keep the original attributes but remove content
                    } else {
                        // For regular elements, just replace the content
                        replacement.push_str(value);
                    }

                    replacement.push_str(&format!("</{}>", tag_name));
                    replacements.push((original, replacement));
                }
            }
        }

        Ok(())
    }

    /// Process elements with data-ssg="key" data-ssg-a="attribute" to modify attributes
    fn process_attribute_replacements(
        &self,
        document: &Html,
        metadata: &HashMap<String, String>,
        replacements: &mut Vec<(String, String)>,
    ) -> Result<(), Box<dyn Error>> {
        // Create selector for elements with both data-ssg and data-ssg-a attributes
        let selector_string = format!("[{}][{}-a]", self.prefix, self.prefix);
        let selector = match Selector::parse(&selector_string) {
            Ok(s) => s,
            Err(e) => {
                warn!("Invalid selector '{}': {:?}", selector_string, e);
                return Ok(());
            }
        };

        for element in document.select(&selector) {
            if let (Some(key), Some(attr_to_change)) = (
                element.value().attr(&self.prefix),
                element.value().attr(&format!("{}-a", self.prefix)),
            ) {
                if let Some(new_value) = metadata.get(key) {
                    debug!(
                        "Replacing attribute '{}' with value from '{}'",
                        attr_to_change, key
                    );

                    // Get the original element HTML
                    let original = element.html();

                    // Create a replacement with updated attribute
                    let mut replacement = String::new();
                    replacement.push_str(&format!("<{}", element.value().name()));

                    // Add all attributes, updating the target one
                    for (attr_name, attr_value) in element.value().attrs() {
                        if attr_name == attr_to_change {
                            // Replace with new value
                            replacement.push_str(&format!(" {}=\"{}\"", attr_name, new_value));
                        } else if !attr_name.starts_with(&self.prefix) {
                            // Keep non-data attributes
                            replacement.push_str(&format!(" {}=\"{}\"", attr_name, attr_value));
                        }
                        // Skip our data-ssg attributes
                    }

                    // Close the opening tag
                    replacement.push('>');

                    // Add any content for non-void elements
                    let has_children = element.children().count() > 0;
                    if has_children {
                        // For elements with children, preserve the original inner HTML
                        // This is a simplification that works for most cases
                        let inner_html = element.inner_html();
                        replacement.push_str(&inner_html);
                        replacement.push_str(&format!("</{}>", element.value().name()));
                    }

                    replacements.push((original, replacement));
                }
            }
        }

        Ok(())
    }

    /// Special handling for the content area (data-ssg="content")
    fn process_content_area(
        &self,
        document: &Html,
        content: &str,
        replacements: &mut Vec<(String, String)>,
    ) -> Result<(), Box<dyn Error>> {
        // Create selector for content area
        let selector_string = format!("[{}=\"content\"]", self.prefix);
        let selector = match Selector::parse(&selector_string) {
            Ok(s) => s,
            Err(e) => {
                warn!("Invalid selector '{}': {:?}", selector_string, e);
                return Ok(());
            }
        };

        for element in document.select(&selector) {
            debug!("Processing content area");

            // Get the original element HTML
            let original = element.html();

            // Create a replacement with app div
            let mut replacement = String::new();
            replacement.push_str(&format!("<{}", element.value().name()));

            // Add all attributes except our data attributes
            for (attr_name, attr_value) in element.value().attrs() {
                if !attr_name.starts_with(&self.prefix) {
                    replacement.push_str(&format!(" {}=\"{}\"", attr_name, attr_value));
                }
            }

            // Add id="app" if not already present
            if element.value().attr("id").is_none() {
                replacement.push_str(" id=\"app\"");
            }

            // Close tag and add content
            replacement.push('>');
            replacement.push_str(content);
            replacement.push_str(&format!("</{}>", element.value().name()));

            replacements.push((original, replacement));
        }

        Ok(())
    }

    /// Remove any remaining data-ssg attributes
    fn clean_data_attributes(&self, html: &mut String) -> Result<(), Box<dyn Error>> {
        // Remove data-ssg and data-ssg-* attributes
        let data_attr_pattern = format!(r#"\s+{}(?:-[a-z]+)?="[^"]*""#, self.prefix);
        if let Ok(re) = regex::Regex::new(&data_attr_pattern) {
            *html = re.replace_all(html, "").to_string();
        }

        Ok(())
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
    use crate::generator::tests::MockGenerator;
    use crate::generator_collection::GeneratorCollection;

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
        assert!(processed.contains("New Content"));
        assert!(!processed.contains("data-test=")); // Ensure data attributes are removed
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

    #[test]
    fn test_content_area_preserves_attributes() {
        // Setup
        let processor = AttributeProcessor::new("data-ssg").with_default_handlers();

        // Test with content area that has additional attributes
        let html = r#"<div class="container" id="root" data-ssg="content"></div>"#;

        // Execute
        let processed = processor
            .process(html, &HashMap::new(), &HashMap::new(), "<p>Content</p>")
            .unwrap();

        // Assert that other attributes are preserved
        assert!(processed.contains("<div class=\"container\" id=\"root\"><p>Content</p></div>"));
        assert!(!processed.contains("data-ssg"));
    }

    #[test]
    fn test_content_area_adds_app_id() {
        // Setup
        let processor = AttributeProcessor::new("data-ssg").with_default_handlers();

        // Test with content area without id
        let html = r#"<div class="container" data-ssg="content"></div>"#;

        // Execute
        let processed = processor
            .process(html, &HashMap::new(), &HashMap::new(), "<p>Content</p>")
            .unwrap();

        // Assert that id="app" was added
        assert!(processed.contains("<div class=\"container\" id=\"app\"><p>Content</p></div>"));
    }
}
