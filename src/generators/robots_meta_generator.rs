use crate::generator::Generator;
use crate::processors::{AttributeSupport, GeneratorOutputSupport, TemplateVariableSupport};
use std::any::Any;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct RobotsMetaGenerator {
    pub default_robots: String,
}

impl Generator for RobotsMetaGenerator {
    fn name(&self) -> &'static str {
        "robots_meta"
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
            // Main output: robots meta tag
            "robots_meta" => {
                let robots = metadata
                    .get("robots")
                    .cloned()
                    .unwrap_or_else(|| self.default_robots.clone());

                Ok(format!("<meta name=\"robots\" content=\"{}\">\n", robots))
            }

            // Individual robots content
            "robots" => {
                let robots = metadata
                    .get("robots")
                    .cloned()
                    .unwrap_or_else(|| self.default_robots.clone());

                Ok(robots)
            }

            // Unsupported key
            _ => Err(format!("RobotsMetaGenerator does not support key: {}", key).into()),
        }
    }

    fn clone_box(&self) -> Box<dyn Generator> {
        Box::new(self.clone())
    }
}

impl GeneratorOutputSupport for RobotsMetaGenerator {
    fn supported_outputs(&self) -> Vec<&'static str> {
        vec!["robots_meta", "robots"]
    }
}

impl AttributeSupport for RobotsMetaGenerator {
    fn attributes(&self) -> Vec<&'static str> {
        vec!["robots_meta", "robots"]
    }
}

impl TemplateVariableSupport for RobotsMetaGenerator {
    fn template_variables(&self) -> Vec<&'static str> {
        vec!["robots"]
    }
}

#[cfg(test)]
mod tests {
    use crate::generator::Generator;
    use crate::generators::RobotsMetaGenerator;
    use std::collections::HashMap;

    #[test]
    fn test_robots_meta_generator() {
        let generator = RobotsMetaGenerator {
            default_robots: "index, follow".to_string(),
        };

        // Test with default value (using main key)
        let result = generator
            .generate(
                "robots_meta",
                "/test-route",
                "<div>Test content</div>",
                &HashMap::new(),
            )
            .unwrap();

        assert!(result.contains("<meta name=\"robots\" content=\"index, follow\">"));

        // Test with custom value
        let mut metadata = HashMap::new();
        metadata.insert("robots".to_string(), "noindex, nofollow".to_string());

        let result = generator
            .generate(
                "robots_meta",
                "/test-route",
                "<div>Test content</div>",
                &metadata,
            )
            .unwrap();

        assert!(result.contains("<meta name=\"robots\" content=\"noindex, nofollow\">"));
    }

    #[test]
    fn test_robots_value_only() {
        let generator = RobotsMetaGenerator {
            default_robots: "index, follow".to_string(),
        };

        // Test the robots value directly
        let result = generator
            .generate(
                "robots",
                "/test-route",
                "<div>Test content</div>",
                &HashMap::new(),
            )
            .unwrap();

        assert_eq!(result, "index, follow");

        // Test with custom value
        let mut metadata = HashMap::new();
        metadata.insert("robots".to_string(), "noindex, nofollow".to_string());

        let result = generator
            .generate(
                "robots",
                "/test-route",
                "<div>Test content</div>",
                &metadata,
            )
            .unwrap();

        assert_eq!(result, "noindex, nofollow");
    }

    #[test]
    fn test_unsupported_key() {
        let generator = RobotsMetaGenerator {
            default_robots: "index, follow".to_string(),
        };

        // Test with an unsupported key
        let result = generator.generate(
            "unsupported_key",
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
