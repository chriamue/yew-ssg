use crate::config::{SsgConfig, SsgConfigBuilder};
use crate::config_loader::RouteParams;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Main configuration structure for the static site generator
/// Used as the intermediate format between file formats and SsgConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsgFileConfig {
    /// Basic configuration options
    #[serde(default)]
    pub general: GeneralConfig,

    /// Global metadata applied to all pages
    #[serde(default)]
    pub global_metadata: HashMap<String, String>,

    /// Route-specific configurations
    #[serde(default)]
    pub routes: Vec<RouteConfig>,

    /// Parameter-based route configurations
    #[serde(default)]
    pub parameterized_routes: Vec<ParameterizedRouteConfig>,
}

/// General configuration options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeneralConfig {
    /// Output directory for generated files
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,

    /// Path to HTML template file
    pub template_path: Option<PathBuf>,

    /// Default template content as a string if no file is used
    #[serde(default)]
    pub default_template: String,

    /// Base directory for asset files
    pub assets_base_dir: Option<String>,

    /// Base directory for JSON-LD files
    pub json_ld_base_dir: Option<String>,

    /// Default site name
    #[serde(default = "default_site_name")]
    pub site_name: String,

    /// Default page title format
    #[serde(default = "default_title_format")]
    pub title_format: String,

    /// Default image for social media shares
    pub default_image: Option<String>,

    /// Twitter handle (without @)
    pub twitter_handle: Option<String>,

    /// Either a boolean for all languages or a comma-separated list of language codes
    /// that should have canonical URLs pointing to the default language.
    /// Examples:
    ///   canonical_to_default_langs: true     // All translated pages point to default language
    ///   canonical_to_default_langs: "de,es"  // Only German and Spanish pages point to default language
    ///   canonical_to_default_langs: false    // All languages have their own canonical URL (default)
    #[serde(default)]
    pub canonical_to_default_langs: Option<CanonicalBehavior>,

    /// Default language code used for canonical URLs (default: "en")
    #[serde(default = "default_language")]
    pub default_language: String,
}

/// Configuration for a specific route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteConfig {
    /// Route path (e.g., "/about")
    pub path: String,

    /// Metadata specific to this route
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Configuration for a parameterized route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterizedRouteConfig {
    /// Route pattern (e.g., "/crate/:id")
    pub pattern: String,

    /// Parameters and their valid values
    pub parameters: Vec<ParameterDefinition>,

    /// Parameter-specific configurations
    #[serde(default)]
    pub variants: Vec<ParameterVariant>,

    /// Metadata common to all parameter combinations
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

/// Definition of a parameter and its valid values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    /// Parameter name
    pub name: String,

    /// Valid values for this parameter
    pub values: Vec<String>,
}

/// Configuration for a specific parameter value combination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterVariant {
    /// Parameter values for this variant
    pub values: HashMap<String, String>,

    /// Metadata specific to this parameter combination
    pub metadata: HashMap<String, String>,
}

// Default functions for serde
fn default_output_dir() -> PathBuf {
    PathBuf::from("dist")
}

fn default_site_name() -> String {
    "My Site".to_string()
}

fn default_title_format() -> String {
    "{title} | {site_name}".to_string()
}

fn default_language() -> String {
    "en".to_string()
}

impl SsgFileConfig {
    /// Convert file config to SsgConfig
    pub fn to_ssg_config(&self) -> SsgConfig {
        // Start with a builder
        let mut builder = SsgConfigBuilder::new().output_dir(self.general.output_dir.clone());

        // Set template path if provided
        if let Some(template_path) = &self.general.template_path {
            builder = builder.template(template_path.clone());
        }

        // Set default template content if provided
        if !self.general.default_template.is_empty() {
            builder = builder.default_template_string(self.general.default_template.clone());
        }

        // Set assets base directory, preferring the new field but falling back to the old one
        if let Some(assets_dir) = &self.general.assets_base_dir {
            builder = builder.assets_base_dir(assets_dir);
        } else if let Some(json_ld_dir) = &self.general.json_ld_base_dir {
            // Fallback to json_ld_base_dir for backward compatibility
            builder = builder.assets_base_dir(json_ld_dir);
        }

        // Set global metadata
        let mut global_metadata = self.global_metadata.clone();

        // Add site name to global metadata if not present
        if !global_metadata.contains_key("site_name") {
            global_metadata.insert("site_name".to_string(), self.general.site_name.clone());
        }

        // Set default image if present
        if let Some(default_image) = &self.general.default_image {
            global_metadata.insert("default_image".to_string(), default_image.clone());
        }

        // Set Twitter handle if present
        if let Some(twitter_handle) = &self.general.twitter_handle {
            global_metadata.insert("twitter_site".to_string(), format!("@{}", twitter_handle));
        }

        builder = builder.global_metadata(global_metadata);

        // Process standard routes
        for route in &self.routes {
            builder = builder.route_metadata(&route.path, route.metadata.clone());
        }

        // Process parameterized routes base metadata
        for param_route in &self.parameterized_routes {
            // First add the basic route metadata
            builder = builder.route_metadata(&param_route.pattern, param_route.metadata.clone());

            // Now set up the route params
            let mut route_params = RouteParams::new();

            // Add parameter definitions
            for param_def in &param_route.parameters {
                route_params.add_param(&param_def.name, &param_def.values);
            }

            // Add parameter variants with their specific metadata
            for variant in &param_route.variants {
                // For each parameter in this variant
                for (param_name, param_value) in &variant.values {
                    route_params.add_param_metadata(
                        param_name,
                        param_value,
                        variant.metadata.clone(),
                    );
                }
            }

            // Add to the builder
            builder = builder.route_params(&param_route.pattern, route_params);
        }

        // Build the final config
        builder.build()
    }
}

/// Represents behavior for canonical URLs in translations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum CanonicalBehavior {
    /// Apply to all languages or none
    Boolean(bool),
    /// Apply to specific languages
    Languages(String),
}

impl Default for CanonicalBehavior {
    fn default() -> Self {
        CanonicalBehavior::Boolean(false)
    }
}
