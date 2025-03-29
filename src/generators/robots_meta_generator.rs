use crate::generator::Generator;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct RobotsMetaGenerator {
    default_robots: String,
}

impl Generator for RobotsMetaGenerator {
    fn name(&self) -> &'static str {
        "robots_meta"
    }

    fn generate(
        &self,
        _route: &str,
        _content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        let robots = metadata
            .get("robots")
            .cloned()
            .unwrap_or_else(|| self.default_robots.clone());

        Ok(format!("<meta name=\"robots\" content=\"{}\">\n", robots))
    }

    fn clone_box(&self) -> Box<dyn Generator> {
        Box::new(self.clone())
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

        // Test with default value
        let result = generator
            .generate("/test-route", "<div>Test content</div>", &HashMap::new())
            .unwrap();

        assert!(result.contains("<meta name=\"robots\" content=\"index, follow\">"));

        // Test with custom value
        let mut metadata = HashMap::new();
        metadata.insert("robots".to_string(), "noindex, nofollow".to_string());

        let result = generator
            .generate("/test-route", "<div>Test content</div>", &metadata)
            .unwrap();

        assert!(result.contains("<meta name=\"robots\" content=\"noindex, nofollow\">"));
    }
}
