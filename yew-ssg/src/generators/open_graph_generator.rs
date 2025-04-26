use crate::generator::Generator;
use crate::processors::GeneratorOutputSupport;
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
        key: &str,
        _route: &str,
        _content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        match key {
            // Main output: full OpenGraph tags
            "open_graph" => {
                let mut tags = String::new();

                // Get OG type from metadata or default to "website"
                let og_type = metadata
                    .get("og:type")
                    .cloned()
                    .unwrap_or_else(|| "website".to_string());
                tags.push_str(&format!(
                    "<meta property=\"og:type\" content=\"{}\">\n",
                    og_type
                ));

                // Title - use og:title if available, then title, then empty
                let title = metadata
                    .get("og:title")
                    .or_else(|| metadata.get("title"))
                    .cloned()
                    .unwrap_or_default();
                tags.push_str(&format!(
                    "<meta property=\"og:title\" content=\"{}\">\n",
                    title
                ));

                // Description - use og:description if available, then description, then empty
                let description = metadata
                    .get("og:description")
                    .or_else(|| metadata.get("description"))
                    .cloned()
                    .unwrap_or_default();
                tags.push_str(&format!(
                    "<meta property=\"og:description\" content=\"{}\">\n",
                    description
                ));

                // URL - use og:url if available, then url, then canonical
                let url = metadata
                    .get("og:url")
                    .or_else(|| metadata.get("url"))
                    .or_else(|| metadata.get("canonical"))
                    .cloned()
                    .unwrap_or_default();
                tags.push_str(&format!("<meta property=\"og:url\" content=\"{}\">\n", url));

                // Image - use og:image if available, then default
                let image = metadata
                    .get("og:image")
                    .cloned()
                    .unwrap_or_else(|| self.default_image.clone());
                tags.push_str(&format!(
                    "<meta property=\"og:image\" content=\"{}\">\n",
                    image
                ));

                // Site name - use og:site_name if available, then site_name, then default
                let site_name = metadata
                    .get("og:site_name")
                    .or_else(|| metadata.get("site_name"))
                    .cloned()
                    .unwrap_or_else(|| self.site_name.clone());
                tags.push_str(&format!(
                    "<meta property=\"og:site_name\" content=\"{}\">\n",
                    site_name
                ));

                // Optional locale if available
                if let Some(locale) = metadata.get("og:locale").or_else(|| metadata.get("locale")) {
                    tags.push_str(&format!(
                        "<meta property=\"og:locale\" content=\"{}\">\n",
                        locale
                    ));
                }

                // Optional image dimensions if available
                if let Some(width) = metadata.get("og:image:width") {
                    tags.push_str(&format!(
                        "<meta property=\"og:image:width\" content=\"{}\">\n",
                        width
                    ));
                }

                if let Some(height) = metadata.get("og:image:height") {
                    tags.push_str(&format!(
                        "<meta property=\"og:image:height\" content=\"{}\">\n",
                        height
                    ));
                }

                if let Some(alt) = metadata.get("og:image:alt") {
                    tags.push_str(&format!(
                        "<meta property=\"og:image:alt\" content=\"{}\">\n",
                        alt
                    ));
                }

                Ok(tags)
            }

            // Individual OpenGraph properties with metadata priority
            "og:title" => {
                let title = metadata
                    .get("og:title")
                    .or_else(|| metadata.get("title"))
                    .cloned()
                    .unwrap_or_default();
                Ok(format!(
                    "<meta property=\"og:title\" content=\"{}\">\n",
                    title
                ))
            }

            "og:description" => {
                let description = metadata
                    .get("og:description")
                    .or_else(|| metadata.get("description"))
                    .cloned()
                    .unwrap_or_default();
                Ok(format!(
                    "<meta property=\"og:description\" content=\"{}\">\n",
                    description
                ))
            }

            "og:url" => {
                let url = metadata
                    .get("og:url")
                    .or_else(|| metadata.get("url"))
                    .or_else(|| metadata.get("canonical"))
                    .cloned()
                    .unwrap_or_default();
                Ok(format!("<meta property=\"og:url\" content=\"{}\">\n", url))
            }

            "og:image" => {
                let image = metadata
                    .get("og:image")
                    .cloned()
                    .unwrap_or_else(|| self.default_image.clone());
                Ok(format!(
                    "<meta property=\"og:image\" content=\"{}\">\n",
                    image
                ))
            }

            "og:site_name" => {
                let site_name = metadata
                    .get("og:site_name")
                    .or_else(|| metadata.get("site_name"))
                    .cloned()
                    .unwrap_or_else(|| self.site_name.clone());
                Ok(format!(
                    "<meta property=\"og:site_name\" content=\"{}\">\n",
                    site_name
                ))
            }

            // Unsupported key
            _ => Err(format!("OpenGraphGenerator does not support key: {}", key).into()),
        }
    }

    fn clone_box(&self) -> Box<dyn Generator> {
        Box::new(self.clone())
    }
}

impl GeneratorOutputSupport for OpenGraphGenerator {
    fn supported_outputs(&self) -> Vec<&'static str> {
        vec![
            "open_graph",
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
            .generate(
                "open_graph",
                "/test-route",
                "<div>Test content</div>",
                &HashMap::new(),
            )
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
            .generate(
                "open_graph",
                "/test-route",
                "<div>Test content</div>",
                &metadata,
            )
            .unwrap();

        assert!(result.contains("<meta property=\"og:title\" content=\"Custom Title\">"));
        assert!(
            result.contains("<meta property=\"og:description\" content=\"Custom description\">")
        );
        assert!(result.contains("<meta property=\"og:url\" content=\"https://example.com/test\">"));
        assert!(result
            .contains("<meta property=\"og:image\" content=\"https://example.com/custom.jpg\">"));
    }

    #[test]
    fn test_metadata_priority() {
        let generator = OpenGraphGenerator {
            site_name: "Default Site".to_string(),
            default_image: "https://example.com/default.jpg".to_string(),
        };

        // Test with both direct and og:-prefixed metadata
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Regular Title".to_string());
        metadata.insert("og:title".to_string(), "OG Title".to_string());
        metadata.insert("site_name".to_string(), "Regular Site Name".to_string());
        metadata.insert("og:site_name".to_string(), "OG Site Name".to_string());

        let result = generator
            .generate(
                "open_graph",
                "/test-route",
                "<div>Test content</div>",
                &metadata,
            )
            .unwrap();

        // OG-prefixed values should take priority
        assert!(result.contains("<meta property=\"og:title\" content=\"OG Title\">"));
        assert!(result.contains("<meta property=\"og:site_name\" content=\"OG Site Name\">"));

        // Test individual property generators also respect priority
        let title_result = generator
            .generate(
                "og:title",
                "/test-route",
                "<div>Test content</div>",
                &metadata,
            )
            .unwrap();
        assert!(title_result.contains("<meta property=\"og:title\" content=\"OG Title\">"));

        let site_name_result = generator
            .generate(
                "og:site_name",
                "/test-route",
                "<div>Test content</div>",
                &metadata,
            )
            .unwrap();
        assert!(
            site_name_result.contains("<meta property=\"og:site_name\" content=\"OG Site Name\">")
        );
    }
}
