use crate::generator::Generator;
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

    fn generate(
        &self,
        _route: &str,
        _content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
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

    fn clone_box(&self) -> Box<dyn Generator> {
        Box::new(self.clone())
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
            .generate("/test-route", "<div>Test content</div>", &HashMap::new())
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
            .generate("/test-route", "<div>Test content</div>", &HashMap::new())
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
            .generate("/test-route", "<div>Test content</div>", &metadata)
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
            .generate("/test-route", "<div>Test content</div>", &metadata)
            .unwrap();

        // Check for player-specific tags
        assert!(result.contains("<meta name=\"twitter:card\" content=\"player\">"));
        assert!(result.contains(
            "<meta name=\"twitter:player\" content=\"https://example.com/player.html\">"
        ));
        assert!(result.contains("<meta name=\"twitter:player:width\" content=\"480\">"));
        assert!(result.contains("<meta name=\"twitter:player:height\" content=\"320\">"));
    }
}
