//! HtmlElementProcessor handles data-* attribute-based content replacement.
//!
//! This processor focuses on replacing the content of HTML elements while preserving
//! the elements themselves, based on data attributes in the HTML template.

use crate::processor::Processor;
use log::{debug, trace, warn};
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;
use std::error::Error;

/// A processor that replaces element content based on data-* attributes
/// while preserving the HTML element structure.
///
/// # Example
///
/// ```html
/// <!-- With this template: -->
/// <title data-ssg="title">Default Title</title>
///
/// <!-- And metadata {"title": "My Page"} -->
///
/// <!-- The result will be: -->
/// <title>My Page</title>
/// ```
#[derive(Debug, Clone)]
pub struct HtmlElementProcessor {
    /// The prefix used for data attributes, e.g., "data-ssg"
    prefix: String,
}

impl HtmlElementProcessor {
    /// Creates a new HtmlElementProcessor with the given attribute prefix.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix for data attributes, typically "data-ssg"
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }

    /// Find elements with data-ssg="key" attributes for content replacement
    fn find_content_replacement_elements<'a>(
        &self,
        document: &'a Html,
    ) -> Vec<(ElementRef<'a>, String)> {
        let selector_string = format!("[{}]", self.prefix);
        let mut results = Vec::new();

        if let Ok(selector) = Selector::parse(&selector_string) {
            for element in document.select(&selector) {
                if let Some(key) = element.value().attr(&self.prefix) {
                    results.push((element, key.to_string()));
                }
            }
        } else {
            warn!("Invalid selector: {}", selector_string);
        }

        results
    }

    /// Find elements by their data-*-placeholder attributes for generator output insertion
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

    /// Create a new element with the same tag and attributes but different content
    fn create_element_with_new_content(
        &self,
        element: ElementRef,
        new_content: &str,
        remove_data_attrs: bool,
    ) -> String {
        let mut result = String::new();

        // Start tag
        result.push('<');
        result.push_str(&element.value().name().to_string());

        // Add all attributes except our data-* ones if removal is requested
        for (name, value) in element.value().attrs() {
            if !remove_data_attrs
                || (!name.starts_with(&self.prefix)
                    && !name.starts_with(&format!("{}-", self.prefix)))
            {
                result.push(' ');
                result.push_str(&name.to_string());
                result.push_str("=\"");
                result.push_str(&value.to_string());
                result.push('"');
            }
        }

        result.push('>');

        // Add new content
        result.push_str(new_content);

        // End tag
        result.push_str("</");
        result.push_str(&element.value().name().to_string());
        result.push('>');

        result
    }
}

impl Processor for HtmlElementProcessor {
    fn name(&self) -> &'static str {
        "html_element_processor"
    }

    fn process(
        &self,
        html: &str,
        metadata: &HashMap<String, String>,
        generator_outputs: &HashMap<String, String>,
        _content: &str,
    ) -> Result<String, Box<dyn Error>> {
        let document = Html::parse_document(html);
        let mut result = html.to_string();
        let mut replacements = Vec::new();

        // 1. Process content replacement elements (data-ssg="key")
        for (element, key) in self.find_content_replacement_elements(&document) {
            if let Some(value) = metadata.get(&key) {
                debug!(
                    "Replacing content for element with {}=\"{}\"",
                    self.prefix, key
                );
                let original = element.html();

                // Create new element with same tag/attrs but new content
                let new_element = self.create_element_with_new_content(
                    element, value, true, // Remove data-* attributes
                );

                trace!("Replacement: {} -> {}", original, new_element);
                replacements.push((original, new_element));
            }
        }

        // 2. Process placeholder elements (data-ssg-placeholder="generator")
        for (generator_name, output) in generator_outputs {
            for element in self.find_placeholder_elements(&document, generator_name) {
                debug!("Processing placeholder for generator: {}", generator_name);
                let original = element.html();

                // For placeholders, we replace the entire element with generator output
                replacements.push((original, output.clone()));
            }
        }

        // Apply all replacements
        // Sort by length to replace longer fragments first (avoiding nested replacements issues)
        replacements.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

        for (original, replacement) in replacements {
            result = result.replace(&original, &replacement);
        }

        // Remove any remaining data-ssg attributes using regex
        let data_attr_pattern = format!(r#"\s+{}(?:-[a-z]+)?="[^"]*""#, self.prefix);
        if let Ok(re) = regex::Regex::new(&data_attr_pattern) {
            result = re.replace_all(&result, "").to_string();
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
    use std::collections::HashMap;

    #[test]
    fn test_content_replacement() {
        let processor = HtmlElementProcessor::new("data-ssg");

        let html = r#"<html>
            <head>
                <title data-ssg="title">Default Title</title>
                <meta name="description" data-ssg="description" content="Default description">
            </head>
            <body>
                <h1 data-ssg="title">Default Title</h1>
            </body>
        </html>"#;

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "My Custom Title".to_string());
        metadata.insert(
            "description".to_string(),
            "My custom description".to_string(),
        );

        let result = processor
            .process(html, &metadata, &HashMap::new(), "")
            .unwrap();

        // Check that element structure is preserved but content is replaced
        assert!(result.contains("<title>My Custom Title</title>"));
        assert!(result.contains("<h1>My Custom Title</h1>"));
        assert!(result.contains(
            r#"<meta name="description" content="Default description">My custom description</meta>"#
        ));

        // Ensure data attributes are removed
        assert!(!result.contains("data-ssg="));
    }

    #[test]
    fn test_placeholder_replacement() {
        let processor = HtmlElementProcessor::new("data-ssg");

        let html = r#"<head>
            <meta data-ssg-placeholder="open_graph" content="og-tags">
        </head>"#;

        let mut generator_outputs = HashMap::new();
        generator_outputs.insert(
            "open_graph".to_string(),
            r#"<meta property="og:type" content="website">
            <meta property="og:title" content="Test Title">"#
                .to_string(),
        );

        let result = processor
            .process(html, &HashMap::new(), &generator_outputs, "")
            .unwrap();

        assert!(result.contains(r#"<meta property="og:type" content="website">"#));
        assert!(!result.contains(r#"content="og-tags""#));
        assert!(!result.contains("data-ssg-placeholder"));
    }

    #[test]
    fn test_no_replacements() {
        let processor = HtmlElementProcessor::new("data-ssg");

        let html = "<html><body>No placeholders or data attributes</body></html>";

        let result = processor
            .process(html, &HashMap::new(), &HashMap::new(), "")
            .unwrap();

        assert_eq!(result, html);
    }

    #[test]
    fn test_complex_html_structure() {
        let processor = HtmlElementProcessor::new("data-ssg");

        let html = r#"<div>
            <section>
                <h2 data-ssg="section_title">Default Section Title</h2>
                <article data-ssg="article_content">
                    <p>Default paragraph 1</p>
                    <p>Default paragraph 2</p>
                </article>
            </section>
        </div>"#;

        let mut metadata = HashMap::new();
        metadata.insert(
            "section_title".to_string(),
            "Custom Section Title".to_string(),
        );
        metadata.insert(
            "article_content".to_string(),
            "<p>Custom paragraph with <strong>formatting</strong></p>".to_string(),
        );

        let result = processor
            .process(html, &metadata, &HashMap::new(), "")
            .unwrap();

        assert!(result.contains("<h2>Custom Section Title</h2>"));
        assert!(result.contains(
            "<article><p>Custom paragraph with <strong>formatting</strong></p></article>"
        ));
        assert!(!result.contains("Default paragraph"));
        assert!(!result.contains("data-ssg="));
    }
}
