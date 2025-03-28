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
