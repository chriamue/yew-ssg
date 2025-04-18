//! This module defines the core `Processor` trait which is the foundation
//! of the HTML processing pipeline in yew-ssg.
//!
//! Processors transform HTML content by applying various operations such as:
//! - Replacing template variables with actual content
//! - Injecting generator outputs into appropriate locations
//! - Modifying HTML attributes based on metadata
//! - Transforming content structure as needed

use std::collections::HashMap;
use std::error::Error;
use std::fmt::Debug;

/// The `Processor` trait defines how HTML content is transformed during the
/// static site generation process.
///
/// Processors are executed in sequence, with each one taking the output from
/// the previous processor. This allows for a composable processing pipeline
/// where each processor focuses on a specific transformation task.
///
/// # Examples
///
/// A simple processor that replaces placeholder text:
///
/// ```
/// use yew_ssg::processor::Processor;
/// use std::collections::HashMap;
/// use std::error::Error;
///
/// #[derive(Debug, Clone)]
/// struct PlaceholderProcessor;
///
/// impl Processor for PlaceholderProcessor {
///     fn name(&self) -> &'static str {
///         "placeholder_processor"
///     }
///
///     fn process(
///         &self,
///         html: &str,
///         _metadata: &HashMap<String, String>,
///         _generator_outputs: &HashMap<String, String>,
///         _content: &str,
///     ) -> Result<String, Box<dyn Error>> {
///         Ok(html.replace("__PLACEHOLDER__", "Replaced Content"))
///     }
///
///     fn clone_box(&self) -> Box<dyn Processor> {
///         Box::new(self.clone())
///     }
/// }
/// ```
pub trait Processor: Debug + Send + Sync {
    /// Returns a unique name that identifies this processor.
    ///
    /// The name is used for logging and debugging purposes.
    /// It should be a constant string that uniquely identifies the processor.
    fn name(&self) -> &'static str;

    /// Processes the HTML with available context and returns the transformed HTML.
    ///
    /// This is the core method that performs the actual transformation on the HTML content.
    ///
    /// # Arguments
    ///
    /// * `html` - The HTML string to process
    /// * `metadata` - Key-value pairs of metadata associated with the current route
    /// * `generator_outputs` - Key-value pairs of generator outputs, where keys are generator names
    /// * `content` - The raw content of the page (usually the rendered Yew component)
    ///
    /// # Returns
    ///
    /// * `Result<String, Box<dyn Error>>` - The processed HTML or an error
    fn process(
        &self,
        html: &str,
        metadata: &HashMap<String, String>,
        generator_outputs: &HashMap<String, String>,
        content: &str,
    ) -> Result<String, Box<dyn Error>>;

    /// Creates a boxed clone of this processor.
    ///
    /// This method is used to enable cloning of trait objects.
    /// Implementations should typically return `Box::new(self.clone())`.
    fn clone_box(&self) -> Box<dyn Processor>;
}

// Enable cloning for trait objects
impl Clone for Box<dyn Processor> {
    fn clone(&self) -> Self {
        self.as_ref().clone_box()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::fmt;

    /// A mock processor for testing
    #[derive(Clone)]
    pub struct MockProcessor {
        name: &'static str,
        transform_fn: fn(&str) -> String,
    }

    impl fmt::Debug for MockProcessor {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("MockProcessor")
                .field("name", &self.name)
                .finish()
        }
    }

    impl MockProcessor {
        pub fn new(name: &'static str, transform_fn: fn(&str) -> String) -> Self {
            Self { name, transform_fn }
        }
    }

    impl Processor for MockProcessor {
        fn name(&self) -> &'static str {
            self.name
        }

        fn process(
            &self,
            html: &str,
            _metadata: &HashMap<String, String>,
            _generator_outputs: &HashMap<String, String>,
            _content: &str,
        ) -> Result<String, Box<dyn Error>> {
            Ok((self.transform_fn)(html))
        }

        fn clone_box(&self) -> Box<dyn Processor> {
            Box::new(self.clone())
        }
    }

    /// Test function to verify Processor trait compliance
    pub fn test_processor_compliance<P: Processor + Clone>(processor: P) {
        // Test name returns a value
        let name = processor.name();
        assert!(!name.is_empty(), "Processor name should not be empty");

        // Test clone_box returns a valid clone
        let cloned: Box<dyn Processor> = processor.clone_box();
        assert_eq!(
            cloned.name(),
            name,
            "Cloned processor should have same name"
        );

        // Test process with minimal input
        let result = processor.process("<div>test</div>", &HashMap::new(), &HashMap::new(), "");
        assert!(result.is_ok(), "Basic process call should not fail");
    }

    #[test]
    fn test_mock_processor() {
        let mock = MockProcessor::new("test_processor", |html| {
            format!("<processed>{}</processed>", html)
        });

        // Test name
        assert_eq!(mock.name(), "test_processor");

        // Test processing
        let result = mock
            .process("<div>test</div>", &HashMap::new(), &HashMap::new(), "")
            .unwrap();
        assert_eq!(result, "<processed><div>test</div></processed>");

        // Run compliance test
        test_processor_compliance(mock);
    }
}
