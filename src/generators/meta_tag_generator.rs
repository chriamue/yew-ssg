use crate::generator::Generator;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct MetaTagGenerator {
    default_description: String,
    default_keywords: Vec<String>,
}

impl Generator for MetaTagGenerator {
    fn name(&self) -> &'static str {
        "meta_tags"
    }

    fn generate(
        &self,
        _route: &str,
        _content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        let mut tags = String::new();

        // Description meta tag
        let description = metadata
            .get("description")
            .cloned()
            .unwrap_or_else(|| self.default_description.clone());
        tags.push_str(&format!(
            "<meta name=\"description\" content=\"{}\">\n",
            description
        ));

        // Keywords meta tag
        let keywords = metadata
            .get("keywords")
            .cloned()
            .unwrap_or_else(|| self.default_keywords.join(", "));
        tags.push_str(&format!(
            "<meta name=\"keywords\" content=\"{}\">\n",
            keywords
        ));

        // Canonical URL
        if let Some(canonical) = metadata.get("canonical") {
            tags.push_str(&format!(
                "<link rel=\"canonical\" href=\"{}\">\n",
                canonical
            ));
        }

        Ok(tags)
    }

    fn clone_box(&self) -> Box<dyn Generator> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::Generator;
    use crate::generators::MetaTagGenerator;
    use std::collections::HashMap;

    #[test]
    fn test_meta_tag_generator() {
        let generator = MetaTagGenerator {
            default_description: "Default description".to_string(),
            default_keywords: vec!["rust".to_string(), "yew".to_string(), "ssg".to_string()],
        };

        // Test with empty metadata
        let result = generator
            .generate("/test-route", "<div>Test content</div>", &HashMap::new())
            .unwrap();

        assert!(result.contains("<meta name=\"description\" content=\"Default description\">"));
        assert!(result.contains("<meta name=\"keywords\" content=\"rust, yew, ssg\">"));

        // Test with custom metadata
        let mut metadata = HashMap::new();
        metadata.insert("description".to_string(), "Custom description".to_string());
        metadata.insert("keywords".to_string(), "custom, keywords".to_string());
        metadata.insert(
            "canonical".to_string(),
            "https://example.com/test".to_string(),
        );

        let result = generator
            .generate("/test-route", "<div>Test content</div>", &metadata)
            .unwrap();

        assert!(result.contains("<meta name=\"description\" content=\"Custom description\">"));
        assert!(result.contains("<meta name=\"keywords\" content=\"custom, keywords\">"));
        assert!(result.contains("<link rel=\"canonical\" href=\"https://example.com/test\">"));
    }
}
