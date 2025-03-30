use crate::processor::Processor;
use log::debug;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct TemplateVariableProcessor {
    start_delimiter: String,
    end_delimiter: String,
}

impl TemplateVariableProcessor {
    pub fn new() -> Self {
        Self {
            start_delimiter: "{{".to_string(),
            end_delimiter: "}}".to_string(),
        }
    }

    pub fn with_delimiters(mut self, start: &str, end: &str) -> Self {
        self.start_delimiter = start.to_string();
        self.end_delimiter = end.to_string();
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
        let mut result = html.to_string();
        let mut replacements = Vec::new();

        // Process all generator outputs first (higher priority)
        for (key, value) in generator_outputs {
            let variable = format!("{}{}{}", self.start_delimiter, key, self.end_delimiter);
            if result.contains(&variable) {
                replacements.push((variable, value.clone()));
            }
        }

        // Then process metadata values (lower priority)
        for (key, value) in metadata {
            let variable = format!("{}{}{}", self.start_delimiter, key, self.end_delimiter);
            if result.contains(&variable) {
                // Only add if not already replaced by a generator output
                if !replacements.iter().any(|(var, _)| var == &variable) {
                    replacements.push((variable, value.clone()));
                }
            }
        }

        // Apply all replacements
        for (var, replacement) in replacements {
            debug!("Replacing template variable: {} -> {}", var, replacement);
            result = result.replace(&var, &replacement);
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
    fn test_variable_substitution() {
        let processor = TemplateVariableProcessor::new();

        let html = r#"<!DOCTYPE html>
        <html>
        <head>
            <title>{{title}}</title>
            {{meta_tags}}
            {{open_graph}}
        </head>
        <body>
            <h1>{{title}}</h1>
            <p>{{description}}</p>
        </body>
        </html>"#;

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "My Page Title".to_string());
        metadata.insert("description".to_string(), "Page description".to_string());

        let mut generator_outputs = HashMap::new();
        generator_outputs.insert(
            "meta_tags".to_string(),
            "<meta name=\"description\" content=\"SEO description\">".to_string(),
        );
        generator_outputs.insert(
            "open_graph".to_string(),
            "<meta property=\"og:title\" content=\"OG Title\">".to_string(),
        );

        let result = processor
            .process(html, &metadata, &generator_outputs, "")
            .unwrap();

        assert!(result.contains("<title>My Page Title</title>"));
        assert!(result.contains("<h1>My Page Title</h1>"));
        assert!(result.contains("<p>Page description</p>"));
        assert!(result.contains("<meta name=\"description\" content=\"SEO description\">"));
        assert!(result.contains("<meta property=\"og:title\" content=\"OG Title\">"));
    }

    #[test]
    fn test_custom_delimiters() {
        let processor = TemplateVariableProcessor::new().with_delimiters("${", "}");

        let html = r#"<div>${title}</div>"#;

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Custom Delimiters".to_string());

        let result = processor
            .process(html, &metadata, &HashMap::new(), "")
            .unwrap();

        assert_eq!(result, "<div>Custom Delimiters</div>");
    }

    #[test]
    fn test_priority_generator_over_metadata() {
        let processor = TemplateVariableProcessor::new();

        let html = "<title>{{title}}</title>";

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Metadata Title".to_string());

        let mut generator_outputs = HashMap::new();
        generator_outputs.insert("title".to_string(), "Generator Title".to_string());

        let result = processor
            .process(html, &metadata, &generator_outputs, "")
            .unwrap();

        // Generator output should take precedence
        assert_eq!(result, "<title>Generator Title</title>");
    }
}
