use crate::config::SsgConfig;
use std::error::Error;
use std::path::Path;

/// Trait for loading SSG configuration from different sources/formats
pub trait ConfigLoader {
    /// Load configuration from a file path
    fn load_from_path<P: AsRef<Path>>(path: P) -> Result<SsgConfig, Box<dyn Error>>;

    /// Load configuration from a string
    fn load_from_str(content: &str) -> Result<SsgConfig, Box<dyn Error>>;

    /// Get the file extensions this loader supports
    fn supported_extensions() -> Vec<&'static str>;
}

/// Load configuration from a file, automatically selecting the appropriate loader
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<SsgConfig, Box<dyn Error>> {
    use crate::config_loader::loaders::{JsonLoader, YamlLoader};

    let path = path.as_ref();
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| "File has no extension".to_string())?
        .to_lowercase();

    // Try to match a loader based on file extension
    match extension.as_str() {
        ext if YamlLoader::supported_extensions().contains(&ext) => {
            YamlLoader::load_from_path(path)
        }
        ext if JsonLoader::supported_extensions().contains(&ext) => {
            JsonLoader::load_from_path(path)
        }
        _ => Err(format!("Unsupported configuration file extension: {}", extension).into()),
    }
}
