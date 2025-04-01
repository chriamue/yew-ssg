use crate::generator_collection::GeneratorCollection;
use crate::processor::Processor;
use log::{debug, trace, warn};
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

/// Processor that replaces placeholder elements with generated content.
///
/// It searches for elements with `data-ssg-placeholder="key"` attributes and replaces
/// them with content from the corresponding generator output or by dynamically
/// generating the content on demand using the appropriate generator.
#[derive(Clone)]
pub struct PlaceholderProcessor {
    /// The prefix used for placeholder attributes (e.g., "data-ssg")
    prefix: String,
    /// Reference to generators for on-demand content generation
    generators: Arc<GeneratorCollection>,
}

impl fmt::Debug for PlaceholderProcessor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlaceholderProcessor")
            .field("prefix", &self.prefix)
            .finish()
    }
}

impl PlaceholderProcessor {
    /// Create a new placeholder processor with the given prefix
    pub fn new(prefix: &str, generators: GeneratorCollection) -> Self {
        Self {
            prefix: prefix.to_string(),
            generators: Arc::new(generators),
        }
    }

    /// Process HTML to replace placeholders with generated content
    fn process_placeholders(
        &self,
        html_str: &str,
        route: &str,
        content: &str,
        metadata: &HashMap<String, String>,
        generator_outputs: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        let document = Html::parse_document(html_str);
        let mut result = html_str.to_string();
        let mut replacements = Vec::new();

        // Create selector for placeholder elements
        let selector_string = format!("[{}-placeholder]", self.prefix);
        let selector = match Selector::parse(&selector_string) {
            Ok(s) => s,
            Err(e) => {
                warn!("Invalid selector '{}': {:?}", selector_string, e);
                return Ok(html_str.to_string());
            }
        };

        // Find all placeholder elements
        for element in document.select(&selector) {
            if let Some(key) = element
                .value()
                .attr(&format!("{}-placeholder", self.prefix))
            {
                debug!("Found placeholder for key: {}", key);
                let original = element.html();

                // Determine replacement content
                let replacement = if let Some(output) = generator_outputs.get(key) {
                    // Use pre-generated output if available
                    debug!("Using pre-generated output for key: {}", key);
                    output.clone()
                } else {
                    // Generate content on demand
                    debug!("Generating content on demand for key: {}", key);
                    self.generate_content_for_key(key, route, content, metadata)?
                };

                replacements.push((original, replacement));
            }
        }

        // Apply all replacements
        for (original, replacement) in replacements {
            trace!("Replacing: {} -> {}", original, replacement);
            result = result.replace(&original, &replacement);
        }

        // Remove any remaining placeholder attributes
        let attr_pattern = format!(r#"\s+{}-placeholder="[^"]*""#, self.prefix);
        if let Ok(re) = regex::Regex::new(&attr_pattern) {
            result = re.replace_all(&result, "").to_string();
        }

        Ok(result)
    }

    /// Generate content for a specific key using the appropriate generator
    fn generate_content_for_key(
        &self,
        key: &str,
        route: &str,
        content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        // Find a generator that supports this output key
        for generator in self.generators.iter() {
            if let Some(support) = self.generators.try_get_output_support(generator) {
                if support.supports_output(key) {
                    debug!("Found generator for key '{}': {}", key, generator.name());
                    return generator.generate(key, route, content, metadata);
                }
            }
        }

        // If no generator supports this key, return an empty string or a warning comment
        warn!("No generator found for placeholder key: {}", key);
        Ok(format!(
            "<!-- No generator found for placeholder key: {} -->",
            key
        ))
    }
}

impl Processor for PlaceholderProcessor {
    fn name(&self) -> &'static str {
        "placeholder_processor"
    }

    fn process(
        &self,
        html: &str,
        metadata: &HashMap<String, String>,
        generator_outputs: &HashMap<String, String>,
        content: &str,
    ) -> Result<String, Box<dyn Error>> {
        // We'll use the empty string as the route when calling from this interface
        // In a real application, you might want to pass the route through the processor chain
        let route = "";
        self.process_placeholders(html, route, content, metadata, generator_outputs)
    }

    fn clone_box(&self) -> Box<dyn Processor> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::generator::tests::MockGenerator;
    use crate::generator_collection::GeneratorCollection;

    #[test]
    fn test_basic_placeholder_replacement() {
        // Setup generators
        let mut generators = GeneratorCollection::new();
        generators.add(MockGenerator::new("meta", vec!["title", "description"]));

        // Setup processor
        let processor = PlaceholderProcessor::new("data-ssg", generators);

        // Test HTML with placeholders
        let html = r#"<!DOCTYPE html>
        <html>
        <head>
            <div data-ssg-placeholder="meta"></div>
            <div data-ssg-placeholder="title"></div>
        </head>
        <body>
            <div id="content">Test content</div>
        </body>
        </html>"#;

        // Process the HTML
        let result = processor
            .process(html, &HashMap::new(), &HashMap::new(), "Original content")
            .unwrap();

        // Verify placeholders were replaced
        assert!(result.contains("<div>Generated content for key 'meta' by meta</div>"));
        assert!(result.contains("<div>Generated content for key 'title' by meta</div>"));
        assert!(!result.contains("data-ssg-placeholder"));
    }

    #[test]
    fn test_with_pre_generated_output() {
        // Setup generators
        let mut generators = GeneratorCollection::new();
        generators.add(MockGenerator::new("meta", vec!["description"]));

        // Setup processor
        let processor = PlaceholderProcessor::new("data-ssg", generators);

        // Test HTML with placeholders
        let html = r#"<div data-ssg-placeholder="meta"></div>"#;

        // Prepare pre-generated output
        let mut generator_outputs = HashMap::new();
        generator_outputs.insert(
            "meta".to_string(),
            "<meta name=\"description\" content=\"Pre-generated\">".to_string(),
        );

        // Process the HTML
        let result = processor
            .process(html, &HashMap::new(), &generator_outputs, "")
            .unwrap();

        // Verify the pre-generated output was used
        assert!(result.contains("<meta name=\"description\" content=\"Pre-generated\">"));
        assert!(!result.contains("data-ssg-placeholder"));
    }

    #[test]
    fn test_unsupported_placeholder() {
        // Setup generators
        let mut generators = GeneratorCollection::new();
        generators.add(MockGenerator::new("meta", vec!["description"]));

        // Setup processor
        let processor = PlaceholderProcessor::new("data-ssg", generators);

        // Test HTML with an unsupported placeholder
        let html = r#"<div data-ssg-placeholder="unsupported"></div>"#;

        // Process the HTML
        let result = processor
            .process(html, &HashMap::new(), &HashMap::new(), "")
            .unwrap();

        // Verify a warning comment was inserted
        assert!(result.contains("<!-- No generator found for placeholder key: unsupported -->"));
        assert!(!result.contains("data-ssg-placeholder"));
    }

    #[test]
    fn test_multiple_placeholders() {
        // Setup generators
        let mut generators = GeneratorCollection::new();
        generators.add(MockGenerator::new("meta", vec!["description"]));
        generators.add(MockGenerator::new("og", vec!["og:title", "og:image"]));

        // Setup processor
        let processor = PlaceholderProcessor::new("data-ssg", generators);

        // Test HTML with multiple placeholders
        let html = r#"<head>
            <div data-ssg-placeholder="meta"></div>
            <div data-ssg-placeholder="og"></div>
            <div data-ssg-placeholder="og:title"></div>
        </head>"#;

        // Process the HTML
        let result = processor
            .process(html, &HashMap::new(), &HashMap::new(), "")
            .unwrap();

        // Verify all placeholders were replaced
        assert!(result.contains("<div>Generated content for key 'meta' by meta</div>"));
        assert!(result.contains("<div>Generated content for key 'og' by og</div>"));
        assert!(result.contains("<div>Generated content for key 'og:title' by og</div>"));
        assert!(!result.contains("data-ssg-placeholder"));
    }

    #[test]
    fn test_nested_placeholders() {
        // Setup generators
        let mut generators = GeneratorCollection::new();
        generators.add(MockGenerator::new("outer", vec![]));

        // Setup processor
        let processor = PlaceholderProcessor::new("data-ssg", generators);

        // Test HTML with nested placeholders
        let html = r#"<div data-ssg-placeholder="outer">
            <span data-ssg-placeholder="inner">Inner content</span>
        </div>"#;

        // Prepare outputs that should be used in correct order
        let mut generator_outputs = HashMap::new();
        generator_outputs.insert(
            "outer".to_string(),
            "<div class=\"outer\">Replaced outer</div>".to_string(),
        );
        generator_outputs.insert(
            "inner".to_string(),
            "<span class=\"inner\">Replaced inner</span>".to_string(),
        );

        // Process the HTML
        let result = processor
            .process(html, &HashMap::new(), &generator_outputs, "")
            .unwrap();

        // Verify the outer placeholder was processed correctly
        assert!(result.contains("<div class=\"outer\">Replaced outer</div>"));
        assert!(!result.contains("data-ssg-placeholder=\"outer\""));

        // Inner placeholder should be gone since the entire outer element was replaced
        assert!(!result.contains("data-ssg-placeholder=\"inner\""));
    }
}
