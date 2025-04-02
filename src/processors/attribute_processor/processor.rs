use crate::generator_collection::GeneratorCollection;
use crate::processor::Processor;
use crate::processors::attribute_processor::{process_element, SsgAttribute};
use log::{debug, warn};
use lol_html::{element, HtmlRewriter, Settings};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

/// Processor that handles attribute-based templating.
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
            .field(
                "generators",
                &if self.generators.is_some() {
                    "Some(GeneratorCollection)"
                } else {
                    "None"
                },
            )
            .finish()
    }
}

impl AttributeProcessor {
    /// Creates a new AttributeProcessor with the given prefix
    pub fn new(prefix: &str, generators: Option<GeneratorCollection>) -> Self {
        Self {
            prefix: prefix.to_string(),
            generators: Arc::new(generators),
        }
    }

    /// Creates a new AttributeProcessor with the default "data-ssg" prefix
    pub fn default_with_generators(generators: Option<GeneratorCollection>) -> Self {
        Self::new("data-ssg", generators)
    }

    /// Creates a version of AttributeProcessor with default handlers
    pub fn with_default_handlers(self) -> Self {
        // This method simply returns self as it's just a convenience method for tests
        self
    }

    /// Attempt to generate output using available generators
    fn try_generate_output(
        &self,
        generators: &GeneratorCollection,
        key: &str,
        route: &str,
        content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        for generator in generators.iter() {
            match generator.generate(key, route, content, metadata) {
                Ok(output) => return Ok(output),
                Err(_) => continue, // Try next generator
            }
        }
        Err(format!("No generator could produce output for key '{}'", key).into())
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
        debug!("Processing HTML with AttributeProcessor");
        let route = metadata.get("route").map_or("/", |r| r.as_str());
        let prefix = self.prefix.clone();
        let generators_ref = Arc::clone(&self.generators);

        // Prepare output buffer
        let mut output = Vec::new();

        // Create separate handlers
        let mut handlers = Vec::new();

        // We need to use a wildcard selector and then filter attributes in the handler
        // because lol_html has limitations with attribute selectors containing special chars
        let self_ref = self.clone();
        handlers.push(element!("*", move |el| {
            // First check for data-ssg-placeholder attribute
            let placeholder_attr = format!("{}-placeholder", prefix);
            if let Some(key) = el.get_attribute(&placeholder_attr) {
                // Get or generate content for placeholder
                let generated_content = if let Some(prebuilt) = generator_outputs.get(&key) {
                    prebuilt.clone()
                } else if let Some(generators) = &*generators_ref {
                    match self_ref.try_generate_output(generators, &key, route, content, metadata) {
                        Ok(generated) => generated,
                        Err(e) => {
                            warn!(
                                "Failed to generate content for placeholder key '{}': {}",
                                key, e
                            );
                            format!("<!-- No generator found for placeholder key: {} -->", key)
                        }
                    }
                } else {
                    format!("<!-- No generator found for placeholder key: {} -->", key)
                };

                // Process placeholder element (completely replace it)
                process_element(SsgAttribute::Placeholder, el, &generated_content, metadata);
                return Ok(());
            }

            // Then, check for data-ssg attribute (like data-ssg="content")
            if let Some(key) = el.get_attribute(&prefix) {
                // Get or generate content, checking in this order:
                // 1. Generator outputs
                // 2. Generators
                // 3. Special case for "content"
                // 4. Metadata
                // 5. If none of the above, preserve original
                let generated_content = if let Some(prebuilt) = generator_outputs.get(&key) {
                    prebuilt.clone()
                } else if let Some(generators) = &*generators_ref {
                    match self_ref.try_generate_output(generators, &key, route, content, metadata) {
                        Ok(generated) => generated,
                        Err(e) => {
                            warn!("Failed to generate content for key '{}': {}", key, e);
                            // Try to get from metadata
                            if let Some(value) = metadata.get(&key) {
                                value.clone()
                            } else {
                                "{{__PRESERVE_ORIGINAL__}}".to_string()
                            }
                        }
                    }
                } else if key == "content" {
                    content.to_string()
                } else if let Some(value) = metadata.get(&key) {
                    // Get from metadata if available
                    value.clone()
                } else {
                    // Preserve original if no replacement found
                    "{{__PRESERVE_ORIGINAL__}}".to_string()
                };

                // Process element, handling the special flag if needed
                process_element(SsgAttribute::Content, el, &generated_content, metadata);
            }

            // Next, look for data-ssg-* attributes
            let prefix_dash = format!("{}-", prefix);

            // Collect attribute names and values for data-ssg-*
            let data_attrs: Vec<(String, String)> = el
                .attributes()
                .into_iter()
                .filter(|attr| {
                    attr.name().starts_with(&prefix_dash) && attr.name() != placeholder_attr
                    // Skip placeholder attribute as we already handled it
                })
                .map(|attr| (attr.name().to_string(), attr.value().to_string()))
                .collect();

            // Process each data-ssg-* attribute
            for (attr_name, key) in data_attrs {
                if let Some(attr_target) = attr_name.strip_prefix(&prefix_dash) {
                    let attr_target = attr_target.to_string();

                    // Get or generate content
                    let generated_content = if let Some(prebuilt) = generator_outputs.get(&key) {
                        prebuilt.clone()
                    } else if let Some(generators) = &*generators_ref {
                        match self_ref
                            .try_generate_output(generators, &key, route, content, metadata)
                        {
                            Ok(generated) => generated,
                            Err(e) => {
                                warn!("Failed to generate content for key '{}': {}", key, e);
                                String::new()
                            }
                        }
                    } else {
                        String::new()
                    };

                    process_element(
                        SsgAttribute::Attribute(attr_target),
                        el,
                        &generated_content,
                        metadata,
                    );
                }
            }

            Ok(())
        }));

        // Create the HTML rewriter
        let mut rewriter = HtmlRewriter::new(
            Settings {
                element_content_handlers: handlers,
                ..Settings::default()
            },
            |c: &[u8]| output.extend_from_slice(c),
        );

        // Process the HTML
        rewriter.write(html.as_bytes())?;
        rewriter.end()?;

        // Convert output to string
        let result = String::from_utf8(output)?;
        Ok(result)
    }

    fn clone_box(&self) -> Box<dyn Processor> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::tests::MockGenerator;

    #[test]
    fn test_attribute_processor_basic() {
        let mut generators = GeneratorCollection::new();
        generators.add(MockGenerator::new("title", vec!["title"]));

        let processor = AttributeProcessor::new("data-ssg", Some(generators));

        let html = r#"<!DOCTYPE html>
        <html>
        <head>
            <div data-ssg="title">Default content</div>
        </head>
        <body>
            <p>This content should be preserved</p>
        </body>
        </html>"#;

        let mut metadata = HashMap::new();
        metadata.insert("route".to_string(), "/test".to_string());

        let result = processor
            .process(html, &metadata, &HashMap::new(), "Page content")
            .unwrap();

        // Verify replaced content
        assert!(
            result.contains(r#"<div>Generated content for key 'title' by title</div>"#),
            "Failed to replace content - got: {}",
            result
        );

        // Verify other parts
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("<html>"));
        assert!(result.contains("<head>"));
        assert!(result.contains("</head>"));
        assert!(result.contains("<body>"));
        assert!(result.contains("<p>This content should be preserved</p>"));
        assert!(result.contains("</body>"));
        assert!(result.contains("</html>"));
    }

    #[test]
    fn test_attribute_processor_with_generator_outputs() {
        let processor = AttributeProcessor::new("data-ssg", None);

        let html = r#"<!DOCTYPE html>
        <html>
        <head>
            <meta data-ssg-content="meta" content="default"/>
            <title>Page Title</title>
        </head>
        <body>
            <h1>Header</h1>
        </body>
        </html>"#;

        let mut generator_outputs = HashMap::new();
        generator_outputs.insert("meta".to_string(), "Generated meta content".to_string());

        let result = processor
            .process(html, &HashMap::new(), &generator_outputs, "")
            .unwrap();

        // Check for attribute replacement
        assert!(
            result.contains(r#"content="Generated meta content""#),
            "Failed to use pre-generated output - got: {}",
            result
        );

        // Check other parts
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("<title>Page Title</title>"));
        assert!(result.contains("<h1>Header</h1>"));
    }

    #[test]
    fn test_content_replacement() {
        let processor = AttributeProcessor::new("data-ssg", None);

        let html = r#"<!DOCTYPE html>
        <html>
        <head>
            <title>Page Title</title>
        </head>
        <body>
            <div id="app">
                <div data-ssg="content"></div>
            </div>
        </body>
        </html>"#;

        let content = "<p>This is the page content</p>";

        let result = processor
            .process(html, &HashMap::new(), &HashMap::new(), content)
            .unwrap();

        // Check for content replacement
        assert!(
            result.contains("<p>This is the page content</p>"),
            "Failed to replace content - got: {}",
            result
        );

        // Check other parts
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("<title>Page Title</title>"));
        assert!(result.contains("<div id=\"app\">"));
        assert!(result.contains("</div>"));
    }

    #[test]
    fn test_title_replacement() {
        let processor = AttributeProcessor::new("data-ssg", None);

        let html = r#"<!DOCTYPE html>
        <html>
        <head>
            <title data-ssg="title">Default Title</title>
            <meta name="description" content="Description">
        </head>
        <body>
            <h1>Header</h1>
        </body>
        </html>"#;

        let mut generator_outputs = HashMap::new();
        generator_outputs.insert("title".to_string(), "Generated Title".to_string());

        let result = processor
            .process(html, &HashMap::new(), &generator_outputs, "")
            .unwrap();

        // Check for title replacement
        assert!(
            result.contains("<title>Generated Title</title>"),
            "Failed to replace title - got: {}",
            result
        );

        // Check other parts
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("<meta name=\"description\" content=\"Description\">"));
        assert!(result.contains("<h1>Header</h1>"));
    }

    #[test]
    fn test_multiple_attributes_on_same_element() {
        let processor = AttributeProcessor::new("data-ssg", None);

        let html = r#"<!DOCTYPE html>
        <html>
        <head>
            <meta
                data-ssg-content="meta-content"
                data-ssg-name="meta-name"
                content="default-content"
                name="default-name">
        </head>
        <body>
            <h1>Header</h1>
        </body>
        </html>"#;

        let mut generator_outputs = HashMap::new();
        generator_outputs.insert("meta-content".to_string(), "Generated content".to_string());
        generator_outputs.insert("meta-name".to_string(), "Generated name".to_string());

        let result = processor
            .process(html, &HashMap::new(), &generator_outputs, "")
            .unwrap();

        // Check that both attributes were replaced
        assert!(result.contains(r#"content="Generated content""#));
        assert!(result.contains(r#"name="Generated name""#));

        // Check that data-ssg-* attributes were removed
        assert!(!result.contains("data-ssg-content"));
        assert!(!result.contains("data-ssg-name"));
    }

    #[test]
    fn test_placeholder_replacement() {
        let mut generators = GeneratorCollection::new();
        generators.add(MockGenerator::new("meta", vec!["description"]));

        let processor = AttributeProcessor::new("data-ssg", Some(generators));

        let html = r#"<!DOCTYPE html>
        <html>
        <head>
            <div data-ssg-placeholder="description">Loading description...</div>
        </head>
        <body>
            <h1>Test Page</h1>
        </body>
        </html>"#;

        let mut metadata = HashMap::new();
        metadata.insert("route".to_string(), "/test".to_string());

        let result = processor
            .process(html, &metadata, &HashMap::new(), "")
            .unwrap();

        // Verify the placeholder was replaced
        assert!(result.contains("Generated content for key 'description' by meta"));

        // Verify the placeholder element and attribute are gone
        assert!(!result.contains("data-ssg-placeholder"));
        assert!(!result.contains("<div>Loading description...</div>"));

        // Verify the document structure is still intact
        assert!(result.contains("<!DOCTYPE html>"));
        assert!(result.contains("<html>"));
        assert!(result.contains("<head>"));
        assert!(result.contains("</head>"));
        assert!(result.contains("<body>"));
        assert!(result.contains("<h1>Test Page</h1>"));
        assert!(result.contains("</body>"));
        assert!(result.contains("</html>"));
    }

    #[test]
    fn test_placeholder_with_pregenerated_content() {
        let processor = AttributeProcessor::new("data-ssg", None);

        let html = r#"<!DOCTYPE html>
        <html>
        <head>
            <div data-ssg-placeholder="meta-tags">Original placeholder content</div>
        </head>
        <body>
            <h1>Test Page</h1>
        </body>
        </html>"#;

        let mut generator_outputs = HashMap::new();
        generator_outputs.insert(
            "meta-tags".to_string(),
            r#"<meta name="description" content="Test description">
            <meta name="keywords" content="test,keywords">"#
                .to_string(),
        );

        let result = processor
            .process(html, &HashMap::new(), &generator_outputs, "")
            .unwrap();

        // Verify placeholder replaced with multiple meta tags
        assert!(result.contains(r#"<meta name="description" content="Test description">"#));
        assert!(result.contains(r#"<meta name="keywords" content="test,keywords">"#));

        // Verify original div and placeholder attribute are gone
        assert!(!result.contains("<div"));
        assert!(!result.contains("data-ssg-placeholder"));
        assert!(!result.contains("Original placeholder content"));
    }

    #[test]
    fn test_with_default_handlers() {
        let processor = AttributeProcessor::new("data-ssg", None).with_default_handlers();
        assert_eq!(processor.prefix, "data-ssg");
    }

    #[test]
    fn test_title_with_metadata_fallback() {
        let processor = AttributeProcessor::new("data-ssg", None);

        let html = r#"<!DOCTYPE html>
        <html>
        <head>
            <title data-ssg="title">Default Title</title>
        </head>
        <body>
            <h1>Test Page</h1>
        </body>
        </html>"#;

        // No generator outputs, but metadata has title
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Title from Metadata".to_string());

        let result = processor
            .process(html, &metadata, &HashMap::new(), "")
            .unwrap();

        // The title should use the metadata value when available
        assert!(
            result.contains("<title>Title from Metadata</title>"),
            "Title should use value from metadata when no generator output is available"
        );
    }
}
