pub mod config;
pub mod generator;
pub mod generator_collection;
pub mod generators;
pub mod static_site_generator;

pub use config::SsgConfig;
pub use config::SsgConfigBuilder;

pub use static_site_generator::StaticSiteGenerator;

pub mod prelude {
    // Configuration
    pub use crate::config::{SsgConfig, SsgConfigBuilder};

    // Core traits and components
    pub use crate::generator::Generator;
    pub use crate::generator_collection::{GeneratorCollection, GeneratorInfoIterator};

    // Generator implementations
    pub use crate::generators::{
        MetaTagGenerator, OpenGraphGenerator, RobotsMetaGenerator, TitleGenerator,
        TwitterCardGenerator,
    };

    // Static site generator
    pub use crate::static_site_generator::StaticSiteGenerator;
}
