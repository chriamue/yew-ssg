use crate::generator::Generator;
use crate::processors::GeneratorOutputSupport;
use std::any::Any;
use std::collections::HashMap;
use std::error::Error;
use url::Url;

#[derive(Debug, Clone)]
pub struct OpenGraphGenerator {
    pub site_name: String,
    pub default_image: String,
}

impl OpenGraphGenerator {
    fn get_og_url(&self, metadata: &HashMap<String, String>) -> String {
        if let Some(url) = metadata.get("og:url") {
            url.clone()
        } else if let Some(canonical) = metadata.get("canonical") {
            canonical.clone()
        } else if let Some(url) = metadata.get("url") {
            url.clone()
        } else if let (Some(domain), Some(path)) = (metadata.get("domain"), metadata.get("path")) {
            if let Ok(base) = Url::parse(domain) {
                if let Ok(joined) = base.join(path) {
                    return joined.to_string();
                }
            }
            // fallback to manual if url crate fails
            let domain = domain.trim_end_matches('/');
            let path = if path.starts_with('/') {
                path.clone()
            } else {
                format!("/{}", path)
            };
            format!("{}{}", domain, path)
        } else {
            "".to_string()
        }
    }

    fn get_og_locale(&self, metadata: &HashMap<String, String>) -> String {
        // Try explicit metadata first
        if let Some(lang) = metadata.get("lang").or_else(|| metadata.get("language")) {
            return lang.clone();
        }

        // Try to detect from path if available
        if let Some(path) = metadata.get("path") {
            // Extract language code from URL path (e.g., "/de/about" → "de")
            let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

            if !path_segments.is_empty() {
                // Check if first segment looks like a language code (2-3 chars)
                let first_segment = path_segments[0];
                if first_segment.len() >= 2 && first_segment.len() <= 3 {
                    // Common language codes
                    if ["en", "de", "es", "fr", "it", "ja", "zh", "ru"].contains(&first_segment) {
                        return first_segment.to_string();
                    }
                }
            }
        }

        // Default fallback
        "en".to_string()
    }

    fn get_og_locale_alternates(
        &self,
        metadata: &HashMap<String, String>,
        current: &str,
    ) -> Vec<String> {
        if let Some(alts) = metadata.get("alternate_languages") {
            alts.split(',')
                .map(|s| s.trim().to_string())
                .filter(|lang| !lang.is_empty() && lang != current)
                .collect()
        } else {
            vec![]
        }
    }
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
            "open_graph" => {
                let mut tags = String::new();

                // og:type
                let og_type = metadata
                    .get("og:type")
                    .cloned()
                    .unwrap_or_else(|| "website".to_string());
                tags.push_str(&format!(
                    "<meta property=\"og:type\" content=\"{}\">\n",
                    og_type
                ));

                // og:title
                let title = metadata
                    .get("og:title")
                    .or_else(|| metadata.get("title"))
                    .cloned()
                    .unwrap_or_default();
                tags.push_str(&format!(
                    "<meta property=\"og:title\" content=\"{}\">\n",
                    title
                ));

                // og:description
                let description = metadata
                    .get("og:description")
                    .or_else(|| metadata.get("description"))
                    .cloned()
                    .unwrap_or_default();
                tags.push_str(&format!(
                    "<meta property=\"og:description\" content=\"{}\">\n",
                    description
                ));

                // og:url
                let url = self.get_og_url(metadata);
                tags.push_str(&format!("<meta property=\"og:url\" content=\"{}\">\n", url));

                // og:image
                let image = metadata
                    .get("og:image")
                    .cloned()
                    .unwrap_or_else(|| self.default_image.clone());
                tags.push_str(&format!(
                    "<meta property=\"og:image\" content=\"{}\">\n",
                    image
                ));

                // og:site_name
                let site_name = metadata
                    .get("og:site_name")
                    .or_else(|| metadata.get("site_name"))
                    .cloned()
                    .unwrap_or_else(|| self.site_name.clone());
                tags.push_str(&format!(
                    "<meta property=\"og:site_name\" content=\"{}\">\n",
                    site_name
                ));

                // og:locale (language code, not locale)
                let og_locale = self.get_og_locale(metadata);
                tags.push_str(&format!(
                    "<meta property=\"og:locale\" content=\"{}\">\n",
                    og_locale
                ));

                // og:locale:alternate
                for alt in self.get_og_locale_alternates(metadata, &og_locale) {
                    tags.push_str(&format!(
                        "<meta property=\"og:locale:alternate\" content=\"{}\">\n",
                        alt
                    ));
                }

                // Optional image dimensions
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
                let url = self.get_og_url(metadata);
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

            "og:locale" => {
                let og_locale = self.get_og_locale(metadata);
                Ok(format!(
                    "<meta property=\"og:locale\" content=\"{}\">\n",
                    og_locale
                ))
            }

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
            "og:locale",
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

    #[test]
    fn test_open_graph_generator_url_and_locale() {
        let generator = OpenGraphGenerator {
            site_name: "Test Site".to_string(),
            default_image: "https://example.com/default.jpg".to_string(),
        };

        // Test with canonical and lang
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Custom Title".to_string());
        metadata.insert("description".to_string(), "Custom description".to_string());
        metadata.insert(
            "canonical".to_string(),
            "https://example.com/404".to_string(),
        );
        metadata.insert("lang".to_string(), "de".to_string());
        metadata.insert("alternate_languages".to_string(), "en,de,es".to_string());

        let result = generator
            .generate("open_graph", "/404", "<div>Test content</div>", &metadata)
            .unwrap();

        assert!(result.contains(r#"<meta property="og:url" content="https://example.com/404">"#));
        assert!(result.contains(r#"<meta property="og:locale" content="de">"#));
        assert!(result.contains(r#"<meta property="og:locale:alternate" content="en">"#));
        assert!(result.contains(r#"<meta property="og:locale:alternate" content="es">"#));
        // Should not contain "de" as alternate
        let alternates: Vec<_> = result
            .lines()
            .filter(|l| l.contains("og:locale:alternate"))
            .collect();
        assert!(!alternates.iter().any(|l| l.contains(r#"content="de""#)));
    }
}
