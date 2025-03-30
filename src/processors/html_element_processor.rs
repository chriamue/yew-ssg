use crate::processor::Processor;
use log::{debug, warn};
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct HtmlElementProcessor {
    prefix: String,
}

impl HtmlElementProcessor {
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }

    /// Find elements by their data-*-placeholder attributes
    fn find_placeholder_elements<'a>(
        &self,
        document: &'a Html,
        generator_name: &str,
    ) -> Vec<ElementRef<'a>> {
        let selector_string = format!("[{}-placeholder=\"{}\"]", self.prefix, generator_name);
        let mut results = Vec::new();

        // This approach works because selector_string lives until the end of this function
        if let Ok(selector) = Selector::parse(&selector_string) {
            for element in document.select(&selector) {
                results.push(element);
            }
        } else {
            warn!("Invalid selector: {}", selector_string);
        }

        results
    }
}

impl Processor for HtmlElementProcessor {
    fn name(&self) -> &'static str {
        "html_element_processor"
    }

    fn process(
        &self,
        html: &str,
        _metadata: &HashMap<String, String>,
        generator_outputs: &HashMap<String, String>,
        _content: &str,
    ) -> Result<String, Box<dyn Error>> {
        // If there are no generator outputs, return unchanged
        if generator_outputs.is_empty() {
            return Ok(html.to_string());
        }

        let document = Html::parse_document(html);
        let mut result = html.to_string();
        let mut replacements = Vec::new();

        // Find all elements with data-ssg-placeholder attributes
        for (generator_name, output) in generator_outputs {
            // Use the helper method instead of inline code
            for element in self.find_placeholder_elements(&document, generator_name) {
                let original = element.html();
                debug!("Found placeholder for {}: {}", generator_name, original);
                replacements.push((original, output.clone()));
            }
        }

        // Apply all replacements (if any)
        if !replacements.is_empty() {
            debug!("Applying {} placeholder replacements", replacements.len());

            // Sort replacements by length to replace longer fragments first
            replacements.sort_by(|a, b| b.0.len().cmp(&a.0.len()));

            for (original, replacement) in replacements {
                result = result.replace(&original, &replacement);
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
    use std::collections::HashMap;

    #[test]
    fn test_element_processor_with_placeholders() {
        let processor = HtmlElementProcessor::new("data-ssg");

        let html = r#"<!DOCTYPE html>
        <html>
        <head>
            <meta data-ssg-placeholder="meta_tags" content="default">
            <meta data-ssg-placeholder="open_graph" content="default">
        </head>
        <body>
            <div>Content</div>
        </body>
        </html>"#;

        let mut generator_outputs = HashMap::new();
        generator_outputs.insert(
            "meta_tags".to_string(),
            "<meta name=\"description\" content=\"Test description\">".to_string(),
        );
        generator_outputs.insert(
            "open_graph".to_string(),
            "<meta property=\"og:title\" content=\"Test Title\">".to_string(),
        );

        let result = processor
            .process(html, &HashMap::new(), &generator_outputs, "")
            .unwrap();

        assert!(result.contains("<meta name=\"description\" content=\"Test description\">"));
        assert!(result.contains("<meta property=\"og:title\" content=\"Test Title\">"));
        assert!(!result.contains("data-ssg-placeholder=\"meta_tags\""));
    }

    #[test]
    fn test_element_processor_no_placeholders() {
        let processor = HtmlElementProcessor::new("data-ssg");

        let html = "<html><body>No placeholders</body></html>";
        let generator_outputs = HashMap::new();

        let result = processor
            .process(html, &HashMap::new(), &generator_outputs, "")
            .unwrap();

        assert_eq!(result, html);
    }
}
