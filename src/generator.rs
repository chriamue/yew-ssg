use async_trait::async_trait;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;

#[async_trait]
pub trait Generator: Send + Sync + Debug {
    fn name(&self) -> &'static str;

    async fn generate(
        &self,
        route: &str,
        content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>>;

    // Add method to clone self
    fn box_clone(&self) -> Box<dyn Generator + Send + Sync>;
}
