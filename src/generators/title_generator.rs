use crate::generator::Generator;
use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct TitleGenerator;

#[async_trait]
impl Generator for TitleGenerator {
    fn name(&self) -> &'static str {
        "title"
    }

    async fn generate(
        &self,
        _route: &str,
        _content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        let title = metadata.get("title").cloned().unwrap_or_default();
        Ok(format!("<title>{}</title>", title))
    }

    fn box_clone(&self) -> Box<dyn Generator + Send + Sync> {
        Box::new(self.clone())
    }
}
