#[cfg(test)]
mod tests {
    use crate::config_loader::{
        loader::{load_config, ConfigLoader},
        loaders::{JsonLoader, YamlLoader},
        model::{
            GeneralConfig, ParameterDefinition, ParameterVariant, ParameterizedRouteConfig,
            RouteConfig, SsgFileConfig,
        },
    };
    use std::collections::HashMap;
    use std::error::Error;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use tempfile::NamedTempFile;

    // Helper function to create test file with content
    fn create_temp_file(content: &str, extension: &str) -> Result<NamedTempFile, Box<dyn Error>> {
        let mut temp_file = NamedTempFile::new()?;
        // Write content to file
        temp_file.write_all(content.as_bytes())?;

        // Rename to add extension
        let path = temp_file.path().to_owned();
        let new_path = path.with_extension(extension);
        std::fs::rename(&path, &new_path)?;

        Ok(temp_file)
    }

    #[test]
    fn test_yaml_loader_from_str() {
        let yaml_str = r#"
general:
  output_dir: "test_dist"
  site_name: "Test Site"
  default_template: "<html>{{ content }}</html>"

global_metadata:
  author: "Test Author"
  keywords: "test, yaml, config"

routes:
  - path: "/"
    metadata:
      title: "Home Page"
      description: "Welcome to the test site"

  - path: "/about"
    metadata:
      title: "About Us"
      description: "Learn about our project"

parameterized_routes:
  - pattern: "/blog/:id"
    parameters:
      - name: "id"
        values: ["post1", "post2"]
    metadata:
      section: "Blog"
    variants:
      - values:
          id: "post1"
        metadata:
          title: "Post 1"
          description: "First blog post"
      - values:
          id: "post2"
        metadata:
          title: "Post 2"
          description: "Second blog post"
"#;

        // Load config using the YAML loader
        let config = YamlLoader::load_from_str(yaml_str).unwrap();

        // Verify basic configuration
        assert_eq!(config.output_dir, PathBuf::from("test_dist"));
        assert_eq!(config.default_template, "<html>{{ content }}</html>");

        // Verify global metadata
        assert_eq!(config.global_metadata.get("author").unwrap(), "Test Author");
        assert_eq!(
            config.global_metadata.get("keywords").unwrap(),
            "test, yaml, config"
        );
        assert_eq!(
            config.global_metadata.get("site_name").unwrap(),
            "Test Site"
        );

        // Verify routes
        let home_meta = config.route_metadata.get("/").unwrap();
        assert_eq!(home_meta.get("title").unwrap(), "Home Page");
        assert_eq!(
            home_meta.get("description").unwrap(),
            "Welcome to the test site"
        );

        let about_meta = config.route_metadata.get("/about").unwrap();
        assert_eq!(about_meta.get("title").unwrap(), "About Us");

        // Verify parameterized routes
        let blog_params = config.route_params.get("/blog/:id").unwrap();

        // Check parameter values
        let id_values = blog_params.param_values.get("id").unwrap();
        assert!(id_values.contains("post1"));
        assert!(id_values.contains("post2"));

        // Check parameter metadata
        let post1_meta = blog_params.get_param_metadata("id", "post1").unwrap();
        assert_eq!(post1_meta.get("title").unwrap(), "Post 1");
        assert_eq!(post1_meta.get("description").unwrap(), "First blog post");

        let post2_meta = blog_params.get_param_metadata("id", "post2").unwrap();
        assert_eq!(post2_meta.get("title").unwrap(), "Post 2");

        // Verify combined metadata
        let params = HashMap::from([("id".to_string(), "post1".to_string())]);
        let combined_meta = config.get_metadata_for_parameterized_route("/blog/:id", &params);

        assert_eq!(combined_meta.get("title").unwrap(), "Post 1");
        assert_eq!(combined_meta.get("section").unwrap(), "Blog");
        assert_eq!(combined_meta.get("param_id").unwrap(), "post1");
        assert_eq!(combined_meta.get("site_name").unwrap(), "Test Site");
    }

    #[test]
    fn test_json_loader_from_str() {
        let json_str = r#"
{
  "general": {
    "output_dir": "json_dist",
    "site_name": "JSON Test Site"
  },
  "global_metadata": {
    "framework": "Yew",
    "version": "1.0.0"
  },
  "routes": [
    {
      "path": "/",
      "metadata": {
        "title": "JSON Home"
      }
    }
  ],
  "parameterized_routes": [
    {
      "pattern": "/user/:username",
      "parameters": [
        {
          "name": "username",
          "values": ["admin", "guest"]
        }
      ],
      "metadata": {
        "section": "Users"
      },
      "variants": [
        {
          "values": {
            "username": "admin"
          },
          "metadata": {
            "title": "Admin Profile",
            "access_level": "full"
          }
        },
        {
          "values": {
            "username": "guest"
          },
          "metadata": {
            "title": "Guest Profile",
            "access_level": "limited"
          }
        }
      ]
    }
  ]
}
"#;

        // Load config using the JSON loader
        let config = JsonLoader::load_from_str(json_str).unwrap();

        // Verify basic configuration
        assert_eq!(config.output_dir, PathBuf::from("json_dist"));

        // Verify global metadata
        assert_eq!(config.global_metadata.get("framework").unwrap(), "Yew");
        assert_eq!(
            config.global_metadata.get("site_name").unwrap(),
            "JSON Test Site"
        );

        // Verify routes
        let home_meta = config.route_metadata.get("/").unwrap();
        assert_eq!(home_meta.get("title").unwrap(), "JSON Home");

        // Verify parameterized routes
        let users_params = config.route_params.get("/user/:username").unwrap();

        // Check parameter values
        let username_values = users_params.param_values.get("username").unwrap();
        assert!(username_values.contains("admin"));
        assert!(username_values.contains("guest"));

        // Check parameter metadata
        let admin_meta = users_params
            .get_param_metadata("username", "admin")
            .unwrap();
        assert_eq!(admin_meta.get("title").unwrap(), "Admin Profile");
        assert_eq!(admin_meta.get("access_level").unwrap(), "full");

        // Verify combined metadata for guest
        let params = HashMap::from([("username".to_string(), "guest".to_string())]);
        let combined_meta = config.get_metadata_for_parameterized_route("/user/:username", &params);

        assert_eq!(combined_meta.get("title").unwrap(), "Guest Profile");
        assert_eq!(combined_meta.get("section").unwrap(), "Users");
        assert_eq!(combined_meta.get("access_level").unwrap(), "limited");
        assert_eq!(combined_meta.get("param_username").unwrap(), "guest");
    }

    #[test]
    fn test_load_from_file() -> Result<(), Box<dyn Error>> {
        // Create a temporary YAML file
        let yaml_content = r#"
general:
  output_dir: "file_test"
  site_name: "File Test"

routes:
  - path: "/file"
    metadata:
      title: "From File"
"#;
        let temp_file = create_temp_file(yaml_content, "yaml")?;
        let yaml_path = temp_file.path().with_extension("yaml");

        // Load with explicit loader
        let yaml_config = YamlLoader::load_from_path(&yaml_path)?;
        assert_eq!(yaml_config.output_dir, PathBuf::from("file_test"));

        let file_meta = yaml_config.route_metadata.get("/file").unwrap();
        assert_eq!(file_meta.get("title").unwrap(), "From File");

        // Load with automatic loader detection
        let auto_config = load_config(&yaml_path)?;
        assert_eq!(auto_config.output_dir, PathBuf::from("file_test"));

        // Create a temporary JSON file
        let json_content = r#"
{
  "general": {
    "output_dir": "json_file_test"
  },
  "routes": [
    {
      "path": "/json_file",
      "metadata": {
        "title": "JSON From File"
      }
    }
  ]
}
"#;
        let json_temp_file = create_temp_file(json_content, "json")?;
        let json_path = json_temp_file.path().with_extension("json");

        // Load JSON with automatic detection
        let json_config = load_config(&json_path)?;
        assert_eq!(json_config.output_dir, PathBuf::from("json_file_test"));

        let json_file_meta = json_config.route_metadata.get("/json_file").unwrap();
        assert_eq!(json_file_meta.get("title").unwrap(), "JSON From File");

        Ok(())
    }

    #[test]
    fn test_file_config_to_ssg_config() {
        // Create a FileConfig and test conversion to SsgConfig
        let file_config = SsgFileConfig {
            general: GeneralConfig {
                output_dir: PathBuf::from("conversion_test"),
                template_path: Some(PathBuf::from("template.html")),
                default_template: "<html>{{ content }}</html>".to_string(),
                site_name: "Conversion Test".to_string(),
                title_format: "{title} - {site_name}".to_string(),
                default_image: Some("/images/default.jpg".to_string()),
                twitter_handle: Some("testhandle".to_string()),
            },
            global_metadata: HashMap::from([
                ("lang".to_string(), "en".to_string()),
                ("author".to_string(), "Test Author".to_string()),
            ]),
            routes: vec![RouteConfig {
                path: "/conversion".to_string(),
                metadata: HashMap::from([("title".to_string(), "Conversion Page".to_string())]),
            }],
            parameterized_routes: vec![ParameterizedRouteConfig {
                pattern: "/item/:id".to_string(),
                parameters: vec![ParameterDefinition {
                    name: "id".to_string(),
                    values: vec!["item1".to_string(), "item2".to_string()],
                }],
                metadata: HashMap::from([("section".to_string(), "Items".to_string())]),
                variants: vec![
                    ParameterVariant {
                        values: HashMap::from([("id".to_string(), "item1".to_string())]),
                        metadata: HashMap::from([("title".to_string(), "Item 1".to_string())]),
                    },
                    ParameterVariant {
                        values: HashMap::from([("id".to_string(), "item2".to_string())]),
                        metadata: HashMap::from([("title".to_string(), "Item 2".to_string())]),
                    },
                ],
            }],
        };

        // Convert to SsgConfig
        let config = file_config.to_ssg_config();

        // Verify basic properties
        assert_eq!(config.output_dir, PathBuf::from("conversion_test"));
        assert_eq!(config.template_path, Some(PathBuf::from("template.html")));
        assert_eq!(config.default_template, "<html>{{ content }}</html>");

        // Verify global metadata
        assert_eq!(config.global_metadata.get("lang").unwrap(), "en");
        assert_eq!(config.global_metadata.get("author").unwrap(), "Test Author");
        assert_eq!(
            config.global_metadata.get("site_name").unwrap(),
            "Conversion Test"
        );
        assert_eq!(
            config.global_metadata.get("twitter_site").unwrap(),
            "@testhandle"
        );
        assert_eq!(
            config.global_metadata.get("default_image").unwrap(),
            "/images/default.jpg"
        );

        // Verify routes
        let route_meta = config.route_metadata.get("/conversion").unwrap();
        assert_eq!(route_meta.get("title").unwrap(), "Conversion Page");

        // Verify parameterized routes
        let item_params = config.route_params.get("/item/:id").unwrap();

        // Check parameter values
        let id_values = item_params.param_values.get("id").unwrap();
        assert!(id_values.contains("item1"));
        assert!(id_values.contains("item2"));

        // Check parameter metadata
        let item1_meta = item_params.get_param_metadata("id", "item1").unwrap();
        assert_eq!(item1_meta.get("title").unwrap(), "Item 1");

        // Test generating parameter combinations
        let combinations = item_params.generate_param_combinations();
        assert_eq!(combinations.len(), 2);

        // One combination should have id=item1
        assert!(combinations
            .iter()
            .any(|c| c.get("id") == Some(&"item1".to_string())));

        // One combination should have id=item2
        assert!(combinations
            .iter()
            .any(|c| c.get("id") == Some(&"item2".to_string())));
    }

    #[test]
    fn test_unsupported_extension() {
        // Try to load a file with an unsupported extension
        let result = load_config(Path::new("config.txt"));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unsupported configuration file extension: txt"));
    }

    #[test]
    fn test_invalid_yaml() {
        // Test with invalid YAML
        let invalid_yaml = "general: {"; // Invalid YAML - corrected semicolon
        let result = YamlLoader::load_from_str(invalid_yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_json() {
        // Test with invalid JSON
        let invalid_json = r#"{"general": {"output_dir": "invalid"}"#; // Missing closing brace
        let result = JsonLoader::load_from_str(invalid_json);
        assert!(result.is_err());
    }
}
