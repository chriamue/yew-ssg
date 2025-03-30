pub trait TemplateVariableSupport {
    /// Returns a list of supported template variables for this generator.
    fn template_variables(&self) -> Vec<&'static str>;
}
