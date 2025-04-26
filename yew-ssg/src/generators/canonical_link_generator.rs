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

    /// Get the base path without the language prefix for alternate links
    fn get_base_path(&self, path: &str, current_lang: &str) -> String {
        // Check if path starts with the current language prefix
        let lang_prefix = format!("/{}/", current_lang);
        let lang_prefix_end = format!("/{}", current_lang);

        if path.starts_with(&lang_prefix) {
            // Remove the language prefix
            let base_path = path.replacen(&lang_prefix, "/", 1);
            base_path
        } else if path == &lang_prefix_end {
            // Handle case where path is just "/lang"
            "/".to_string()
        } else if path.starts_with(&lang_prefix_end) && path.len() > lang_prefix_end.len() {
            // Handle case like "/de" but we need to keep the rest
            path.replacen(&lang_prefix_end, "", 1)
        } else {
            // No language prefix, return the path as is
            path.to_string()
        }
    }

    /// Generate alternate language links
    fn generate_alternate_links(&self, metadata: &HashMap<String, String>) -> String {
        let mut result = String::new();

        // Check if alternate_languages is defined in metadata
        if let Some(langs) = metadata.get("alternate_languages") {
            // The languages are expected to be a comma-separated list
            let languages: Vec<&str> = langs
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            if languages.is_empty() {
                return result;
            }

            // Get the domain
            let domain = match self.get_domain(metadata) {
                Some(d) => d,
                None => return result, // Can't generate alternate links without a domain
            };

            // Try to parse domain for validation
            let base_url = match url::Url::parse(&domain) {
                Ok(url) => url,
                Err(_) => {
                    // Try adding scheme if missing
                    match url::Url::parse(&format!("https://{}", domain)) {
                        Ok(url) => url,
                        Err(_) => return result, // Invalid domain
                    }
                }
            };

            // Get the current path from metadata (default to / if not present)
            let current_path = metadata
                .get("path")
                .cloned()
                .unwrap_or_else(|| "/".to_string());

            // Get current language to avoid duplicating it in alternates
            let current_lang = metadata
                .get("lang")
                .cloned()
                .unwrap_or_else(|| "en".to_string());

            // First, determine if we need an x-default link (typically points to English or root)
            let default_lang = if languages.contains(&"en") {
                "en"
            } else {
                languages[0] // Use first language as default if no English
            };

            // Check if original path has a trailing slash
            let has_trailing_slash = current_path.ends_with('/') && current_path != "/";

            // Get base path without language prefix
            let base_path = self.get_base_path(&current_path, &current_lang);

            // Clean base path (no trailing or leading slashes)
            let clean_base_path = base_path.trim_matches('/');

            // Generate alternate links for each language
            for lang in &languages {
                // Skip if it's the current language (no need for self-reference)
                if *lang == current_lang {
                    continue;
                }

                // Check if we have a custom URL for this language
                let lang_key = format!("alternate_url_{}", lang);

                if let Some(lang_url) = metadata.get(&lang_key) {
                    // Use the custom URL as provided
                    result.push_str(&format!(
                        "<link rel=\"alternate\" hreflang=\"{}\" href=\"{}\">\n",
                        lang, lang_url
                    ));
                } else {
                    // Construct the language-specific path
                    let lang_path = if *lang == "en" {
                        // English version typically at root
                        if clean_base_path.is_empty() {
                            "/".to_string()
                        } else if has_trailing_slash {
                            format!("/{}/", clean_base_path)
                        } else {
                            format!("/{}", clean_base_path)
                        }
                    } else {
                        // Other languages with prefix
                        if clean_base_path.is_empty() {
                            format!("/{}/", lang)
                        } else if has_trailing_slash {
                            format!("/{}/{}/", lang, clean_base_path)
                        } else {
                            format!("/{}/{}", lang, clean_base_path)
                        }
                    };

                    // Use URL joining for proper path construction
                    if let Ok(full_url) = base_url.join(&lang_path) {
                        result.push_str(&format!(
                            "<link rel=\"alternate\" hreflang=\"{}\" href=\"{}\">\n",
                            lang, full_url
                        ));
                    } else {
                        // Fallback to manual joining
                        let domain_str = domain.trim_end_matches('/');

                        result.push_str(&format!(
                            "<link rel=\"alternate\" hreflang=\"{}\" href=\"{}{}\">\n",
                            lang, domain_str, lang_path
                        ));
                    }
                }
            }

            // Add x-default link (typically points to default language content)
            let default_key = format!("alternate_url_{}", default_lang);
            let x_default_url = if let Some(url) = metadata.get("alternate_url_x_default") {
                // Use explicit x-default URL if provided
                url.clone()
            } else if let Some(default_url) = metadata.get(&default_key) {
                // Use default language URL
                default_url.clone()
            } else {
                // Construct default URL using the same trailing slash logic
                let default_path = if default_lang == "en" {
                    if clean_base_path.is_empty() {
                        "/".to_string()
                    } else if has_trailing_slash {
                        format!("/{}/", clean_base_path)
                    } else {
                        format!("/{}", clean_base_path)
                    }
                } else {
                    if clean_base_path.is_empty() {
                        format!("/{}/", default_lang)
                    } else if has_trailing_slash {
                        format!("/{}/{}/", default_lang, clean_base_path)
                    } else {
                        format!("/{}/{}", default_lang, clean_base_path)
                    }
                };

                // Try URL joining
                if let Ok(url) = base_url.join(&default_path) {
                    url.to_string()
                } else {
                    format!("{}{}", domain.trim_end_matches('/'), default_path)
                }
            };

            result.push_str(&format!(
                "<link rel=\"alternate\" hreflang=\"x-default\" href=\"{}\">\n",
                x_default_url
            ));
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
        metadata.insert("path".to_string(), "/page".to_string());
        metadata.insert("alternate_languages".to_string(), "en,es,fr".to_string());
        // Set current language to something other than English to see the English link
        metadata.insert("lang".to_string(), "de".to_string());

        let result = generator
            .generate("alternate_links", "", "", &metadata)
            .unwrap();

        // Now English should be included since current lang is German
        assert!(result
            .contains("<link rel=\"alternate\" hreflang=\"en\" href=\"https://example.com/page"));
        // Other languages should use a language path prefix
        assert!(result.contains(
            "<link rel=\"alternate\" hreflang=\"es\" href=\"https://example.com/es/page"
        ));
        assert!(result.contains(
            "<link rel=\"alternate\" hreflang=\"fr\" href=\"https://example.com/fr/page"
        ));
        // Should not contain current language
        assert!(!result.contains("hreflang=\"de\""));
    }

    #[test]
    fn test_custom_alternate_urls() {
        let generator = CanonicalLinkGenerator::with_domain("https://example.com");
        let mut metadata = HashMap::new();
        metadata.insert(
            "canonical".to_string(),
            "https://example.com/page".to_string(),
        );
        metadata.insert("path".to_string(), "/page".to_string());
        metadata.insert("alternate_languages".to_string(), "en,es".to_string());
        metadata.insert(
            "alternate_url_es".to_string(),
            "https://example.es/pagina".to_string(),
        );
        // Set current language to something other than English
        metadata.insert("lang".to_string(), "fr".to_string());

        let result = generator
            .generate("alternate_links", "", "", &metadata)
            .unwrap();

        // English URL should be present since current lang is French
        assert!(result
            .contains("<link rel=\"alternate\" hreflang=\"en\" href=\"https://example.com/page"));
        // Spanish URL should use the custom URL provided
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
        metadata.insert("path".to_string(), "/page".to_string());
        metadata.insert("alternate_languages".to_string(), "en,es".to_string());
        // Set current language to something other than English
        metadata.insert("lang".to_string(), "fr".to_string());

        let result = generator
            .generate("canonical_links", "", "", &metadata)
            .unwrap();

        // Should contain both canonical and alternate links
        assert!(result.contains("<link rel=\"canonical\" href=\"https://example.com/page\">"));
        assert!(result
            .contains("<link rel=\"alternate\" hreflang=\"en\" href=\"https://example.com/page"));
        assert!(result.contains(
            "<link rel=\"alternate\" hreflang=\"es\" href=\"https://example.com/es/page"
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

    #[test]
    fn test_alternate_links_complete() {
        // This test covers all aspects of alternate link generation:
        // - Basic alternate links
        // - Custom alternate URLs
        // - Handling current language
        // - x-default link
        // - Various path formats

        let generator = CanonicalLinkGenerator::with_domain("https://example.com");

        // Test 1: Basic case with English as current language
        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), "/products".to_string());
        metadata.insert("lang".to_string(), "en".to_string());
        metadata.insert("alternate_languages".to_string(), "en,es,fr,de".to_string());

        let result = generator.generate_alternate_links(&metadata);

        // Should contain links for es, fr, de but not en (current language)
        assert!(!result.contains("hreflang=\"en\""));
        assert!(result.contains("hreflang=\"es\" href=\"https://example.com/es/products\""));
        assert!(result.contains("hreflang=\"fr\" href=\"https://example.com/fr/products\""));
        assert!(result.contains("hreflang=\"de\" href=\"https://example.com/de/products\""));
        // Should have x-default pointing to English
        assert!(result.contains("hreflang=\"x-default\" href=\"https://example.com/products\""));

        // Test 2: Non-English current language
        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), "/produkte".to_string());
        metadata.insert("lang".to_string(), "de".to_string());
        metadata.insert("alternate_languages".to_string(), "en,es,fr,de".to_string());

        let result = generator.generate_alternate_links(&metadata);

        // Should contain links for en, es, fr but not de (current)
        assert!(result.contains("hreflang=\"en\" href=\"https://example.com/produkte"));
        assert!(result.contains("hreflang=\"es\" href=\"https://example.com/es/produkte"));
        assert!(result.contains("hreflang=\"fr\" href=\"https://example.com/fr/produkte"));
        assert!(!result.contains("hreflang=\"de\""));
        // x-default should still point to English
        assert!(result.contains("hreflang=\"x-default\" href=\"https://example.com/produkte\""));

        // Test 3: Custom URLs for specific languages
        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), "/about".to_string());
        metadata.insert("lang".to_string(), "en".to_string());
        metadata.insert("alternate_languages".to_string(), "en,es,fr".to_string());
        metadata.insert(
            "alternate_url_es".to_string(),
            "https://example.es/sobre-nosotros".to_string(),
        );
        metadata.insert(
            "alternate_url_x_default".to_string(),
            "https://example.com/about-us".to_string(),
        );

        let result = generator.generate_alternate_links(&metadata);

        // Should use custom URL for Spanish
        assert!(result.contains("hreflang=\"es\" href=\"https://example.es/sobre-nosotros\""));
        // Should use standard format for French
        assert!(result.contains("hreflang=\"fr\" href=\"https://example.com/fr/about\""));
        // Should use custom x-default
        assert!(result.contains("hreflang=\"x-default\" href=\"https://example.com/about-us\""));

        // Test 4: Root path handling
        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), "/".to_string());
        metadata.insert("lang".to_string(), "fr".to_string());
        metadata.insert("alternate_languages".to_string(), "en,fr,de".to_string());

        let result = generator.generate_alternate_links(&metadata);

        // Root paths should be handled correctly
        assert!(result.contains("hreflang=\"en\" href=\"https://example.com/\""));
        assert!(result.contains("hreflang=\"de\" href=\"https://example.com/de/\""));

        // Test 5: No alternate languages
        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), "/page".to_string());

        let result = generator.generate_alternate_links(&metadata);
        assert_eq!(result, ""); // Should return empty string

        // Test 6: Empty alternate languages
        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), "/page".to_string());
        metadata.insert("alternate_languages".to_string(), "".to_string());

        let result = generator.generate_alternate_links(&metadata);
        assert_eq!(result, ""); // Should return empty string
    }

    #[test]
    fn test_language_path_handling() {
        let generator = CanonicalLinkGenerator::with_domain("https://example.com");

        // Test with path that already has language prefix
        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), "/de/404".to_string()); // No trailing slash
        metadata.insert("lang".to_string(), "de".to_string());
        metadata.insert("alternate_languages".to_string(), "en,de,es".to_string());

        let result = generator.generate_alternate_links(&metadata);

        println!("Generated alternate links: {}", result);

        // Should not duplicate language in path for es
        assert!(result.contains("hreflang=\"es\" href=\"https://example.com/es/404\""));
        // Should have en link to unprefixed path
        assert!(result.contains("hreflang=\"en\" href=\"https://example.com/404\""));
        // Should not have de (current language)
        assert!(!result.contains("hreflang=\"de\""));

        // Test with trailing slash in the path
        let mut metadata_with_slash = HashMap::new();
        metadata_with_slash.insert("path".to_string(), "/de/page/".to_string()); // With trailing slash
        metadata_with_slash.insert("lang".to_string(), "de".to_string());
        metadata_with_slash.insert("alternate_languages".to_string(), "en,de,es".to_string());

        let result_with_slash = generator.generate_alternate_links(&metadata_with_slash);

        println!(
            "Generated alternate links with slash: {}",
            result_with_slash
        );

        // Should preserve trailing slash for other languages
        assert!(result_with_slash.contains("hreflang=\"es\" href=\"https://example.com/es/page/\""));
        assert!(result_with_slash.contains("hreflang=\"en\" href=\"https://example.com/page/\""));
    }
}
