use crate::generator_collection::GeneratorCollection;
use crate::processor::Processor;
use log::{debug, trace, warn};
use regex::Regex;
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
        // In a real implementation, it might configure additional internal handlers
        self
    }

    /// Try to generate output using available generators
    fn try_generate_output(
        &self,
        generators: &GeneratorCollection,
        key: &str,
        route: &str,
        content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        for generator in generators.iter() {
            // Try each generator until one succeeds
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

        // Extract route from metadata if available
        let route = metadata.get("route").map_or("/", |r| r.as_str());

        let mut result_html = html.to_string();

        // Find all elements with our directive attribute
        let prefix = regex::escape(&self.prefix);
        let directive_pattern = format!(
            r#"<(?P<tag>\w+)(?P<attributes>.*?)\s+{}="(?P<key>[^"]*)"(?P<closing>.*?)>"#,
            prefix
        );
        let re = Regex::new(&directive_pattern)?;

        // Need to process matches in reverse to avoid index shifts
        let mut matches = Vec::new();
        for cap in re.captures_iter(html) {
            let full_match = cap.get(0).unwrap().as_str();
            let tag = cap.name("tag").map_or("", |m| m.as_str());
            let attributes = cap.name("attributes").map_or("", |m| m.as_str());
            let key = cap.name("key").map_or("", |m| m.as_str());
            let closing = cap.name("closing").map_or("", |m| m.as_str());

            // Store the match and its details
            matches.push((
                full_match.to_string(),
                tag.to_string(),
                attributes.to_string(),
                key.to_string(),
                closing.to_string(),
            ));
        }

        // Process matches in reverse order to maintain indices
        for (full_match, tag, attributes, key, closing) in matches.iter().rev() {
            trace!("Processing element with {}=\"{}\"", self.prefix, key);

            // Generate content for this key
            let generated_content = if let Some(content) = generator_outputs.get(key) {
                content.clone()
            } else if let Some(generators) = &*self.generators {
                match self.try_generate_output(generators, key, route, content, metadata) {
                    Ok(content) => content,
                    Err(e) => {
                        warn!("Failed to generate content for key '{}': {}", key, e);
                        String::new()
                    }
                }
            } else {
                String::new()
            };

            let mut new_attributes = attributes.to_string();

            // Find all data-ssg-* attributes in this tag
            let data_attr_prefix_pattern = format!(r#"{}-([\w-]+)="([^"]*)""#, prefix);
            let re_prefix_attrs = Regex::new(&data_attr_prefix_pattern)?;

            for prefix_cap in re_prefix_attrs.captures_iter(attributes) {
                let target_attr = prefix_cap.get(1).map_or("", |m| m.as_str());

                // Remove the data-ssg-* attribute
                let prefix_attr_pattern = format!(r#"\s+{}-{}="[^"]*""#, prefix, target_attr);
                new_attributes = Regex::new(&prefix_attr_pattern)?
                    .replace_all(&new_attributes, "")
                    .to_string();

                // Set the target attribute with generated content
                let new_attr_str = format!(
                    r#" {}="{}""#,
                    target_attr,
                    regex::escape(&generated_content)
                );

                // Check if attribute exists
                let existing_attr_pattern = format!(r#"\s+{}="[^"]*""#, target_attr);
                if Regex::new(&existing_attr_pattern)?.is_match(&new_attributes) {
                    new_attributes = Regex::new(&existing_attr_pattern)?
                        .replace(
                            &new_attributes,
                            &format!(r#" {}="{}""#, target_attr, generated_content),
                        )
                        .to_string();
                } else {
                    new_attributes = format!(
                        "{} {}=\"{}\"",
                        new_attributes, target_attr, generated_content
                    );
                }
            }

            // Remove the main data-ssg attribute
            let main_attr_pattern = format!(r#"\s+{}="[^"]*""#, prefix);
            new_attributes = Regex::new(&main_attr_pattern)?
                .replace_all(&new_attributes, "")
                .to_string();

            let new_tag = format!("<{}{}{}>", tag, new_attributes, closing);

            // Replace the original tag in the result HTML
            result_html = result_html.replace(full_match, &new_tag);
        }

        Ok(result_html)
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
        // Setup test environment
        let mut generators = GeneratorCollection::new();
        generators.add(MockGenerator::new("title", vec!["title"]));

        let processor = AttributeProcessor::new("data-ssg", Some(generators));

        // Test simple attribute replacement
        let html = r#"<div data-ssg="title" data-ssg-content="title">Default content</div>"#;
        let mut metadata = HashMap::new();
        metadata.insert("route".to_string(), "/test".to_string());

        let result = processor
            .process(html, &metadata, &HashMap::new(), "Page content")
            .unwrap();

        assert!(
            result.contains(
                r#"<div content="<div>Generated content for key 'title' by title</div>">"#
            ),
            "Failed to replace attribute - got: {}",
            result
        );
    }

    #[test]
    fn test_attribute_processor_with_generator_outputs() {
        let processor = AttributeProcessor::new("data-ssg", None);

        let html = r#"<div data-ssg="meta" data-ssg-content="meta">Default</div>"#;
        let mut generator_outputs = HashMap::new();
        generator_outputs.insert("meta".to_string(), "Generated meta content".to_string());

        let result = processor
            .process(html, &HashMap::new(), &generator_outputs, "")
            .unwrap();

        assert!(
            result.contains(r#"<div content="Generated meta content">"#),
            "Failed to use pre-generated output - got: {}",
            result
        );
    }

    #[test]
    fn test_with_default_handlers() {
        let processor = AttributeProcessor::new("data-ssg", None).with_default_handlers();
        assert_eq!(processor.prefix, "data-ssg");
    }
}
