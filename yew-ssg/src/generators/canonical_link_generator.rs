use crate::generator::Generator;
use crate::processors::GeneratorOutputSupport;
use std::any::Any;
use std::collections::HashMap;
use std::error::Error;

/// Generator for canonical URLs and alternate language links
///
/// This generator produces link tags for:
/// - Canonical URLs (the definitive version of a page)
/// - Alternate language versions of a page (hreflang)
#[derive(Debug, Clone)]
pub struct CanonicalLinkGenerator {
    /// Default domain to use when constructing full URLs if not provided in metadata
    pub default_domain: Option<String>,
}

impl Default for CanonicalLinkGenerator {
    fn default() -> Self {
        Self {
            default_domain: None,
        }
    }
}

impl CanonicalLinkGenerator {
    /// Create a new generator instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new generator with a default domain
    pub fn with_domain(domain: &str) -> Self {
        Self {
            default_domain: Some(domain.to_string()),
        }
    }

    /// Get the base domain from metadata or default
    fn get_domain(&self, metadata: &HashMap<String, String>) -> Option<String> {
        metadata
            .get("domain")
            .cloned()
            .or_else(|| self.default_domain.clone())
    }

    /// Generate a canonical link tag
    fn generate_canonical(&self, metadata: &HashMap<String, String>) -> String {
        // First check if an explicit canonical URL is provided
        if let Some(canonical) = metadata.get("canonical") {
            return format!("<link rel=\"canonical\" href=\"{}\">\n", canonical);
        }

        // If not, try to construct one from domain and path
        if let Some(domain) = self.get_domain(metadata) {
            // Get the path - use "path" from metadata or default to "/"
            let path = metadata
                .get("path")
                .cloned()
                .unwrap_or_else(|| "/".to_string());
            // Construct the full URL, handling trailing slashes properly
            let domain_trimmed = domain.trim_end_matches('/');
            let path_with_slash = if !path.starts_with('/') {
                format!("/{}", path)
            } else {
                path.clone()
            };

            let url = if path_with_slash == "/" {
                format!("{}/", domain_trimmed)
            } else {
                format!("{}{}", domain_trimmed, path_with_slash)
            };

            return format!("<link rel=\"canonical\" href=\"{}\">\n", url);
        }

        // If we can't construct a canonical URL, return empty string
        "".to_string()
    }

    /// Generate alternate language links
    fn generate_alternate_links(&self, metadata: &HashMap<String, String>) -> String {
        let mut result = String::new();

        // Check if alternate_languages is defined in metadata
        if let Some(langs) = metadata.get("alternate_languages") {
            // The languages are expected to be a comma-separated list
            let languages: Vec<&str> = langs.split(',').map(|s| s.trim()).collect();

            // Get the domain
            let domain = if let Some(domain) = self.get_domain(metadata) {
                domain.trim_end_matches('/').to_string()
            } else {
                return "".to_string(); // Can't generate alternate links without a domain
            };

            // Get the current path from metadata (default to / if not present)
            let current_path = metadata
                .get("path")
                .cloned()
                .unwrap_or_else(|| "/".to_string());
            let path_with_slash = if !current_path.starts_with('/') {
                format!("/{}", current_path)
            } else {
                current_path
            };

            // For each language, generate an alternate link
            for lang in languages {
                // Check if we have a custom URL for this language
                let lang_key = format!("alternate_url_{}", lang);
                let url = if let Some(lang_url) = metadata.get(&lang_key) {
                    // Use the custom URL as is
                    lang_url.clone()
                } else {
                    // Otherwise, construct the URL with the language parameter
                    // Use the current path (not just root)
                    format!("{}{}?lang={}", domain, path_with_slash, lang)
                };

                result.push_str(&format!(
                    "<link rel=\"alternate\" hreflang=\"{}\" href=\"{}\">\n",
                    lang, url
                ));
            }
        }

        result
    }

    /// Get just the canonical URL value (without HTML tags)
    fn get_canonical_url(&self, metadata: &HashMap<String, String>) -> String {
        if let Some(canonical) = metadata.get("canonical") {
            canonical.clone()
        } else if let (Some(domain), Some(path)) = (self.get_domain(metadata), metadata.get("path"))
        {
            if path == "/" {
                domain.clone()
            } else {
                format!("{}{}", domain.trim_end_matches('/'), path)
            }
        } else {
            "".to_string()
        }
    }
}

impl Generator for CanonicalLinkGenerator {
    fn name(&self) -> &'static str {
        "canonical_links"
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
            // Main output: all link tags (canonical + alternates)
            "canonical_links" => {
                let mut result = String::new();

                // Add canonical link
                let canonical = self.generate_canonical(metadata);
                if !canonical.is_empty() {
                    result.push_str(&canonical);
                }

                // Add alternate language links
                let alternates = self.generate_alternate_links(metadata);
                if !alternates.is_empty() {
                    result.push_str(&alternates);
                }

                Ok(result)
            }

            // Just the canonical link tag
            "canonical" => Ok(self.generate_canonical(metadata)),

            // Just the canonical URL (no HTML)
            "canonical_url" => Ok(self.get_canonical_url(metadata)),

            // Just the alternate language links
            "alternate_links" => Ok(self.generate_alternate_links(metadata)),

            // Unsupported key
            _ => Err(format!("CanonicalLinkGenerator does not support key: {}", key).into()),
        }
    }

    fn clone_box(&self) -> Box<dyn Generator> {
        Box::new(self.clone())
    }
}

impl GeneratorOutputSupport for CanonicalLinkGenerator {
    fn supported_outputs(&self) -> Vec<&'static str> {
        vec![
            "canonical_links",
            "canonical",
            "canonical_url",
            "alternate_links",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_canonical_link_generator() {
        // Test with explicit canonical URL
        let generator = CanonicalLinkGenerator::new();
        let mut metadata = HashMap::new();
        metadata.insert(
            "canonical".to_string(),
            "https://example.com/page".to_string(),
        );

        let result = generator.generate("canonical", "", "", &metadata).unwrap();
        assert_eq!(
            result,
            "<link rel=\"canonical\" href=\"https://example.com/page\">\n"
        );

        // Test URL-only output
        let url_result = generator
            .generate("canonical_url", "", "", &metadata)
            .unwrap();
        assert_eq!(url_result, "https://example.com/page");
    }

    #[test]
    fn test_alternate_links() {
        let generator = CanonicalLinkGenerator::with_domain("https://example.com");
        let mut metadata = HashMap::new();
        metadata.insert(
            "canonical".to_string(),
            "https://example.com/page".to_string(),
        );
        metadata.insert("alternate_languages".to_string(), "en,es,fr".to_string());

        let result = generator
            .generate("alternate_links", "", "", &metadata)
            .unwrap();

        assert!(result.contains(
            "<link rel=\"alternate\" hreflang=\"en\" href=\"https://example.com/page/?lang=en\">"
        ));
        assert!(result.contains(
            "<link rel=\"alternate\" hreflang=\"es\" href=\"https://example.com/page/?lang=es\">"
        ));
        assert!(result.contains(
            "<link rel=\"alternate\" hreflang=\"fr\" href=\"https://example.com/page/?lang=fr\">"
        ));
    }

    #[test]
    fn test_custom_alternate_urls() {
        let generator = CanonicalLinkGenerator::with_domain("https://example.com");
        let mut metadata = HashMap::new();
        metadata.insert(
            "canonical".to_string(),
            "https://example.com/page".to_string(),
        );
        metadata.insert("alternate_languages".to_string(), "en,es".to_string());
        metadata.insert(
            "alternate_url_es".to_string(),
            "https://example.es/pagina".to_string(),
        );

        let result = generator
            .generate("alternate_links", "", "", &metadata)
            .unwrap();

        assert!(result.contains(
            "<link rel=\"alternate\" hreflang=\"en\" href=\"https://example.com/page/?lang=en\">"
        ));
        assert!(result.contains(
            "<link rel=\"alternate\" hreflang=\"es\" href=\"https://example.es/pagina\">"
        ));
    }

    #[test]
    fn test_combined_output() {
        let generator = CanonicalLinkGenerator::with_domain("https://example.com");
        let mut metadata = HashMap::new();
        metadata.insert(
            "canonical".to_string(),
            "https://example.com/page".to_string(),
        );
        metadata.insert("alternate_languages".to_string(), "en,es".to_string());

        let result = generator
            .generate("canonical_links", "", "", &metadata)
            .unwrap();

        // Should contain both canonical and alternate links
        assert!(result.contains("<link rel=\"canonical\" href=\"https://example.com/page\">"));
        assert!(result.contains(
            "<link rel=\"alternate\" hreflang=\"en\" href=\"https://example.com/page/?lang=en\">"
        ));
        assert!(result.contains(
            "<link rel=\"alternate\" hreflang=\"es\" href=\"https://example.com/page/?lang=es\">"
        ));
    }

    #[test]
    fn test_constructed_canonical() {
        let generator = CanonicalLinkGenerator::with_domain("https://example.com");
        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), "/constructed-page".to_string());

        let result = generator.generate("canonical", "", "", &metadata).unwrap();
        assert_eq!(
            result,
            "<link rel=\"canonical\" href=\"https://example.com/constructed-page\">\n"
        );
    }
}
