pub mod config;
pub mod config_loader;
pub mod generator;
pub mod generator_collection;
pub mod generators;
pub mod processor;
pub mod processor_collection;
pub mod processors;
pub mod static_site_generator;

pub use config::SsgConfig;
pub use config::SsgConfigBuilder;

pub use static_site_generator::StaticSiteGenerator;

pub mod prelude {
    // Configuration
    pub use crate::config::{SsgConfig, SsgConfigBuilder};

    // Core traits and components
    pub use crate::generator::Generator;
    pub use crate::generator_collection::GeneratorCollection;
    pub use crate::processor::Processor;
    pub use crate::processor_collection::ProcessorCollection;

    // Generator implementations
    pub use crate::generators::{
        MetaTagGenerator, OpenGraphGenerator, RobotsMetaGenerator, TitleGenerator,
        TwitterCardGenerator,
    };

    // Processor implementations
    pub use crate::processors::{AttributeProcessor, TemplateVariableProcessor};

    // Static site generator
    pub use crate::static_site_generator::StaticSiteGenerator;
}
