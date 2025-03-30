use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;

pub trait Processor: Debug + Send + Sync {
    /// Name of the processor for identification
    fn name(&self) -> &'static str;

    /// Process the HTML with available context
    fn process(
        &self,
        html: &str,
        metadata: &HashMap<String, String>,
        generator_outputs: &HashMap<String, String>,
        content: &str,
    ) -> Result<String, Box<dyn Error>>;

    /// Clone this processor
    fn clone_box(&self) -> Box<dyn Processor>;
}

// Enable cloning for trait objects
impl Clone for Box<dyn Processor> {
    fn clone(&self) -> Self {
        self.as_ref().clone_box()
    }
}
