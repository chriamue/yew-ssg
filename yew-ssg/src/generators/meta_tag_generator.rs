use crate::generator::Generator;
use crate::processors::GeneratorOutputSupport;
use std::any::Any;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct MetaTagGenerator {
    pub default_description: String,
    pub default_keywords: Vec<String>,
}

impl Generator for MetaTagGenerator {
    fn name(&self) -> &'static str {
        "meta_tags"
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
            // Main output: full meta tags block
            "meta_tags" => {
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

            // Individual meta content values (not the complete HTML tags)
            "description_content" => {
                let description = metadata
                    .get("description")
                    .cloned()
                    .unwrap_or_else(|| self.default_description.clone());
                Ok(description)
            }

            "keywords_content" => {
                let keywords = metadata
                    .get("keywords")
                    .cloned()
                    .unwrap_or_else(|| self.default_keywords.join(", "));
                Ok(keywords)
            }

            // Individual HTML meta elements
            "description" => {
                let description = metadata
                    .get("description")
                    .cloned()
                    .unwrap_or_else(|| self.default_description.clone());
                Ok(format!(
                    "<meta name=\"description\" content=\"{}\">\n",
                    description
                ))
            }

            "keywords" => {
                let keywords = metadata
                    .get("keywords")
                    .cloned()
                    .unwrap_or_else(|| self.default_keywords.join(", "));
                Ok(format!(
                    "<meta name=\"keywords\" content=\"{}\">\n",
                    keywords
                ))
            }

            "canonical" => {
                if let Some(canonical) = metadata.get("canonical") {
                    Ok(format!("<link rel=\"canonical\" href=\"{}\">\n", canonical))
                } else {
                    Ok("".to_string()) // No canonical URL available
                }
            }

            // Unsupported key
            _ => Err(format!("MetaTagGenerator does not support key: {}", key).into()),
        }
    }

    fn clone_box(&self) -> Box<dyn Generator> {
        Box::new(self.clone())
    }
}

impl GeneratorOutputSupport for MetaTagGenerator {
    fn supported_outputs(&self) -> Vec<&'static str> {
        vec![
            "meta_tags",
            "description",
            "keywords",
            "canonical",
            "description_content",
            "keywords_content",
        ]
    }
}
