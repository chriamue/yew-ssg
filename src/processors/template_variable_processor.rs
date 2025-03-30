use crate::processor::Processor;
use log::warn;
use regex::Regex;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct TemplateVariableProcessor {
    title_fallback_pattern: Option<String>,
}

impl TemplateVariableProcessor {
    pub fn new() -> Self {
        Self {
            title_fallback_pattern: Some("Page: {path}".to_string()),
        }
    }

    /// Set a custom title fallback pattern using {path} as a placeholder
    pub fn with_title_fallback(mut self, pattern: Option<String>) -> Self {
        self.title_fallback_pattern = pattern;
        self
    }
}

impl Processor for TemplateVariableProcessor {
    fn name(&self) -> &'static str {
        "template_variable_processor"
    }

    fn process(
        &self,
        html: &str,
        metadata: &HashMap<String, String>,
        generator_outputs: &HashMap<String, String>,
        _content: &str,
    ) -> Result<String, Box<dyn Error>> {
        // Check if a title is either in metadata, generator outputs, or existing HTML
        let has_title = metadata.contains_key("title")
            || generator_outputs.contains_key("title")
            || html.contains("<title>");

        // If there's no title but we have a fallback pattern, add a title
        if !has_title && self.title_fallback_pattern.is_some() {
            let path = metadata
                .get("path")
                .cloned()
                .unwrap_or_else(|| "".to_string());
            let fallback_title = self
                .title_fallback_pattern
                .as_ref()
                .unwrap()
                .replace("{path}", &path);

            // If there's a <head> tag, add the title inside it
            let with_title = if html.contains("<head>") {
                let re = Regex::new(r"<head>").unwrap();
                re.replace(html, &format!("<head>\n<title>{}</title>", fallback_title))
                    .to_string()
            } else {
                // If there's no head tag, warn but don't modify
                warn!("No <head> tag found to insert title for path: {}", path);
                html.to_string()
            };

            Ok(with_title)
        } else {
            // No changes needed
            Ok(html.to_string())
        }
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
    fn test_template_variable_processor_with_fallback() {
        let processor = TemplateVariableProcessor::new();

        let html = r#"<!DOCTYPE html><html><head></head><body><div id="app"></div></body></html>"#;
        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), "/test".to_string());

        let generator_outputs = HashMap::new();

        let processed = processor
            .process(html, &metadata, &generator_outputs, "")
            .unwrap();

        assert!(processed.contains("<title>Page: /test</title>"));
    }

    #[test]
    fn test_template_variable_processor_with_existing_title() {
        let processor = TemplateVariableProcessor::new();

        let html = r#"<!DOCTYPE html><html><head><title>Existing Title</title></head><body><div id="app"></div></body></html>"#;
        let metadata = HashMap::new();
        let generator_outputs = HashMap::new();

        let processed = processor
            .process(html, &metadata, &generator_outputs, "")
            .unwrap();

        assert!(processed.contains("<title>Existing Title</title>"));
        assert!(!processed.contains("Page:"));
    }

    #[test]
    fn test_template_variable_processor_without_head() {
        let processor = TemplateVariableProcessor::new();

        let html = r#"<!DOCTYPE html><html><body><div id="app"></div></body></html>"#;
        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), "/test".to_string());

        let generator_outputs = HashMap::new();

        let processed = processor
            .process(html, &metadata, &generator_outputs, "")
            .unwrap();

        assert!(!processed.contains("<title>Page: /test</title>"));
        assert!(processed.contains("<body><div id=\"app\"></div></body>"));
    }
}
