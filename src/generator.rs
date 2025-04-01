use std::any::Any;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;

pub trait Generator: Debug + Send + Sync {
    /// Returns the unique name of this generator
    fn name(&self) -> &'static str;

    /// Generates HTML content based on the requested key, route, content, and metadata
    ///
    /// # Arguments
    ///
    /// * `key` - The specific output key that's being requested (e.g. "meta_tags", "og:title")
    /// * `route` - The current route path
    /// * `content` - The rendered content
    /// * `metadata` - Key-value pairs of metadata for the current page
    ///
    /// # Returns
    ///
    /// Generated HTML content for the requested key or an error
    fn generate(
        &self,
        key: &str,
        route: &str,
        content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>>;

    /// Creates a boxed clone of this generator
    fn clone_box(&self) -> Box<dyn Generator>;

    /// Returns a reference to self as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

// Enable cloning for trait objects
impl Clone for Box<dyn Generator> {
    fn clone(&self) -> Self {
        self.as_ref().clone_box()
    }
}
