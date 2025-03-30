use crate::generator::Generator;
pub use crate::processors::{AttributeSupport, TemplateVariableSupport};
use std::any::Any;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct OpenGraphGenerator {
    pub site_name: String,
    pub default_image: String,
}

impl Generator for OpenGraphGenerator {
    fn name(&self) -> &'static str {
        "open_graph"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn generate(
        &self,
        _route: &str,
        _content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        let mut tags = String::new();

        // Basic OG tags
        tags.push_str("<meta property=\"og:type\" content=\"website\">\n");

        let title = metadata.get("title").cloned().unwrap_or_default();
        tags.push_str(&format!(
            "<meta property=\"og:title\" content=\"{}\">\n",
            title
        ));

        let description = metadata.get("description").cloned().unwrap_or_default();
        tags.push_str(&format!(
            "<meta property=\"og:description\" content=\"{}\">\n",
            description
        ));

        let url = metadata.get("url").cloned().unwrap_or_default();
        tags.push_str(&format!("<meta property=\"og:url\" content=\"{}\">\n", url));

        let image = metadata
            .get("og:image")
            .cloned()
            .unwrap_or_else(|| self.default_image.clone());
        tags.push_str(&format!(
            "<meta property=\"og:image\" content=\"{}\">\n",
            image
        ));

        tags.push_str(&format!(
            "<meta property=\"og:site_name\" content=\"{}\">\n",
            self.site_name
        ));

        Ok(tags)
    }

    fn clone_box(&self) -> Box<dyn Generator> {
        Box::new(self.clone())
    }
}

impl AttributeSupport for OpenGraphGenerator {
    fn attributes(&self) -> Vec<&'static str> {
        vec!["open_graph"]
    }
}

impl TemplateVariableSupport for OpenGraphGenerator {
    fn template_variables(&self) -> Vec<&'static str> {
        vec![
            "og:title",
            "og:description",
            "og:url",
            "og:image",
            "og:site_name",
        ]
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::Generator;
    use crate::generators::OpenGraphGenerator;
    use std::collections::HashMap;

    #[test]
    fn test_open_graph_generator() {
        let generator = OpenGraphGenerator {
            site_name: "Test Site".to_string(),
            default_image: "https://example.com/default.jpg".to_string(),
        };

        // Test with empty metadata
        let result = generator
            .generate("/test-route", "<div>Test content</div>", &HashMap::new())
            .unwrap();

        assert!(result.contains("<meta property=\"og:type\" content=\"website\">"));
        assert!(result.contains("<meta property=\"og:site_name\" content=\"Test Site\">"));
        assert!(result
            .contains("<meta property=\"og:image\" content=\"https://example.com/default.jpg\">"));

        // Test with custom metadata
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Custom Title".to_string());
        metadata.insert("description".to_string(), "Custom description".to_string());
        metadata.insert("url".to_string(), "https://example.com/test".to_string());
        metadata.insert(
            "og:image".to_string(),
            "https://example.com/custom.jpg".to_string(),
        );

        let result = generator
            .generate("/test-route", "<div>Test content</div>", &metadata)
            .unwrap();

        assert!(result.contains("<meta property=\"og:title\" content=\"Custom Title\">"));
        assert!(
            result.contains("<meta property=\"og:description\" content=\"Custom description\">")
        );
        assert!(result.contains("<meta property=\"og:url\" content=\"https://example.com/test\">"));
        assert!(result
            .contains("<meta property=\"og:image\" content=\"https://example.com/custom.jpg\">"));
    }
}
