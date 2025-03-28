use crate::generator::Generator;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct TitleGenerator;

impl Generator for TitleGenerator {
    fn name(&self) -> &'static str {
        "title"
    }

    fn generate(
        &self,
        _route: &str,
        _content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        let title = metadata.get("title").cloned().unwrap_or_default();
        Ok(format!("<title>{}</title>", title))
    }

    fn clone_box(&self) -> Box<dyn Generator> {
        Box::new(self.clone())
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
            .generate("/test-route", "<div>Test content</div>", &HashMap::new())
            .unwrap();

        assert_eq!(result, "<title></title>");

        // Test with title in metadata
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "My Page Title".to_string());

        let result = generator
            .generate("/test-route", "<div>Test content</div>", &metadata)
            .unwrap();

        assert_eq!(result, "<title>My Page Title</title>");
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
            .generate("/test-route", "<div>Test content</div>", &metadata)
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
            .generate("/test-route", "<div>Test content</div>", &metadata)
            .unwrap();

        assert!(result.contains("This is an extremely long title"));
    }
}
