use crate::generator::Generator;
use crate::processors::GeneratorOutputSupport;
use serde_json::{json, Value};
use std::any::Any;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

/// Generator for JSON-LD structured data
///
/// JSON-LD (JavaScript Object Notation for Linked Data) is a method of encoding
/// linked data using JSON. This generator creates appropriate JSON-LD markup
/// based on the page type and available metadata, or loads it from files.
#[derive(Debug, Clone)]
pub struct JsonLdGenerator {
    /// Default type of the page (e.g., "WebPage", "Article", "Product")
    pub default_page_type: String,

    /// Domain to use for URL construction
    pub domain: Option<String>,

    /// Base directory for JSON-LD files (if not absolute)
    pub json_ld_base_dir: Option<String>,
}

impl Default for JsonLdGenerator {
    fn default() -> Self {
        Self {
            default_page_type: "WebPage".to_string(),
            domain: None,
            json_ld_base_dir: None,
        }
    }
}

impl JsonLdGenerator {
    /// Create a new JsonLdGenerator with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new JsonLdGenerator with a specified domain
    pub fn with_domain(domain: &str) -> Self {
        Self {
            default_page_type: "WebPage".to_string(),
            domain: Some(domain.to_string()),
            json_ld_base_dir: None,
        }
    }

    /// Create a new JsonLdGenerator with domain and base directory for JSON-LD files
    pub fn with_domain_and_base_dir(domain: &str, base_dir: &str) -> Self {
        Self {
            default_page_type: "WebPage".to_string(),
            domain: Some(domain.to_string()),
            json_ld_base_dir: Some(base_dir.to_string()),
        }
    }

    /// Load JSON-LD from a file
    fn load_json_ld_from_file(&self, file_path: &str) -> Result<Value, Box<dyn Error>> {
        // Resolve the path (either absolute or relative to base dir)
        let resolved_path = if Path::new(file_path).is_absolute() {
            file_path.to_string()
        } else if let Some(base_dir) = &self.json_ld_base_dir {
            format!("{}/{}", base_dir, file_path)
        } else {
            file_path.to_string()
        };

        // Read the file content
        let file_content = fs::read_to_string(&resolved_path)
            .map_err(|e| format!("Failed to read JSON-LD file '{}': {}", resolved_path, e))?;

        // Parse the JSON content
        let json_value: Value = serde_json::from_str(&file_content)
            .map_err(|e| format!("Failed to parse JSON-LD file '{}': {}", resolved_path, e))?;

        Ok(json_value)
    }

    /// Get the full URL by combining domain and path
    fn get_full_url(&self, path: &str, metadata: &HashMap<String, String>) -> String {
        // First check if a canonical URL is provided in metadata
        if let Some(canonical) = metadata.get("canonical") {
            return canonical.clone();
        }

        // Otherwise try to construct from domain and path
        if let Some(domain) = self.get_domain(metadata) {
            let domain_trimmed = domain.trim_end_matches('/');
            let path_with_slash = if !path.starts_with('/') {
                format!("/{}", path)
            } else {
                path.to_string()
            };

            return format!("{}{}", domain_trimmed, path_with_slash);
        }

        // If no domain is available, return just the path
        path.to_string()
    }

    /// Get domain from metadata or default
    fn get_domain(&self, metadata: &HashMap<String, String>) -> Option<String> {
        metadata
            .get("domain")
            .cloned()
            .or_else(|| self.domain.clone())
    }

    /// Generate WebPage JSON-LD
    fn generate_webpage(&self, path: &str, metadata: &HashMap<String, String>) -> Value {
        let url = self.get_full_url(path, metadata);
        let title = metadata.get("title").cloned().unwrap_or_default();
        let description = metadata.get("description").cloned().unwrap_or_default();

        json!({
            "@context": "https://schema.org",
            "@type": "WebPage",
            "@id": format!("{}#webpage", url),
            "url": url,
            "name": title,
            "description": description,
            "inLanguage": metadata.get("language").cloned().unwrap_or_else(|| "en".to_string())
        })
    }

    /// Generate Article JSON-LD
    fn generate_article(&self, path: &str, metadata: &HashMap<String, String>) -> Value {
        let url = self.get_full_url(path, metadata);
        let title = metadata.get("title").cloned().unwrap_or_default();
        let description = metadata.get("description").cloned().unwrap_or_default();
        let author = metadata.get("author").cloned().unwrap_or_default();
        let date_published = metadata.get("date_published").cloned();
        let date_modified = metadata.get("date_modified").cloned();

        let mut article = json!({
            "@context": "https://schema.org",
            "@type": "Article",
            "@id": format!("{}#article", url),
            "headline": title,
            "description": description,
            "url": url
        });

        // Add article-specific fields when available
        if !author.is_empty() {
            article["author"] = json!({
                "@type": "Person",
                "name": author
            });
        }

        if let Some(published) = date_published {
            article["datePublished"] = json!(published);
        }

        if let Some(modified) = date_modified {
            article["dateModified"] = json!(modified);
        }

        article
    }

    /// Generate Organization JSON-LD
    fn generate_organization(&self, metadata: &HashMap<String, String>) -> Value {
        let organization_name = metadata
            .get("organization_name")
            .cloned()
            .or_else(|| metadata.get("site_name").cloned())
            .unwrap_or_else(|| "Organization".to_string());

        let logo = metadata.get("organization_logo").cloned();
        let url = self.get_domain(metadata).unwrap_or_default();

        let mut org = json!({
            "@context": "https://schema.org",
            "@type": "Organization",
            "name": organization_name,
            "url": url,
        });

        if let Some(logo_url) = logo {
            org["logo"] = json!(logo_url);
        }

        org
    }

    /// Generate BreadcrumbList JSON-LD
    fn generate_breadcrumbs(&self, path: &str, metadata: &HashMap<String, String>) -> Value {
        let domain = self.get_domain(metadata).unwrap_or_else(|| "".to_string());

        // Parse the path into breadcrumb segments
        let segments: Vec<&str> = path.trim_matches('/').split('/').collect();
        let mut items = Vec::new();

        // Add home page as first item
        items.push(json!({
            "@type": "ListItem",
            "position": 1,
            "name": "Home",
            "item": domain
        }));

        // Build breadcrumb items from path segments
        let mut current_path = "".to_string();
        for (i, segment) in segments.iter().enumerate() {
            if segment.is_empty() {
                continue;
            }

            current_path = format!("{}/{}", current_path, segment);
            let position = i + 2; // +2 because home is position 1

            // Try to get a friendly name from metadata
            let key = format!("breadcrumb_{}", segment);
            let name = metadata
                .get(&key)
                .cloned()
                .unwrap_or_else(|| segment.replace('-', " ").replace('_', " "));

            items.push(json!({
                "@type": "ListItem",
                "position": position,
                "name": name,
                "item": format!("{}{}", domain, current_path)
            }));
        }

        json!({
            "@context": "https://schema.org",
            "@type": "BreadcrumbList",
            "itemListElement": items
        })
    }

    /// Get JSON-LD data based on configuration - either from file or generated
    fn get_json_ld_data(
        &self,
        route: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<Value, Box<dyn Error>> {
        // First check if a file path is provided
        if let Some(file_path) = metadata.get("json_ld_file") {
            return self.load_json_ld_from_file(file_path);
        }

        // Otherwise generate based on type
        let page_type = metadata
            .get("json_ld_type")
            .cloned()
            .unwrap_or_else(|| self.default_page_type.clone());

        let json_ld = match page_type.as_str() {
            "WebPage" | "WebSite" => self.generate_webpage(route, metadata),
            "Article" | "BlogPosting" | "TechArticle" => self.generate_article(route, metadata),
            "Organization" => self.generate_organization(metadata),
            "BreadcrumbList" => self.generate_breadcrumbs(route, metadata),
            "AboutPage" => {
                // Special case for AboutPage which extends WebPage
                let mut page = self.generate_webpage(route, metadata);
                page["@type"] = json!("AboutPage");
                page
            }
            _ => self.generate_webpage(route, metadata), // Default to WebPage
        };

        Ok(json_ld)
    }
}

impl Generator for JsonLdGenerator {
    fn name(&self) -> &'static str {
        "json_ld"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn generate(
        &self,
        key: &str,
        route: &str,
        _content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        // Check if JSON-LD is explicitly disabled
        if let Some(enabled) = metadata.get("json_ld_enabled") {
            if enabled.to_lowercase() == "false" {
                return Ok("".to_string());
            }
        }

        match key {
            // Main output: complete JSON-LD script tag
            "json_ld" => {
                let json_ld = self.get_json_ld_data(route, metadata)?;
                let json_string = serde_json::to_string_pretty(&json_ld)?;
                Ok(format!(
                    "<script type=\"application/ld+json\">\n{}\n</script>",
                    json_string
                ))
            }

            // Just the JSON-LD object without script tags
            "json_ld_data" => {
                let json_ld = self.get_json_ld_data(route, metadata)?;
                Ok(serde_json::to_string_pretty(&json_ld)?)
            }

            // Generate specific types regardless of metadata type setting
            "webpage_json_ld" => {
                let json_ld = self.generate_webpage(route, metadata);
                let json_string = serde_json::to_string_pretty(&json_ld)?;
                Ok(format!(
                    "<script type=\"application/ld+json\">\n{}\n</script>",
                    json_string
                ))
            }

            "article_json_ld" => {
                let json_ld = self.generate_article(route, metadata);
                let json_string = serde_json::to_string_pretty(&json_ld)?;
                Ok(format!(
                    "<script type=\"application/ld+json\">\n{}\n</script>",
                    json_string
                ))
            }

            "organization_json_ld" => {
                let json_ld = self.generate_organization(metadata);
                let json_string = serde_json::to_string_pretty(&json_ld)?;
                Ok(format!(
                    "<script type=\"application/ld+json\">\n{}\n</script>",
                    json_string
                ))
            }

            "breadcrumbs_json_ld" => {
                let json_ld = self.generate_breadcrumbs(route, metadata);
                let json_string = serde_json::to_string_pretty(&json_ld)?;
                Ok(format!(
                    "<script type=\"application/ld+json\">\n{}\n</script>",
                    json_string
                ))
            }

            // Multiple types combined
            "all_json_ld" => {
                // Check if we have a file
                if let Some(file_path) = metadata.get("json_ld_file") {
                    let json_ld = self.load_json_ld_from_file(file_path)?;
                    let json_string = serde_json::to_string_pretty(&json_ld)?;
                    return Ok(format!(
                        "<script type=\"application/ld+json\">\n{}\n</script>",
                        json_string
                    ));
                }

                // Otherwise generate multiple types
                let webpage = self.generate_webpage(route, metadata);
                let organization = self.generate_organization(metadata);
                let breadcrumbs = self.generate_breadcrumbs(route, metadata);

                let mut combined = String::new();

                let webpage_string = serde_json::to_string_pretty(&webpage)?;
                combined.push_str(&format!(
                    "<script type=\"application/ld+json\">\n{}\n</script>\n",
                    webpage_string
                ));

                let org_string = serde_json::to_string_pretty(&organization)?;
                combined.push_str(&format!(
                    "<script type=\"application/ld+json\">\n{}\n</script>\n",
                    org_string
                ));

                let breadcrumbs_string = serde_json::to_string_pretty(&breadcrumbs)?;
                combined.push_str(&format!(
                    "<script type=\"application/ld+json\">\n{}\n</script>",
                    breadcrumbs_string
                ));

                Ok(combined)
            }

            // Unsupported key
            _ => Err(format!("JsonLdGenerator does not support key: {}", key).into()),
        }
    }

    fn clone_box(&self) -> Box<dyn Generator> {
        Box::new(self.clone())
    }
}

impl GeneratorOutputSupport for JsonLdGenerator {
    fn supported_outputs(&self) -> Vec<&'static str> {
        vec![
            "json_ld",
            "json_ld_data",
            "webpage_json_ld",
            "article_json_ld",
            "organization_json_ld",
            "breadcrumbs_json_ld",
            "all_json_ld",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_webpage_jsonld() {
        let generator = JsonLdGenerator::with_domain("https://example.com");

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Test Page".to_string());
        metadata.insert("description".to_string(), "Test description".to_string());

        let result = generator
            .generate("json_ld", "/test", "", &metadata)
            .unwrap();

        assert!(result.contains("application/ld+json"));
        assert!(result.contains("\"@type\": \"WebPage\""));
        assert!(result.contains("\"url\": \"https://example.com/test\""));
        assert!(result.contains("\"name\": \"Test Page\""));
    }

    #[test]
    fn test_article_jsonld() {
        let generator = JsonLdGenerator::with_domain("https://example.com");

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Article Title".to_string());
        metadata.insert("description".to_string(), "Article description".to_string());
        metadata.insert("author".to_string(), "John Doe".to_string());
        metadata.insert("date_published".to_string(), "2023-05-15".to_string());
        metadata.insert("json_ld_type".to_string(), "Article".to_string());

        let result = generator
            .generate("json_ld", "/blog/post", "", &metadata)
            .unwrap();

        assert!(result.contains("\"@type\": \"Article\""));
        assert!(result.contains("\"datePublished\": \"2023-05-15\""));
        assert!(result.contains("\"author\": {"));
        assert!(result.contains("\"name\": \"John Doe\""));
    }

    #[test]
    fn test_load_from_file() -> Result<(), Box<dyn Error>> {
        // Create a temporary JSON-LD file
        let json_content = r#"{
            "@context": "https://schema.org",
            "@type": "SoftwareSourceCode",
            "name": "test-crate",
            "description": "A test crate",
            "version": "1.0.0"
        }"#;

        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(json_content.as_bytes())?;
        let file_path = temp_file.path().to_string_lossy().to_string();

        // Test loading from file
        let generator = JsonLdGenerator::new();
        let mut metadata = HashMap::new();
        metadata.insert("json_ld_file".to_string(), file_path);

        let result = generator.generate("json_ld", "/test", "", &metadata)?;

        // Check the loaded content
        assert!(result.contains("application/ld+json"));
        assert!(result.contains("\"@type\": \"SoftwareSourceCode\""));
        assert!(result.contains("\"name\": \"test-crate\""));
        assert!(result.contains("\"version\": \"1.0.0\""));

        Ok(())
    }

    #[test]
    fn test_about_page_type() {
        let generator = JsonLdGenerator::with_domain("https://example.com");

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "About Us".to_string());
        metadata.insert(
            "description".to_string(),
            "About page description".to_string(),
        );
        metadata.insert("json_ld_type".to_string(), "AboutPage".to_string());

        let result = generator
            .generate("json_ld", "/about", "", &metadata)
            .unwrap();

        assert!(result.contains("\"@type\": \"AboutPage\""));
        assert!(result.contains("\"url\": \"https://example.com/about\""));
        assert!(result.contains("\"name\": \"About Us\""));
    }

    #[test]
    fn test_json_ld_disabled() {
        let generator = JsonLdGenerator::with_domain("https://example.com");

        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "Error Page".to_string());
        metadata.insert("json_ld_enabled".to_string(), "false".to_string());

        let result = generator
            .generate("json_ld", "/error", "", &metadata)
            .unwrap();

        // Should be empty when disabled
        assert_eq!(result, "");
    }

    #[test]
    fn test_with_base_dir() -> Result<(), Box<dyn Error>> {
        // Create a temporary directory
        let temp_dir = tempfile::tempdir()?;
        let base_dir = temp_dir.path().to_string_lossy().to_string();

        // Create a JSON-LD file in the base directory
        let json_path = format!("{}/test.jsonld", base_dir);
        let json_content = r#"{
            "@context": "https://schema.org",
            "@type": "WebSite",
            "name": "Test with Base Dir",
            "url": "https://example.com"
        }"#;

        fs::write(&json_path, json_content)?;

        // Use a relative path in metadata with the base dir in generator
        let generator = JsonLdGenerator::with_domain_and_base_dir("https://example.com", &base_dir);
        let mut metadata = HashMap::new();
        metadata.insert("json_ld_file".to_string(), "test.jsonld".to_string());

        let result = generator.generate("json_ld", "/test", "", &metadata)?;

        // Check the loaded content
        assert!(result.contains("\"name\": \"Test with Base Dir\""));

        Ok(())
    }
}
