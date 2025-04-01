pub trait GeneratorOutputSupport {
    /// Returns all variable/attribute names this generator can provide values for
    fn supported_outputs(&self) -> Vec<&'static str>;

    /// Returns if this generator supports a specific output key
    fn supports_output(&self, key: &str) -> bool {
        self.supported_outputs().contains(&key)
    }
}
