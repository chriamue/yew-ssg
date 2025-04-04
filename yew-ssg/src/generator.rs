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

#[cfg(test)]
pub mod tests {
    use super::*;

    /// A simple mock generator for testing trait implementations
    #[derive(Debug, Clone)]
    pub struct MockGenerator {
        pub name: String,
        pub supported_keys: Vec<&'static str>,
    }

    impl MockGenerator {
        pub fn new(name: &str, supported_keys: Vec<&'static str>) -> Self {
            Self {
                name: name.to_string(),
                supported_keys,
            }
        }
    }

    impl Generator for MockGenerator {
        fn name(&self) -> &'static str {
            Box::leak(self.name.clone().into_boxed_str())
        }

        fn generate(
            &self,
            key: &str,
            _route: &str,
            _content: &str,
            _metadata: &HashMap<String, String>,
        ) -> Result<String, Box<dyn Error>> {
            if key == self.name() || self.supported_keys.contains(&key) {
                Ok(format!(
                    "<div>Generated content for key '{}' by {}</div>",
                    key, self.name
                ))
            } else {
                Err(format!("Key '{}' not supported by generator '{}'", key, self.name).into())
            }
        }

        fn clone_box(&self) -> Box<dyn Generator> {
            Box::new(self.clone())
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    /// Standard test for Generator trait implementations
    pub fn test_generator_trait_compliance<G: Generator + Clone>(generator: G) {
        // Test name() returns a &'static str
        let name = generator.name();
        assert!(!name.is_empty(), "Generator name should not be empty");

        // Test clone_box() returns a valid clone
        let cloned: Box<dyn Generator> = generator.clone_box();
        assert_eq!(
            cloned.name(),
            name,
            "Cloned generator should have same name"
        );

        // Test as_any() returns a valid reference
        let _any_ref = generator.as_any();
        // We just verify it doesn't panic
    }

    #[test]
    fn test_mock_generator() {
        let mock = MockGenerator::new("test_gen", vec!["key1", "key2"]);

        // Verify basic behavior
        assert_eq!(mock.name(), "test_gen");

        // Test successful generation
        let result = mock.generate("key1", "/test", "", &HashMap::new()).unwrap();
        assert_eq!(
            result,
            "<div>Generated content for key 'key1' by test_gen</div>"
        );

        // Test error case
        let err = mock.generate("unknown", "/test", "", &HashMap::new());
        assert!(err.is_err());

        // Run the compliance test on our mock
        test_generator_trait_compliance(mock.clone());
    }
}
