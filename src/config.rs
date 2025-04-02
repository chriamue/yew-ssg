use crate::generator::Generator;
use crate::generator_collection::GeneratorCollection;
use crate::generators::{
    MetaTagGenerator, OpenGraphGenerator, RobotsMetaGenerator, TitleGenerator, TwitterCardGenerator,
};
use crate::processor::Processor;
use crate::processor_collection::ProcessorCollection;
use crate::processors::{AttributeProcessor, TemplateVariableProcessor};
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
    pub processors: ProcessorCollection,
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

    /// Add default generators if none have been added
    pub fn with_default_generators(mut self) -> Self {
        if self.generators.is_empty() {
            // Title generator
            self.generators.add(TitleGenerator);

            // Meta tags generator
            self.generators.add(MetaTagGenerator {
                default_description: "A website created with yew-ssg.".to_string(),
                default_keywords: vec!["yew".to_string(), "rust".to_string(), "ssg".to_string()],
            });

            // Open Graph generator
            self.generators.add(OpenGraphGenerator {
                site_name: "Yew SSG Site".to_string(),
                default_image: "/images/default-cover.jpg".to_string(),
            });

            // Twitter Card generator
            self.generators.add(TwitterCardGenerator {
                twitter_site: None,
                default_card_type: "summary".to_string(),
            });

            // Robots meta generator
            self.generators.add(RobotsMetaGenerator {
                default_robots: "index, follow".to_string(),
            });
        }
        self
    }

    /// Add default processors if none have been added
    pub fn with_default_processors(mut self) -> Self {
        if self.processors.is_empty() {
            // Template variable processor for {{var}} syntax
            let template_processor = TemplateVariableProcessor::new();
            self.processors.add(template_processor);

            // Attribute processor for attribute-based content
            let attribute_processor = AttributeProcessor::new("data-ssg", None);
            self.processors.add(attribute_processor);
        }
        self
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
            processors: ProcessorCollection::new(),
        }
        // Don't add defaults here to allow more control
    }
}

pub struct SsgConfigBuilder {
    pub config: SsgConfig,
    pub use_default_generators: bool,
    pub use_default_processors: bool,
}

impl SsgConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: SsgConfig::default(),
            use_default_generators: true,
            use_default_processors: true,
        }
    }

    pub fn without_default_generators(mut self) -> Self {
        self.use_default_generators = false;
        self
    }

    pub fn without_default_processors(mut self) -> Self {
        self.use_default_processors = false;
        self
    }

    pub fn output_dir<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.output_dir = path.into();
        self
    }

    pub fn template<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.config.template_path = Some(path.into());
        self
    }

    pub fn default_template_string(mut self, template_content: String) -> Self {
        self.config.default_template = template_content;
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

    pub fn add_processor<P: Processor + 'static>(mut self, processor: P) -> Self {
        self.config.processors.add(processor);
        self
    }

    pub fn build(self) -> SsgConfig {
        let mut config = self.config;

        // Apply defaults if requested
        if self.use_default_generators {
            config = config.with_default_generators();
        }

        if self.use_default_processors {
            config = config.with_default_processors();
        }

        config
    }
}
