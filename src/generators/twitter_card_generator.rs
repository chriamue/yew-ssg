use crate::generator::Generator;
use crate::processors::GeneratorOutputSupport;
use std::any::Any;
use std::collections::HashMap;
use std::error::Error;

/// Generator for Twitter Card meta tags
///
/// Twitter Cards allow you to attach rich photos, videos and media to tweets
/// that drive traffic to your website.
///
/// See: https://developer.twitter.com/en/docs/twitter-for-websites/cards/overview/abouts-cards
#[derive(Debug, Clone)]
pub struct TwitterCardGenerator {
    /// Twitter handle of the site (e.g., "@yoursitename")
    pub twitter_site: Option<String>,

    /// Default card type if not specified in metadata
    ///
    /// Options include:
    /// - "summary" (default) - Title, description, and thumbnail
    /// - "summary_large_image" - Similar to summary but with a larger image
    /// - "app" - For mobile app downloads
    /// - "player" - For video content
    pub default_card_type: String,
}

impl Default for TwitterCardGenerator {
    fn default() -> Self {
        Self {
            twitter_site: None,
            default_card_type: "summary".to_string(),
        }
    }
}

impl Generator for TwitterCardGenerator {
    fn name(&self) -> &'static str {
        "twitter_card"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn generate(
        &self,
        key: &str,
        _route: &str,
        _content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        match key {
            // Main output: full Twitter Card tags
            "twitter_card" => {
                let mut tags = String::new();

                // Card type
                let card_type = metadata
                    .get("twitter:card")
                    .cloned()
                    .unwrap_or_else(|| self.default_card_type.clone());
                tags.push_str(&format!(
                    "<meta name=\"twitter:card\" content=\"{}\">\n",
                    card_type
                ));

                // Site account
                if let Some(site) = &self.twitter_site {
                    tags.push_str(&format!(
                        "<meta name=\"twitter:site\" content=\"{}\">\n",
                        site
                    ));
                }

                // Creator account (if different from site)
                if let Some(creator) = metadata.get("twitter:creator") {
                    tags.push_str(&format!(
                        "<meta name=\"twitter:creator\" content=\"{}\">\n",
                        creator
                    ));
                }

                // Title (required)
                let title = metadata
                    .get("twitter:title")
                    .or_else(|| metadata.get("title"))
                    .cloned()
                    .unwrap_or_else(|| "".to_string());
                tags.push_str(&format!(
                    "<meta name=\"twitter:title\" content=\"{}\">\n",
                    title
                ));

                // Description
                if let Some(description) = metadata
                    .get("twitter:description")
                    .or_else(|| metadata.get("description"))
                {
                    tags.push_str(&format!(
                        "<meta name=\"twitter:description\" content=\"{}\">\n",
                        description
                    ));
                }

                // Image
                if let Some(image) = metadata.get("twitter:image") {
                    tags.push_str(&format!(
                        "<meta name=\"twitter:image\" content=\"{}\">\n",
                        image
                    ));

                    // Image alt text (accessibility)
                    if let Some(alt) = metadata.get("twitter:image:alt") {
                        tags.push_str(&format!(
                            "<meta name=\"twitter:image:alt\" content=\"{}\">\n",
                            alt
                        ));
                    }
                }

                // For player cards
                if card_type == "player" {
                    if let Some(player) = metadata.get("twitter:player") {
                        tags.push_str(&format!(
                            "<meta name=\"twitter:player\" content=\"{}\">\n",
                            player
                        ));
                    }

                    if let Some(width) = metadata.get("twitter:player:width") {
                        tags.push_str(&format!(
                            "<meta name=\"twitter:player:width\" content=\"{}\">\n",
                            width
                        ));
                    }

                    if let Some(height) = metadata.get("twitter:player:height") {
                        tags.push_str(&format!(
                            "<meta name=\"twitter:player:height\" content=\"{}\">\n",
                            height
                        ));
                    }
                }

                // For app cards
                if card_type == "app" {
                    // iOS app details
                    if let Some(id) = metadata.get("twitter:app:id:iphone") {
                        tags.push_str(&format!(
                            "<meta name=\"twitter:app:id:iphone\" content=\"{}\">\n",
                            id
                        ));
                    }

                    if let Some(name) = metadata.get("twitter:app:name:iphone") {
                        tags.push_str(&format!(
                            "<meta name=\"twitter:app:name:iphone\" content=\"{}\">\n",
                            name
                        ));
                    }

                    // Android app details
                    if let Some(id) = metadata.get("twitter:app:id:googleplay") {
                        tags.push_str(&format!(
                            "<meta name=\"twitter:app:id:googleplay\" content=\"{}\">\n",
                            id
                        ));
                    }

                    if let Some(name) = metadata.get("twitter:app:name:googleplay") {
                        tags.push_str(&format!(
                            "<meta name=\"twitter:app:name:googleplay\" content=\"{}\">\n",
                            name
                        ));
                    }
                }

                Ok(tags)
            }

            // Individual Twitter Card properties
            "twitter:card" => {
                let card_type = metadata
                    .get("twitter:card")
                    .cloned()
                    .unwrap_or_else(|| self.default_card_type.clone());

                Ok(format!(
                    "<meta name=\"twitter:card\" content=\"{}\">\n",
                    card_type
                ))
            }

            "twitter:site" => {
                if let Some(site) = &self.twitter_site {
                    Ok(format!(
                        "<meta name=\"twitter:site\" content=\"{}\">\n",
                        site
                    ))
                } else {
                    Ok("".to_string())
                }
            }

            "twitter:creator" => {
                if let Some(creator) = metadata.get("twitter:creator") {
                    Ok(format!(
                        "<meta name=\"twitter:creator\" content=\"{}\">\n",
                        creator
                    ))
                } else {
                    Ok("".to_string())
                }
            }

            "twitter:title" => {
                let title = metadata
                    .get("twitter:title")
                    .or_else(|| metadata.get("title"))
                    .cloned()
                    .unwrap_or_else(|| "".to_string());

                Ok(format!(
                    "<meta name=\"twitter:title\" content=\"{}\">\n",
                    title
                ))
            }

            "twitter:description" => {
                if let Some(description) = metadata
                    .get("twitter:description")
                    .or_else(|| metadata.get("description"))
                {
                    Ok(format!(
                        "<meta name=\"twitter:description\" content=\"{}\">\n",
                        description
                    ))
                } else {
                    Ok("".to_string())
                }
            }

            "twitter:image" => {
                if let Some(image) = metadata.get("twitter:image") {
                    Ok(format!(
                        "<meta name=\"twitter:image\" content=\"{}\">\n",
                        image
                    ))
                } else {
                    Ok("".to_string())
                }
            }

            "twitter:image:alt" => {
                if let Some(alt) = metadata.get("twitter:image:alt") {
                    Ok(format!(
                        "<meta name=\"twitter:image:alt\" content=\"{}\">\n",
                        alt
                    ))
                } else {
                    Ok("".to_string())
                }
            }

            // Player-specific properties
            "twitter:player" => {
                if let Some(player) = metadata.get("twitter:player") {
                    Ok(format!(
                        "<meta name=\"twitter:player\" content=\"{}\">\n",
                        player
                    ))
                } else {
                    Ok("".to_string())
                }
            }

            "twitter:player:width" => {
                if let Some(width) = metadata.get("twitter:player:width") {
                    Ok(format!(
                        "<meta name=\"twitter:player:width\" content=\"{}\">\n",
                        width
                    ))
                } else {
                    Ok("".to_string())
                }
            }

            "twitter:player:height" => {
                if let Some(height) = metadata.get("twitter:player:height") {
                    Ok(format!(
                        "<meta name=\"twitter:player:height\" content=\"{}\">\n",
                        height
                    ))
                } else {
                    Ok("".to_string())
                }
            }

            // App-specific properties
            "twitter:app:id:iphone" => {
                if let Some(id) = metadata.get("twitter:app:id:iphone") {
                    Ok(format!(
                        "<meta name=\"twitter:app:id:iphone\" content=\"{}\">\n",
                        id
                    ))
                } else {
                    Ok("".to_string())
                }
            }

            "twitter:app:name:iphone" => {
                if let Some(name) = metadata.get("twitter:app:name:iphone") {
                    Ok(format!(
                        "<meta name=\"twitter:app:name:iphone\" content=\"{}\">\n",
                        name
                    ))
                } else {
                    Ok("".to_string())
                }
            }

            "twitter:app:id:googleplay" => {
                if let Some(id) = metadata.get("twitter:app:id:googleplay") {
                    Ok(format!(
                        "<meta name=\"twitter:app:id:googleplay\" content=\"{}\">\n",
                        id
                    ))
                } else {
                    Ok("".to_string())
                }
            }

            "twitter:app:name:googleplay" => {
                if let Some(name) = metadata.get("twitter:app:name:googleplay") {
                    Ok(format!(
                        "<meta name=\"twitter:app:name:googleplay\" content=\"{}\">\n",
                        name
                    ))
                } else {
                    Ok("".to_string())
                }
            }

            // Return just the value without the meta tag for each property
            key if key.starts_with("twitter_value:") => {
                let prop_name = key.strip_prefix("twitter_value:").unwrap_or("");
                let twitter_key = format!("twitter:{}", prop_name);

                if prop_name == "site" {
                    if let Some(site) = &self.twitter_site {
                        return Ok(site.clone());
                    }
                } else if let Some(value) = metadata.get(&twitter_key) {
                    return Ok(value.clone());
                } else if prop_name == "card" {
                    return Ok(self.default_card_type.clone());
                } else if prop_name == "title" && metadata.get("title").is_some() {
                    return Ok(metadata.get("title").unwrap().clone());
                } else if prop_name == "description" && metadata.get("description").is_some() {
                    return Ok(metadata.get("description").unwrap().clone());
                }

                Ok("".to_string())
            }

            // Unsupported key
            _ => Err(format!("TwitterCardGenerator does not support key: {}", key).into()),
        }
    }

    fn clone_box(&self) -> Box<dyn Generator> {
        Box::new(self.clone())
    }
}

impl GeneratorOutputSupport for TwitterCardGenerator {
    fn supported_outputs(&self) -> Vec<&'static str> {
        vec![
            "twitter_card",
            "twitter:card",
            "twitter:site",
            "twitter:creator",
            "twitter:title",
            "twitter:description",
            "twitter:image",
            "twitter:image:alt",
            "twitter:player",
            "twitter:player:width",
            "twitter:player:height",
            "twitter:app:id:iphone",
            "twitter:app:name:iphone",
            "twitter:app:id:googleplay",
            "twitter:app:name:googleplay",
            "twitter_value:card",
            "twitter_value:site",
            "twitter_value:creator",
            "twitter_value:title",
            "twitter_value:description",
            "twitter_value:image",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_twitter_card() {
        let generator = TwitterCardGenerator::default();

        // Test with empty metadata
        let result = generator
            .generate(
                "twitter_card",
                "/test-route",
                "<div>Test content</div>",
                &HashMap::new(),
            )
            .unwrap();

        assert!(result.contains("<meta name=\"twitter:card\" content=\"summary\">"));
        assert!(result.contains("<meta name=\"twitter:title\" content=\"\">"));
        assert!(!result.contains("twitter:site")); // Should not be present if not set
    }

    #[test]
    fn test_twitter_card_with_site() {
        let generator = TwitterCardGenerator {
            twitter_site: Some("@rustyew".to_string()),
            default_card_type: "summary".to_string(),
        };

        // Test with site metadata
        let result = generator
            .generate(
                "twitter_card",
                "/test-route",
                "<div>Test content</div>",
                &HashMap::new(),
            )
            .unwrap();

        assert!(result.contains("<meta name=\"twitter:site\" content=\"@rustyew\">"));
    }

    #[test]
    fn test_twitter_card_with_custom_metadata() {
        let generator = TwitterCardGenerator {
            twitter_site: Some("@rustyew".to_string()),
            default_card_type: "summary".to_string(),
        };

        // Set up custom metadata
        let mut metadata = HashMap::new();
        metadata.insert(
            "twitter:card".to_string(),
            "summary_large_image".to_string(),
        );
        metadata.insert("title".to_string(), "My Test Page".to_string());
        metadata.insert(
            "description".to_string(),
            "A page about testing".to_string(),
        );
        metadata.insert(
            "twitter:image".to_string(),
            "https://example.com/image.jpg".to_string(),
        );
        metadata.insert(
            "twitter:image:alt".to_string(),
            "Alt text for image".to_string(),
        );

        // Test with custom metadata
        let result = generator
            .generate(
                "twitter_card",
                "/test-route",
                "<div>Test content</div>",
                &metadata,
            )
            .unwrap();

        // Check that all the expected tags are present
        assert!(result.contains("<meta name=\"twitter:card\" content=\"summary_large_image\">"));
        assert!(result.contains("<meta name=\"twitter:site\" content=\"@rustyew\">"));
        assert!(result.contains("<meta name=\"twitter:title\" content=\"My Test Page\">"));
        assert!(
            result.contains("<meta name=\"twitter:description\" content=\"A page about testing\">")
        );
        assert!(result
            .contains("<meta name=\"twitter:image\" content=\"https://example.com/image.jpg\">"));
        assert!(result.contains("<meta name=\"twitter:image:alt\" content=\"Alt text for image\">"));
    }

    #[test]
    fn test_twitter_player_card() {
        let generator = TwitterCardGenerator {
            twitter_site: Some("@rustyew".to_string()),
            default_card_type: "player".to_string(),
        };

        // Set up player card metadata
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "My Video".to_string());
        metadata.insert(
            "twitter:player".to_string(),
            "https://example.com/player.html".to_string(),
        );
        metadata.insert("twitter:player:width".to_string(), "480".to_string());
        metadata.insert("twitter:player:height".to_string(), "320".to_string());

        // Test with player card metadata
        let result = generator
            .generate(
                "twitter_card",
                "/test-route",
                "<div>Test content</div>",
                &metadata,
            )
            .unwrap();

        // Check for player-specific tags
        assert!(result.contains("<meta name=\"twitter:card\" content=\"player\">"));
        assert!(result.contains(
            "<meta name=\"twitter:player\" content=\"https://example.com/player.html\">"
        ));
        assert!(result.contains("<meta name=\"twitter:player:width\" content=\"480\">"));
        assert!(result.contains("<meta name=\"twitter:player:height\" content=\"320\">"));
    }

    #[test]
    fn test_individual_twitter_properties() {
        let generator = TwitterCardGenerator {
            twitter_site: Some("@rustyew".to_string()),
            default_card_type: "summary".to_string(),
        };

        // Set up metadata
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "My Page Title".to_string());
        metadata.insert(
            "twitter:image".to_string(),
            "https://example.com/image.jpg".to_string(),
        );

        // Test individual card property
        let card_result = generator
            .generate(
                "twitter:card",
                "/test-route",
                "<div>Test content</div>",
                &metadata,
            )
            .unwrap();
        assert_eq!(
            card_result,
            "<meta name=\"twitter:card\" content=\"summary\">\n"
        );

        // Test site property
        let site_result = generator
            .generate(
                "twitter:site",
                "/test-route",
                "<div>Test content</div>",
                &metadata,
            )
            .unwrap();
        assert_eq!(
            site_result,
            "<meta name=\"twitter:site\" content=\"@rustyew\">\n"
        );

        // Test title property (falls back to regular title)
        let title_result = generator
            .generate(
                "twitter:title",
                "/test-route",
                "<div>Test content</div>",
                &metadata,
            )
            .unwrap();
        assert_eq!(
            title_result,
            "<meta name=\"twitter:title\" content=\"My Page Title\">\n"
        );

        // Test image property
        let image_result = generator
            .generate(
                "twitter:image",
                "/test-route",
                "<div>Test content</div>",
                &metadata,
            )
            .unwrap();
        assert_eq!(
            image_result,
            "<meta name=\"twitter:image\" content=\"https://example.com/image.jpg\">\n"
        );
    }

    #[test]
    fn test_twitter_value_properties() {
        let generator = TwitterCardGenerator {
            twitter_site: Some("@rustyew".to_string()),
            default_card_type: "summary".to_string(),
        };

        // Set up metadata
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), "My Page Title".to_string());
        metadata.insert(
            "twitter:image".to_string(),
            "https://example.com/image.jpg".to_string(),
        );
        metadata.insert("twitter:creator".to_string(), "@author".to_string());

        // Test plain values without HTML tags
        let card_value = generator
            .generate(
                "twitter_value:card",
                "/test-route",
                "<div>Test content</div>",
                &metadata,
            )
            .unwrap();
        assert_eq!(card_value, "summary");

        let site_value = generator
            .generate(
                "twitter_value:site",
                "/test-route",
                "<div>Test content</div>",
                &metadata,
            )
            .unwrap();
        assert_eq!(site_value, "@rustyew");

        let creator_value = generator
            .generate(
                "twitter_value:creator",
                "/test-route",
                "<div>Test content</div>",
                &metadata,
            )
            .unwrap();
        assert_eq!(creator_value, "@author");

        let image_value = generator
            .generate(
                "twitter_value:image",
                "/test-route",
                "<div>Test content</div>",
                &metadata,
            )
            .unwrap();
        assert_eq!(image_value, "https://example.com/image.jpg");
    }

    #[test]
    fn test_unsupported_twitter_key() {
        let generator = TwitterCardGenerator::default();

        // Test with an unsupported key
        let result = generator.generate(
            "unsupported_key",
            "/test-route",
            "<div>Test content</div>",
            &HashMap::new(),
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("does not support key"));
    }
}
