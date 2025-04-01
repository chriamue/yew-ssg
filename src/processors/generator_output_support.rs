pub trait GeneratorOutputSupport {
    /// Returns all variable/attribute names this generator can provide values for
    fn supported_outputs(&self) -> Vec<&'static str>;

    /// Returns if this generator supports a specific output key
    fn supports_output(&self, key: &str) -> bool {
        self.supported_outputs().contains(&key)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::generator::tests::MockGenerator;
    use crate::generator::Generator;

    impl GeneratorOutputSupport for MockGenerator {
        fn supported_outputs(&self) -> Vec<&'static str> {
            let mut outputs = Vec::new();
            outputs.push(self.name());
            outputs.extend_from_slice(&self.supported_keys);
            outputs
        }
    }

    /// Test function to verify GeneratorOutputSupport compliance
    pub fn test_generator_output_support<G: GeneratorOutputSupport>(generator: G) {
        let outputs = generator.supported_outputs();
        assert!(
            !outputs.is_empty(),
            "Generator should support at least one output"
        );

        // Test that supports_output works correctly
        for output in &outputs {
            assert!(
                generator.supports_output(output),
                "Generator should support its declared outputs"
            );
        }

        assert!(
            !generator.supports_output("definitely_not_supported_output_xyz"),
            "Generator should not support random outputs"
        );
    }

    #[test]
    fn test_mock_generator_output_support() {
        let mock = MockGenerator::new("test_gen", vec!["key1", "key2"]);

        // Verify output support
        let outputs = mock.supported_outputs();
        assert_eq!(outputs.len(), 3); // name + 2 keys
        assert!(outputs.contains(&"test_gen"));
        assert!(outputs.contains(&"key1"));
        assert!(outputs.contains(&"key2"));

        // Run the compliance test
        test_generator_output_support(mock);
    }
}
