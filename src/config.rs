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
    config: SsgConfig,
}

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
