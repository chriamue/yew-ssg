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

    /// Languages that should have canonical URLs pointing to default language
    /// - If None: Each language has its own canonical URL
    /// - If Some(vec): Only languages in this list have canonical to default
    /// - If Some(vec) and vec is empty: All languages point to default
    pub canonical_to_default_langs: Option<Vec<String>>,

    /// The default language code to use when constructing canonical URLs
    pub default_language: String,
}

impl Default for CanonicalLinkGenerator {
    fn default() -> Self {
        Self {
            default_domain: None,
            canonical_to_default_langs: None, // Default to each language having its own canonical URL
            default_language: "en".to_string(),
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
            ..Self::default()
        }
    }

    /// Create a new generator with specific canonical URL behavior for translations
    pub fn with_domain_and_language_config(
        domain: &str,
        canonical_to_default_langs: Option<Vec<String>>,
        default_language: Option<&str>,
    ) -> Self {
        Self {
            default_domain: Some(domain.to_string()),
            canonical_to_default_langs,
            default_language: default_language.unwrap_or("en").to_string(),
        }
    }
    /// Get the base domain from metadata or default
    fn get_domain(&self, metadata: &HashMap<String, String>) -> Option<String> {
        metadata
            .get("domain")
            .cloned()
            .or_else(|| self.default_domain.clone())
    }

    /// Remove any known language prefix from a path (for canonical and alternates)
    fn strip_any_language_prefix<'a>(&self, path: &'a str, langs: &[&str]) -> String {
        for lang in langs {
            let prefix = format!("/{}/", lang);
            let prefix_end = format!("/{}", lang);
            if path.starts_with(&prefix) {
                return path.replacen(&prefix, "/", 1);
            }
            if path == prefix_end {
                return "/".to_string();
            }
            if path.starts_with(&prefix_end) && path.chars().nth(prefix_end.len()) == Some('/') {
                return path.replacen(&prefix_end, "", 1);
            }
        }
        path.to_string()
    }

    /// Get all language codes to check for prefix removal
    fn all_langs<'a>(&self, metadata: &'a HashMap<String, String>) -> Vec<&'a str> {
        if let Some(langs) = metadata.get("alternate_languages") {
            langs
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect()
        } else {
            // No alternates configured, so don't generate any alternates or strip unknown prefixes
            vec![]
        }
    }

    /// Generate a canonical link tag
    fn generate_canonical(&self, metadata: &HashMap<String, String>) -> String {
        if let Some(canonical) = metadata.get("canonical") {
            return format!("<link rel=\"canonical\" href=\"{}\">\n", canonical);
        }
        let domain = match self.get_domain(metadata) {
            Some(d) => d.trim_end_matches('/').to_string(),
            None => return "".to_string(),
        };
        let current_lang = metadata
            .get("lang")
            .cloned()
            .unwrap_or_else(|| self.default_language.clone());
        let path = metadata
            .get("path")
            .cloned()
            .unwrap_or_else(|| "/".to_string());
        // If lang is missing, strip all alternates; else, strip current and default
        let langs: Vec<&str> = if metadata.get("lang").is_some() {
            let mut langs = vec![current_lang.as_str()];
            if current_lang != self.default_language {
                langs.push(self.default_language.as_str());
            }
            langs
        } else {
            self.all_langs(metadata)
        };
        let base_path = self.strip_any_language_prefix(&path, &langs);

        let is_default_lang = current_lang == self.default_language;
        let should_point_to_default = match &self.canonical_to_default_langs {
            None => false,
            Some(langs) if langs.is_empty() => false,
            Some(langs) => langs.contains(&current_lang),
        };

        let canonical_url = if is_default_lang || should_point_to_default {
            if base_path == "/" {
                format!("{}/", domain)
            } else {
                format!("{}{}", domain, base_path)
            }
        } else {
            if base_path == "/" {
                format!("{}/{}/", domain, current_lang)
            } else {
                format!("{}/{}{}", domain, current_lang, base_path)
            }
        };
        format!("<link rel=\"canonical\" href=\"{}\">\n", canonical_url)
    }

    /// Generate alternate language links
    fn generate_alternate_links(&self, metadata: &HashMap<String, String>) -> String {
        let mut result = String::new();
        let langs: Vec<&str> = self.all_langs(metadata);
        if langs.is_empty() {
            return result;
        }
        let domain = match self.get_domain(metadata) {
            Some(d) => d,
            None => return result,
        };
        let base_url = url::Url::parse(&domain)
            .unwrap_or_else(|_| url::Url::parse(&format!("https://{}", domain)).unwrap());
        let current_path = metadata
            .get("path")
            .cloned()
            .unwrap_or_else(|| "/".to_string());
        let current_lang = metadata
            .get("lang")
            .cloned()
            .unwrap_or_else(|| self.default_language.clone());
        let base_path = self.strip_any_language_prefix(&current_path, &langs);
        let clean_base_path = base_path.trim_matches('/');
        let has_trailing_slash = base_path.ends_with('/') && base_path != "/";

        for lang in &langs {
            if *lang == current_lang {
                continue;
            }
            let lang_key = format!("alternate_url_{}", lang);
            if let Some(lang_url) = metadata.get(&lang_key) {
                result.push_str(&format!(
                    "<link rel=\"alternate\" hreflang=\"{}\" href=\"{}\">\n",
                    lang, lang_url
                ));
            } else {
                let lang_path = if *lang == self.default_language {
                    if clean_base_path.is_empty() {
                        "/".to_string()
                    } else if has_trailing_slash {
                        format!("/{}/", clean_base_path)
                    } else {
                        format!("/{}", clean_base_path)
                    }
                } else {
                    if clean_base_path.is_empty() {
                        format!("/{}/", lang)
                    } else if has_trailing_slash {
                        format!("/{}/{}/", lang, clean_base_path)
                    } else {
                        format!("/{}/{}", lang, clean_base_path)
                    }
                };
                let full_url = base_url.join(&lang_path).unwrap_or_else(|_| {
                    format!("{}{}", domain.trim_end_matches('/'), lang_path)
                        .parse()
                        .unwrap()
                });
                result.push_str(&format!(
                    "<link rel=\"alternate\" hreflang=\"{}\" href=\"{}\">\n",
                    lang, full_url
                ));
            }
        }
        // x-default
        let default_lang = self.default_language.as_str();
        let default_key = format!("alternate_url_{}", default_lang);
        let x_default_url = if let Some(url) = metadata.get("alternate_url_x_default") {
            url.clone()
        } else if let Some(default_url) = metadata.get(&default_key) {
            default_url.clone()
        } else {
            let default_path = if clean_base_path.is_empty() {
                "/".to_string()
            } else if has_trailing_slash {
                format!("/{}/", clean_base_path)
            } else {
                format!("/{}", clean_base_path)
            };
            base_url
                .join(&default_path)
                .unwrap_or_else(|_| {
                    format!("{}{}", domain.trim_end_matches('/'), default_path)
                        .parse()
                        .unwrap()
                })
                .to_string()
        };
        result.push_str(&format!(
            "<link rel=\"alternate\" hreflang=\"x-default\" href=\"{}\">\n",
            x_default_url
        ));
        result
    }

    /// Get just the canonical URL value (without HTML tags)
    fn get_canonical_url(&self, metadata: &HashMap<String, String>) -> String {
        if let Some(canonical) = metadata.get("canonical") {
            canonical.clone()
        } else if let Some(domain) = self.get_domain(metadata) {
            let current_lang = metadata
                .get("lang")
                .cloned()
                .unwrap_or_else(|| self.default_language.clone());
            let path = metadata
                .get("path")
                .cloned()
                .unwrap_or_else(|| "/".to_string());
            let langs: Vec<&str> = if metadata.get("lang").is_some() {
                let mut langs = vec![current_lang.as_str()];
                if current_lang != self.default_language {
                    langs.push(self.default_language.as_str());
                }
                langs
            } else {
                self.all_langs(metadata)
            };
            let base_path = self.strip_any_language_prefix(&path, &langs);
            let is_default_lang = current_lang == self.default_language;
            let should_point_to_default = match &self.canonical_to_default_langs {
                None => false,
                Some(langs) if langs.is_empty() => false,
                Some(langs) => langs.contains(&current_lang),
            };
            let domain = domain.trim_end_matches('/');
            if is_default_lang || should_point_to_default {
                if base_path == "/" {
                    format!("{}/", domain)
                } else {
                    format!("{}{}", domain, base_path)
                }
            } else {
                if base_path == "/" {
                    format!("{}/{}/", domain, current_lang)
                } else {
                    format!("{}/{}{}", domain, current_lang, base_path)
                }
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

    #[test]
    fn test_canonical_translation_behavior() {
        // Test with translations pointing to default language
        let generator = CanonicalLinkGenerator::with_domain_and_language_config(
            "https://example.com",
            Some(vec!["de".to_string()]),
            Some("en"),
        );

        // Test German page with canonical pointing to default language
        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), "/de/about".to_string());
        metadata.insert("lang".to_string(), "de".to_string());

        let result = generator.generate("canonical", "", "", &metadata).unwrap();
        assert_eq!(
            result, "<link rel=\"canonical\" href=\"https://example.com/about\">\n",
            "With canonical_to_default_langs=Some([*]), it should point to default language"
        );

        // Test with only specific languages pointing to default
        let generator = CanonicalLinkGenerator::with_domain_and_language_config(
            "https://example.com",
            Some(vec!["de".to_string(), "es".to_string()]), // Only German and Spanish point to default
            Some("en"),
        );

        // Test German (should point to default)
        let result = generator.generate("canonical", "", "", &metadata).unwrap();
        assert_eq!(
            result, "<link rel=\"canonical\" href=\"https://example.com/about\">\n",
            "With canonical_to_default_langs=Some(['de']), German should point to default language"
        );

        // Test French (should point to itself)
        let mut metadata_fr = HashMap::new();
        metadata_fr.insert("path".to_string(), "/fr/about".to_string());
        metadata_fr.insert("lang".to_string(), "fr".to_string());

        let result_fr = generator
            .generate("canonical", "", "", &metadata_fr)
            .unwrap();
        assert_eq!(
            result_fr, "<link rel=\"canonical\" href=\"https://example.com/fr/about\">\n",
            "With canonical_to_default_langs=Some(['de', 'es']), French should point to itself"
        );

        // Test with each language having its own canonical
        let generator = CanonicalLinkGenerator::with_domain_and_language_config(
            "https://example.com",
            None, // None means each language has its own canonical
            Some("en"),
        );

        let result = generator.generate("canonical", "", "", &metadata).unwrap();
        assert_eq!(
            result, "<link rel=\"canonical\" href=\"https://example.com/de/about\">\n",
            "With canonical_to_default_langs=None, each language should point to itself"
        );

        // Test with default language itself
        let mut metadata_en = HashMap::new();
        metadata_en.insert("path".to_string(), "/about".to_string());
        metadata_en.insert("lang".to_string(), "en".to_string());

        // Default language should always point to itself regardless of configuration
        let result_en = generator
            .generate("canonical", "", "", &metadata_en)
            .unwrap();
        assert_eq!(
            result_en, "<link rel=\"canonical\" href=\"https://example.com/about\">\n",
            "Default language should always point to itself"
        );
    }

    #[test]
    fn test_canonical_url_only() {
        // Test the canonical_url output (without HTML tags)
        let generator = CanonicalLinkGenerator::with_domain_and_language_config(
            "https://example.com",
            Some(vec!["de".to_string()]), // Only German points to default
            Some("en"),
        );

        // Test German (should point to default)
        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), "/de/products".to_string());
        metadata.insert("lang".to_string(), "de".to_string());

        let result = generator
            .generate("canonical_url", "", "", &metadata)
            .unwrap();
        assert_eq!(
            result, "https://example.com/products",
            "German canonical URL should point to default language version"
        );

        // Test French (should point to itself)
        let mut metadata_fr = HashMap::new();
        metadata_fr.insert("path".to_string(), "/fr/products".to_string());
        metadata_fr.insert("lang".to_string(), "fr".to_string());

        let result_fr = generator
            .generate("canonical_url", "", "", &metadata_fr)
            .unwrap();
        assert_eq!(
            result_fr, "https://example.com/fr/products",
            "French canonical URL should point to itself"
        );
    }

    #[test]
    fn test_custom_canonical_with_translation_behavior() {
        // Test that explicit canonical URL always takes precedence
        let generator = CanonicalLinkGenerator::with_domain_and_language_config(
            "https://example.com",
            Some(vec![]), // All languages point to default
            Some("en"),
        );

        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), "/de/about".to_string());
        metadata.insert("lang".to_string(), "de".to_string());
        metadata.insert(
            "canonical".to_string(),
            "https://example.com/custom-canonical".to_string(),
        );

        let result = generator.generate("canonical", "", "", &metadata).unwrap();
        assert_eq!(
            result, "<link rel=\"canonical\" href=\"https://example.com/custom-canonical\">\n",
            "Explicit canonical URL should take precedence over translation behavior"
        );
    }

    #[test]
    fn test_complex_path_canonicalization() {
        // Test with complex paths that include other segments besides language
        let generator = CanonicalLinkGenerator::with_domain_and_language_config(
            "https://example.com",
            Some(vec!["de".to_string()]),
            Some("en"),
        );

        // Test nested paths
        let mut metadata = HashMap::new();
        metadata.insert(
            "path".to_string(),
            "/de/products/category/item-123".to_string(),
        );
        metadata.insert("lang".to_string(), "de".to_string());

        let result = generator.generate("canonical", "", "", &metadata).unwrap();
        assert_eq!(
            result,
            "<link rel=\"canonical\" href=\"https://example.com/products/category/item-123\">\n",
            "Complex path canonicalization should work correctly"
        );

        // Test with query parameters
        let mut metadata_with_query = HashMap::new();
        metadata_with_query.insert("path".to_string(), "/de/search?q=test&page=2".to_string());
        metadata_with_query.insert("lang".to_string(), "de".to_string());

        let result_with_query = generator
            .generate("canonical", "", "", &metadata_with_query)
            .unwrap();
        assert_eq!(
            result_with_query,
            "<link rel=\"canonical\" href=\"https://example.com/search?q=test&page=2\">\n",
            "Path with query parameters should be properly canonicalized"
        );
    }

    #[test]
    fn test_default_language_change() {
        // Test with a non-English default language
        let generator = CanonicalLinkGenerator::with_domain_and_language_config(
            "https://example.com",
            Some(vec!["en".to_string(), "es".to_string()]), // English and Spanish point to default
            Some("de"),                                     // German is the default language
        );

        // Test English (should point to German)
        let mut metadata_en = HashMap::new();
        metadata_en.insert("path".to_string(), "/en/about".to_string());
        metadata_en.insert("lang".to_string(), "en".to_string());

        let result_en = generator
            .generate("canonical", "", "", &metadata_en)
            .unwrap();
        assert_eq!(
            result_en, "<link rel=\"canonical\" href=\"https://example.com/about\">\n",
            "With 'de' as default_language, English should point to unprefixed path"
        );

        // Test German (default language)
        let mut metadata_de = HashMap::new();
        metadata_de.insert("path".to_string(), "/about".to_string());
        metadata_de.insert("lang".to_string(), "de".to_string());

        let result_de = generator
            .generate("canonical", "", "", &metadata_de)
            .unwrap();
        assert_eq!(
            result_de, "<link rel=\"canonical\" href=\"https://example.com/about\">\n",
            "Default language (German) should point to itself"
        );

        // Test French (should point to itself since not in the list)
        let mut metadata_fr = HashMap::new();
        metadata_fr.insert("path".to_string(), "/fr/about".to_string());
        metadata_fr.insert("lang".to_string(), "fr".to_string());

        let result_fr = generator
            .generate("canonical", "", "", &metadata_fr)
            .unwrap();
        assert_eq!(
            result_fr, "<link rel=\"canonical\" href=\"https://example.com/fr/about\">\n",
            "French should point to itself since it's not in the canonicalization list"
        );
    }

    #[test]
    fn test_combined_output_with_translation_config() {
        // Test that the canonical_links output includes both canonical and alternates
        // with the translation configuration applied
        let generator = CanonicalLinkGenerator::with_domain_and_language_config(
            "https://example.com",
            Some(vec!["de".to_string()]), // Only German points to default
            Some("en"),
        );

        let mut metadata = HashMap::new();
        metadata.insert("path".to_string(), "/de/products".to_string());
        metadata.insert("lang".to_string(), "de".to_string());
        metadata.insert("alternate_languages".to_string(), "en,de,fr".to_string());

        let result = generator
            .generate("canonical_links", "", "", &metadata)
            .unwrap();

        // Should contain canonical pointing to default language
        assert!(result.contains("<link rel=\"canonical\" href=\"https://example.com/products\">"));

        // Should contain alternate links
        assert!(result.contains(
            "<link rel=\"alternate\" hreflang=\"en\" href=\"https://example.com/products"
        ));
        assert!(result.contains(
            "<link rel=\"alternate\" hreflang=\"fr\" href=\"https://example.com/fr/products"
        ));

        // Should contain x-default link pointing to default language
        assert!(result.contains(
            "<link rel=\"alternate\" hreflang=\"x-default\" href=\"https://example.com/products"
        ));
    }

    #[test]
    fn test_example_config_scenario() {
        // This test replicates the exact conditions from the example config
        // where canonical_to_default_langs is set to "de,es"

        let generator = CanonicalLinkGenerator::with_domain_and_language_config(
            "https://example.com",
            Some(vec!["de".to_string(), "es".to_string()]), // Only German and Spanish point to default
            Some("en"),
        );

        // Test 1: German 404 page (should point to default language)
        let mut metadata_de_404 = HashMap::new();
        metadata_de_404.insert("path".to_string(), "/de/404".to_string());
        metadata_de_404.insert("lang".to_string(), "de".to_string());
        metadata_de_404.insert("alternate_languages".to_string(), "en,es,de".to_string());

        let result_de_404 = generator
            .generate("canonical_links", "", "", &metadata_de_404)
            .unwrap();
        println!("German 404 canonical links:\n{}", result_de_404);

        // Should contain canonical pointing to default language version (without /de/)
        assert!(
            result_de_404.contains("<link rel=\"canonical\" href=\"https://example.com/404\">"),
            "German 404 canonical should point to default language version without prefix"
        );

        // Should contain correct alternate links
        assert!(
            result_de_404.contains(
                "<link rel=\"alternate\" hreflang=\"en\" href=\"https://example.com/404\">"
            ),
            "Should have English alternate link without prefix"
        );
        assert!(
            result_de_404.contains(
                "<link rel=\"alternate\" hreflang=\"es\" href=\"https://example.com/es/404\">"
            ),
            "Should have Spanish alternate link with prefix"
        );
        // Shouldn't have an alternate link for German (current language)
        assert!(
            !result_de_404.contains("hreflang=\"de\""),
            "Shouldn't have alternate link for current language (German)"
        );

        // Test 2: Spanish page (should also point to default language)
        let mut metadata_es = HashMap::new();
        metadata_es.insert("path".to_string(), "/es/about".to_string());
        metadata_es.insert("lang".to_string(), "es".to_string());
        metadata_es.insert("alternate_languages".to_string(), "en,es,de".to_string());

        let result_es = generator
            .generate("canonical_links", "", "", &metadata_es)
            .unwrap();
        println!("Spanish about canonical links:\n{}", result_es);

        // Should contain canonical pointing to default language version (without /es/)
        assert!(
            result_es.contains("<link rel=\"canonical\" href=\"https://example.com/about\">"),
            "Spanish about canonical should point to default language version without prefix"
        );

        // Test 3: French page (should have its own canonical because not in the list)
        let mut metadata_fr = HashMap::new();
        metadata_fr.insert("path".to_string(), "/fr/about".to_string());
        metadata_fr.insert("lang".to_string(), "fr".to_string());
        metadata_fr.insert("alternate_languages".to_string(), "en,es,de,fr".to_string());

        let result_fr = generator
            .generate("canonical_links", "", "", &metadata_fr)
            .unwrap();
        println!("French about canonical links:\n{}", result_fr);

        // Should contain canonical pointing to itself (with /fr/)
        assert!(
            result_fr.contains("<link rel=\"canonical\" href=\"https://example.com/fr/about\">"),
            "French about canonical should point to itself with language prefix"
        );

        // Test 4: English page (default language)
        let mut metadata_en = HashMap::new();
        metadata_en.insert("path".to_string(), "/en/about".to_string()); // Explicitly with /en/ prefix
        metadata_en.insert("lang".to_string(), "en".to_string());
        metadata_en.insert("alternate_languages".to_string(), "en,es,de".to_string());

        let result_en = generator
            .generate("canonical_links", "", "", &metadata_en)
            .unwrap();
        println!("English about canonical links:\n{}", result_en);

        // Should contain canonical pointing to version without language prefix
        assert!(
            result_en.contains("<link rel=\"canonical\" href=\"https://example.com/about\">"),
            "English page canonical should always point to version without language prefix"
        );

        // Test 5: Nested path with language prefix
        let mut metadata_nested = HashMap::new();
        metadata_nested.insert("path".to_string(), "/de/products/category/item".to_string());
        metadata_nested.insert("lang".to_string(), "de".to_string());
        metadata_nested.insert("alternate_languages".to_string(), "en,es,de".to_string());

        let result_nested = generator
            .generate("canonical_links", "", "", &metadata_nested)
            .unwrap();
        println!("Nested path canonical links:\n{}", result_nested);

        // Should contain canonical without language prefix
        assert!(
            result_nested.contains(
                "<link rel=\"canonical\" href=\"https://example.com/products/category/item\">"
            ),
            "Nested German path should have canonical to unprefixed version"
        );

        // Should have correct alternate links (no double prefixes)
        assert!(result_nested.contains("<link rel=\"alternate\" hreflang=\"en\" href=\"https://example.com/products/category/item\">"),
                    "English alternate should not have language prefix");
        assert!(result_nested.contains("<link rel=\"alternate\" hreflang=\"es\" href=\"https://example.com/es/products/category/item\">"),
                    "Spanish alternate should have language prefix (no duplicates)");

        // Test 6: Edge case - no language in metadata
        let mut metadata_no_lang = HashMap::new();
        metadata_no_lang.insert("path".to_string(), "/de/404".to_string()); // Path has language but metadata doesn't
        metadata_no_lang.insert("alternate_languages".to_string(), "en,es,de".to_string());

        let result_no_lang = generator
            .generate("canonical_links", "", "", &metadata_no_lang)
            .unwrap();
        println!(
            "No language in metadata canonical links:\n{}",
            result_no_lang
        );

        // Should detect the language from path and remove it, defaulting to unprefixed
        assert!(
            result_no_lang.contains("<link rel=\"canonical\" href=\"https://example.com/404\">"),
            "Should still remove language prefix even when lang not in metadata"
        );
    }

    #[test]
    fn test_canonical_to_default_langs_none_and_empty_vec() {
        // Test with None (should mean "each language has its own canonical")
        let generator_none = CanonicalLinkGenerator::with_domain_and_language_config(
            "https://example.com",
            None,
            Some("en"),
        );

        // Test with Some(vec![]) (should also mean "each language has its own canonical")
        let generator_empty = CanonicalLinkGenerator::with_domain_and_language_config(
            "https://example.com",
            Some(vec![]),
            Some("en"),
        );

        for generator in &[&generator_none, &generator_empty] {
            // German page: canonical should be /de/about
            let mut metadata_de = HashMap::new();
            metadata_de.insert("path".to_string(), "/de/about".to_string());
            metadata_de.insert("lang".to_string(), "de".to_string());
            metadata_de.insert("alternate_languages".to_string(), "en,de,es".to_string());

            let result_de = generator
                .generate("canonical", "", "", &metadata_de)
                .unwrap();
            assert_eq!(
                result_de, "<link rel=\"canonical\" href=\"https://example.com/de/about\">\n",
                "With canonical_to_default_langs: None or [], German should have its own canonical"
            );

            // Spanish page: canonical should be /es/about
            let mut metadata_es = HashMap::new();
            metadata_es.insert("path".to_string(), "/es/about".to_string());
            metadata_es.insert("lang".to_string(), "es".to_string());
            metadata_es.insert("alternate_languages".to_string(), "en,de,es".to_string());

            let result_es = generator
                .generate("canonical", "", "", &metadata_es)
                .unwrap();
            assert_eq!(
                result_es,
                "<link rel=\"canonical\" href=\"https://example.com/es/about\">\n",
                "With canonical_to_default_langs: None or [], Spanish should have its own canonical"
            );

            // English page: canonical should be /about (no prefix)
            let mut metadata_en = HashMap::new();
            metadata_en.insert("path".to_string(), "/en/about".to_string());
            metadata_en.insert("lang".to_string(), "en".to_string());
            metadata_en.insert("alternate_languages".to_string(), "en,de,es".to_string());

            let result_en = generator
                .generate("canonical", "", "", &metadata_en)
                .unwrap();
            assert_eq!(
                result_en,
                "<link rel=\"canonical\" href=\"https://example.com/about\">\n",
                "With canonical_to_default_langs: None or [], English should have canonical without prefix"
            );
        }
    }

    #[test]
    fn test_canonical_to_default_langs_config_parser() {
        use crate::config_loader::{CanonicalBehavior, GeneralConfig};
        // false
        let yaml_false = r#"
    canonical_to_default_langs: false
    default_language: en
    "#;
        let config: GeneralConfig = serde_yaml::from_str(yaml_false).unwrap();
        assert_eq!(
            config.canonical_to_default_langs,
            Some(CanonicalBehavior::Boolean(false)),
            "false should parse as Boolean(false)"
        );

        // true
        let yaml_true = r#"
    canonical_to_default_langs: true
    default_language: en
    "#;
        let config: GeneralConfig = serde_yaml::from_str(yaml_true).unwrap();
        assert_eq!(
            config.canonical_to_default_langs,
            Some(CanonicalBehavior::Boolean(true)),
            "true should parse as Boolean(true)"
        );

        // list
        let yaml_list = r#"
    canonical_to_default_langs: "de,es"
    default_language: en
    "#;
        let config: GeneralConfig = serde_yaml::from_str(yaml_list).unwrap();
        assert_eq!(
            config.canonical_to_default_langs,
            Some(CanonicalBehavior::Languages("de,es".to_string())),
            "list should parse as Languages"
        );
    }
}
