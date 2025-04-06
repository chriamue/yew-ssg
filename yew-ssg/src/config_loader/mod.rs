mod loader;
mod loaders;
mod model;

pub use loader::{load_config, ConfigLoader};
pub use loaders::{JsonLoader, YamlLoader};
pub use model::*;

// Re-export from the main config module
pub use super::config::{RouteParams, SsgConfig, SsgConfigBuilder};

#[cfg(test)]
mod tests;
