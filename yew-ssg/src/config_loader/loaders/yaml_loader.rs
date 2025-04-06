use crate::config_loader::model::SsgFileConfig;
use crate::config_loader::{ConfigLoader, SsgConfig};
use std::error::Error;
use std::path::Path;

/// YAML implementation of ConfigLoader trait
pub struct YamlLoader;

impl ConfigLoader for YamlLoader {
    fn load_from_path<P: AsRef<Path>>(path: P) -> Result<SsgConfig, Box<dyn Error>> {
        let content = std::fs::read_to_string(path)?;
        Self::load_from_str(&content)
    }

    fn load_from_str(content: &str) -> Result<SsgConfig, Box<dyn Error>> {
        let file_config: SsgFileConfig = serde_yaml::from_str(content)?;
        Ok(file_config.to_ssg_config())
    }

    fn supported_extensions() -> Vec<&'static str> {
        vec!["yaml", "yml"]
    }
}
