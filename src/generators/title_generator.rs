use crate::generator::Generator;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct TitleGenerator;

impl Generator for TitleGenerator {
    fn name(&self) -> &'static str {
        "title"
    }

    fn generate(
        &self,
        _route: &str,
        _content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        let title = metadata.get("title").cloned().unwrap_or_default();
        Ok(format!("<title>{}</title>", title))
    }

    fn clone_box(&self) -> Box<dyn Generator> {
        Box::new(self.clone())
    }
}
