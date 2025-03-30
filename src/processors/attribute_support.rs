pub trait AttributeSupport {
    /// Returns a list of supported attributes for this generator.
    fn attributes(&self) -> Vec<&'static str>;
}
