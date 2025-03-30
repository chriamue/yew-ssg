use crate::generator::Generator;
use crate::generator_collection::GeneratorCollection;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct SsgConfig {
    pub output_dir: PathBuf,
    pub template_path: Option<PathBuf>,
    pub default_template: String,
    pub global_metadata: HashMap<String, String>,
    pub route_metadata: HashMap<String, HashMap<String, String>>,
    pub generators: GeneratorCollection,
}

impl SsgConfig {
    /// Get combined metadata for a specific route, merging global and route-specific metadata.
    /// Route-specific metadata takes precedence.
    pub fn get_metadata_for_route(&self, route_path: &str) -> HashMap<String, String> {
        let mut metadata = self.global_metadata.clone();

        if let Some(route_specific) = self.route_metadata.get(route_path) {
            // Route-specific metadata overrides global metadata
            metadata.extend(route_specific.clone());
        }

        metadata
    }
}

impl Default for SsgConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("dist"),
            template_path: None,
            default_template: String::new(),
            global_metadata: HashMap::new(),
            route_metadata: HashMap::new(),
            generators: GeneratorCollection::new(),
        }
    }
}

pub struct SsgConfigBuilder {
    pub config: SsgConfig,
}

// Ensure the method is INSIDE this block
impl SsgConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: SsgConfig::default(),
        }
    }

    pub fn output_dir<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.output_dir = path.into();
        self
    }

    pub fn template<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.template_path = Some(path.into());
        // Clear default_template if a path is provided? Optional decision.
        // self.config.default_template = String::new();
        self
    }

    // Add a method to set the default template string directly
    pub fn default_template_string(mut self, template_content: String) -> Self {
        self.config.default_template = template_content;
        // Clear template_path if string content is provided? Optional decision.
        // self.config.template_path = None;
        self
    }

    pub fn global_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.config.global_metadata = metadata;
        self
    }

    pub fn route_metadata(mut self, path: &str, metadata: HashMap<String, String>) -> Self {
        self.config
            .route_metadata
            .insert(path.to_string(), metadata);
        self
    }

    pub fn add_generator<G: Generator + 'static>(mut self, generator: G) -> Self {
        self.config.generators.add(generator);
        self
    }

    pub fn build(self) -> SsgConfig {
        self.config
    }
}
