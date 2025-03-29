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
