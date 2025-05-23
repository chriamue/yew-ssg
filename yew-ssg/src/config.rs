use crate::generator::Generator;
use crate::generator_collection::GeneratorCollection;
use crate::generators::{
    CanonicalLinkGenerator, JsonLdGenerator, MetaTagGenerator, OpenGraphGenerator,
    RobotsMetaGenerator, TitleGenerator, TwitterCardGenerator,
};
use crate::processor::Processor;
use crate::processor_collection::ProcessorCollection;
use crate::processors::{AttributeProcessor, TemplateVariableProcessor};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Defines parameter names and their valid values for routes with path parameters
#[derive(Debug, Clone, Default)]
pub struct RouteParams {
    /// Maps parameter names to sets of allowed values
    pub param_values: HashMap<String, HashSet<String>>,

    /// Stores metadata for specific parameter values
    /// The key is formatted as "param_name=value" (e.g., "id=yew-ssg")
    pub param_metadata: HashMap<String, HashMap<String, String>>,
}

impl RouteParams {
    /// Creates a new empty RouteParams
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a parameter with its valid values
    pub fn add_param<I, S>(&mut self, name: &str, values: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let values_set = values.into_iter().map(|s| s.into()).collect();
        self.param_values.insert(name.to_string(), values_set);
        self
    }

    /// Adds metadata for a specific parameter value
    pub fn add_param_metadata(
        &mut self,
        param_name: &str,
        param_value: &str,
        metadata: HashMap<String, String>,
    ) -> &mut Self {
        let key = format!("{}={}", param_name, param_value);
        self.param_metadata.insert(key, metadata);
        self
    }

    /// Gets metadata for a specific parameter value
    pub fn get_param_metadata(
        &self,
        param_name: &str,
        param_value: &str,
    ) -> Option<&HashMap<String, String>> {
        let key = format!("{}={}", param_name, param_value);
        self.param_metadata.get(&key)
    }

    /// Checks if a parameter value is valid according to the defined constraints
    pub fn is_valid_param_value(&self, param_name: &str, value: &str) -> bool {
        if let Some(values) = self.param_values.get(param_name) {
            values.contains(value)
        } else {
            false
        }
    }

    /// Generates all possible parameter combinations based on defined parameter values
    pub fn generate_param_combinations(&self) -> Vec<HashMap<String, String>> {
        if self.param_values.is_empty() {
            return vec![];
        }

        let mut result = Vec::new();
        self.generate_combinations_recursive(
            &mut result,
            &mut HashMap::new(),
            self.param_values.keys().collect::<Vec<_>>().as_slice(),
        );
        result
    }

    /// Helper method for recursively generating parameter combinations
    fn generate_combinations_recursive(
        &self,
        result: &mut Vec<HashMap<String, String>>,
        current: &mut HashMap<String, String>,
        params: &[&String],
    ) {
        if params.is_empty() {
            result.push(current.clone());
            return;
        }

        let param_name = params[0];
        let remaining_params = &params[1..];

        if let Some(values) = self.param_values.get(param_name) {
            for value in values {
                current.insert(param_name.clone(), value.clone());
                self.generate_combinations_recursive(result, current, remaining_params);
            }
            current.remove(param_name);
        } else {
            self.generate_combinations_recursive(result, current, remaining_params);
        }
    }
}

#[derive(Debug, Clone)]
pub struct SsgConfig {
    pub output_dir: PathBuf,
    pub template_path: Option<PathBuf>,
    pub default_template: String,
    pub global_metadata: HashMap<String, String>,
    pub route_metadata: HashMap<String, HashMap<String, String>>,
    pub generators: GeneratorCollection,
    pub processors: ProcessorCollection,
    /// Parameter definitions for routes with dynamic segments
    pub route_params: HashMap<String, RouteParams>,
    /// Base directory for asset files (images, JSON-LD, etc.)
    pub assets_base_dir: Option<String>,
}

impl SsgConfig {
    /// Get combined metadata for a specific route, merging global and route-specific metadata.
    /// Route-specific metadata takes precedence. Supports parent path inheritance.
    pub fn get_metadata_for_route(&self, route_path: &str) -> HashMap<String, String> {
        let mut metadata = self.global_metadata.clone();

        // Collect all parent paths (including the route itself)
        let mut paths = Vec::new();
        let mut path = route_path.trim_end_matches('/').to_string();
        if path.is_empty() {
            path = "/".to_string();
        }
        loop {
            // Always add a trailing slash except for root
            let normalized = if path == "/" {
                "/".to_string()
            } else {
                format!("{}/", path.trim_end_matches('/'))
            };
            paths.push(normalized);
            if path == "/" {
                break;
            }
            // Remove last segment
            if let Some(pos) = path.rfind('/') {
                if pos == 0 {
                    path = "/".to_string();
                } else {
                    path = path[..pos].to_string();
                }
            } else {
                break;
            }
        }
        // Reverse so that more general paths are merged first
        paths.reverse();

        for p in paths {
            if let Some(route_specific) = self.route_metadata.get(&p) {
                metadata.extend(route_specific.clone());
            }
        }

        // Also check for exact match (without trailing slash)
        if let Some(route_specific) = self.route_metadata.get(route_path) {
            metadata.extend(route_specific.clone());
        }

        metadata.insert("path".to_string(), route_path.to_string());

        metadata
    }

    /// Get metadata for a parameterized route, including parameter-specific metadata
    pub fn get_metadata_for_parameterized_route(
        &self,
        route_pattern: &str,
        params: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut metadata = self.get_metadata_for_route(route_pattern);

        // Add parameter values to metadata
        for (param_name, param_value) in params {
            metadata.insert(format!("param_{}", param_name), param_value.clone());

            // Add any parameter-specific metadata if available
            if let Some(route_params) = self.route_params.get(route_pattern) {
                if let Some(param_metadata) =
                    route_params.get_param_metadata(param_name, param_value)
                {
                    metadata.extend(param_metadata.clone());
                }
            }
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

            // Canonical link generator
            self.generators.add(CanonicalLinkGenerator::new());

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

            // JSON-LD generator (now with assets base directory)
            let mut json_ld_generator = JsonLdGenerator::new();

            // Use either the direct assets_base_dir field, or check global_metadata for
            // either "assets_base_dir" or legacy "json_ld_base_dir" (for backward compatibility)
            let assets_dir = self.assets_base_dir.clone().or_else(|| {
                self.global_metadata
                    .get("assets_base_dir")
                    .or_else(|| self.global_metadata.get("json_ld_base_dir"))
                    .cloned()
            });

            if let Some(dir) = assets_dir {
                json_ld_generator.json_ld_base_dir = Some(dir);
            }

            self.generators.add(json_ld_generator);
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
            route_params: HashMap::new(),
            assets_base_dir: None,
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

    /// Define parameters for a specific route pattern with dynamic segments
    pub fn route_params(mut self, route_pattern: &str, params: RouteParams) -> Self {
        self.config
            .route_params
            .insert(route_pattern.to_string(), params);
        self
    }

    pub fn assets_base_dir(mut self, path: &str) -> Self {
        self.config.assets_base_dir = Some(path.to_string());
        self
    }

    /// Add a parameter with values to a route pattern
    pub fn add_route_param<I, S>(mut self, route_pattern: &str, param_name: &str, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let route_params = self
            .config
            .route_params
            .entry(route_pattern.to_string())
            .or_insert_with(RouteParams::new);

        route_params.add_param(param_name, values);
        self
    }

    /// Add metadata for a specific parameter value
    pub fn add_param_metadata(
        mut self,
        route_pattern: &str,
        param_name: &str,
        param_value: &str,
        metadata: HashMap<String, String>,
    ) -> Self {
        let route_params = self
            .config
            .route_params
            .entry(route_pattern.to_string())
            .or_insert_with(RouteParams::new);

        route_params.add_param_metadata(param_name, param_value, metadata);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> SsgConfig {
        let mut global = HashMap::new();
        global.insert("site".to_string(), "GlobalSite".to_string());
        global.insert("lang".to_string(), "en".to_string());

        let mut route_metadata = HashMap::new();
        route_metadata.insert(
            "/".to_string(),
            HashMap::from([
                ("site".to_string(), "RootSite".to_string()),
                ("root_only".to_string(), "yes".to_string()),
            ]),
        );
        route_metadata.insert(
            "/de/".to_string(),
            HashMap::from([
                ("lang".to_string(), "de".to_string()),
                ("site".to_string(), "GermanSite".to_string()),
            ]),
        );
        route_metadata.insert(
            "/de/404".to_string(),
            HashMap::from([("title".to_string(), "404 Deutsch".to_string())]),
        );
        route_metadata.insert(
            "/about".to_string(),
            HashMap::from([("title".to_string(), "About Us".to_string())]),
        );

        SsgConfig {
            output_dir: PathBuf::from("dist"),
            template_path: None,
            default_template: String::new(),
            global_metadata: global,
            route_metadata,
            generators: GeneratorCollection::new(),
            processors: ProcessorCollection::new(),
            route_params: HashMap::new(),
            assets_base_dir: None,
        }
    }

    #[test]
    fn test_root_metadata() {
        let config = make_config();
        let meta = config.get_metadata_for_route("/");
        assert_eq!(meta.get("site").unwrap(), "RootSite");
        assert_eq!(meta.get("root_only").unwrap(), "yes");
        assert_eq!(meta.get("lang").unwrap(), "en");
    }

    #[test]
    fn test_about_metadata() {
        let config = make_config();
        let meta = config.get_metadata_for_route("/about");
        assert_eq!(meta.get("site").unwrap(), "RootSite");
        assert_eq!(meta.get("title").unwrap(), "About Us");
        assert_eq!(meta.get("lang").unwrap(), "en");
    }

    #[test]
    fn test_de_metadata() {
        let config = make_config();
        let meta = config.get_metadata_for_route("/de/");
        assert_eq!(meta.get("site").unwrap(), "GermanSite");
        assert_eq!(meta.get("lang").unwrap(), "de");
        assert_eq!(meta.get("root_only").unwrap(), "yes");
    }

    #[test]
    fn test_de_404_metadata() {
        let config = make_config();
        let meta = config.get_metadata_for_route("/de/404");
        // Should inherit from /de/ and /
        assert_eq!(meta.get("site").unwrap(), "GermanSite");
        assert_eq!(meta.get("lang").unwrap(), "de");
        assert_eq!(meta.get("title").unwrap(), "404 Deutsch");
        assert_eq!(meta.get("root_only").unwrap(), "yes");
    }

    #[test]
    fn test_nonexistent_route_metadata() {
        let config = make_config();
        let meta = config.get_metadata_for_route("/foo/bar");
        // Should inherit from global and root only
        assert_eq!(meta.get("site").unwrap(), "RootSite");
        assert_eq!(meta.get("lang").unwrap(), "en");
        assert_eq!(meta.get("root_only").unwrap(), "yes");
        assert!(meta.get("title").is_none());
    }
}
