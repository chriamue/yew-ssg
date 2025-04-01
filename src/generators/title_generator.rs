use crate::generator::Generator;
use crate::processors::GeneratorOutputSupport;
use std::any::Any;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct TitleGenerator;

impl Generator for TitleGenerator {
    fn name(&self) -> &'static str {
        "title"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn generate(
        &self,
        key: &str,
        _route: &str,
        _content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        match key {
            // Main output: full title tag
            "title" => {
                let title = metadata.get("title").cloned().unwrap_or_default();
                Ok(format!("<title>{}</title>", title))
            }

            // Just the title text without HTML tags
            "title_text" => {
                let title = metadata.get("title").cloned().unwrap_or_default();
                Ok(title)
            }

            // Unsupported key
            _ => Err(format!("TitleGenerator does not support key: {}", key).into()),
        }
    }

    fn clone_box(&self) -> Box<dyn Generator> {
        Box::new(self.clone())
    }
}

impl GeneratorOutputSupport for TitleGenerator {
    fn supported_outputs(&self) -> Vec<&'static str> {
        vec!["title", "title_text"]
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::Generator;
    use crate::generators::TitleGenerator;
    use std::collections::HashMap;

    #[test]
    fn test_title_generator() {
        let generator = TitleGenerator;

        // Test with empty metadata (should return empty title)
        let result = generator
            .generate(
                "title",
                "/test-route",
                "<div>Test content</div>",
                &HashMap::new(),
            )
            .unwrap();

        assert_eq!(result, "<title></title>");

        // Test with title in metadata
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "My Page Title".to_string());

        let result = generator
            .generate("title", "/test-route", "<div>Test content</div>", &metadata)
            .unwrap();

        assert_eq!(result, "<title>My Page Title</title>");
    }

    #[test]
    fn test_title_text_only() {
        let generator = TitleGenerator;

        // Test with empty metadata
        let result = generator
            .generate(
                "title_text",
                "/test-route",
                "<div>Test content</div>",
                &HashMap::new(),
            )
            .unwrap();

        assert_eq!(result, "");

        // Test with title in metadata
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "My Page Title".to_string());

        let result = generator
            .generate(
                "title_text",
                "/test-route",
                "<div>Test content</div>",
                &metadata,
            )
            .unwrap();

        assert_eq!(result, "My Page Title");
    }

    #[test]
    fn test_title_with_special_characters() {
        let generator = TitleGenerator;

        // Test with title containing HTML special characters
        let mut metadata = HashMap::new();
        metadata.insert(
            "title".to_string(),
            "Title with <script> & other characters".to_string(),
        );

        let result = generator
            .generate("title", "/test-route", "<div>Test content</div>", &metadata)
            .unwrap();

        assert_eq!(
            result,
            "<title>Title with <script> & other characters</title>"
        );
    }

    #[test]
    fn test_title_with_long_text() {
        let generator = TitleGenerator;

        // Test with very long title (SEO best practice is to keep titles under 60-70 characters)
        let mut metadata = HashMap::new();
        metadata.insert(
            "title".to_string(),
            "This is an extremely long title that exceeds recommended SEO guidelines for title length and might get truncated in search results".to_string()
        );

        let result = generator
            .generate("title", "/test-route", "<div>Test content</div>", &metadata)
            .unwrap();

        assert!(result.contains("This is an extremely long title"));
    }

    #[test]
    fn test_unsupported_key() {
        let generator = TitleGenerator;

        // Test with an unsupported key
        let result = generator.generate(
            "invalid_key",
            "/test-route",
            "<div>Test content</div>",
            &HashMap::new(),
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("does not support key"));
    }
}
