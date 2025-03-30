use std::any::Any;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;

pub trait Generator: Debug + Send + Sync {
    fn name(&self) -> &'static str;

    fn generate(
        &self,
        route: &str,
        content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>>;

    // Add a method for cloning trait objects
    fn clone_box(&self) -> Box<dyn Generator>;

    // Add a method for downcasting support
    fn as_any(&self) -> &dyn Any;
}

// Enable cloning for trait objects
impl Clone for Box<dyn Generator> {
    fn clone(&self) -> Self {
        self.as_ref().clone_box()
    }
}
