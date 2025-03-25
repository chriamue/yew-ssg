use crate::generator::Generator;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct SsgConfig {
    pub output_dir: PathBuf,
    pub template_path: Option<PathBuf>,
    pub default_template: String,
    pub global_metadata: HashMap<String, String>,
    pub route_metadata: HashMap<String, HashMap<String, String>>,
    pub generators: Vec<Box<dyn Generator + Send + Sync>>,
}

impl Clone for SsgConfig {
    fn clone(&self) -> Self {
        Self {
            output_dir: self.output_dir.clone(),
            template_path: self.template_path.clone(),
            default_template: self.default_template.clone(),
            global_metadata: self.global_metadata.clone(),
            route_metadata: self.route_metadata.clone(),
            generators: self.generators.iter().map(|g| g.box_clone()).collect(),
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

    pub fn add_generator<G: Generator + Send + Sync + 'static>(mut self, generator: G) -> Self {
        self.config.generators.push(Box::new(generator));
        self
    }

    pub fn build(self) -> SsgConfig {
        self.config
    }
}
