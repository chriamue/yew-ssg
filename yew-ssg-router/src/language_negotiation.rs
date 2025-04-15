/// Utility to determine the best language based on user preferences
#[derive(Clone, Debug)]
pub struct LanguageNegotiator {
    supported_languages: Vec<String>,
    default_language: String,
}

impl LanguageNegotiator {
    /// Create a new language negotiator
    pub fn new(supported_languages: &[String], default_language: &str) -> Self {
        Self {
            supported_languages: supported_languages.to_vec(),
            default_language: default_language.to_string(),
        }
    }

    /// Create a new language negotiator from string slices
    pub fn from_static(supported_languages: &[&str], default_language: &str) -> Self {
        Self {
            supported_languages: supported_languages.iter().map(|s| s.to_string()).collect(),
            default_language: default_language.to_string(),
        }
    }

    /// Determine the best language based on Accept-Language header
    /// Returns the best matching language code
    pub fn negotiate_from_header(&self, accept_language: Option<&str>) -> String {
        if let Some(header) = accept_language {
            // Parse the Accept-Language header
            let mut lang_prefs = self.parse_accept_language(header);

            // Sort by quality factor (highest first)
            lang_prefs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            // Try to find a match
            for (lang, _) in lang_prefs {
                let base_lang = lang.split('-').next().unwrap_or(&lang);

                // Try exact match first
                if self.supported_languages.contains(&lang) {
                    return lang;
                }

                // Then try base language match (e.g., "en-US" -> "en")
                if self.supported_languages.contains(&base_lang.to_string()) {
                    return base_lang.to_string();
                }
            }
        }

        // Return default if no match found
        self.default_language.clone()
    }

    /// Parse the Accept-Language header into (language, quality) pairs
    fn parse_accept_language(&self, header: &str) -> Vec<(String, f32)> {
        let mut result = Vec::new();

        for part in header.split(',') {
            let part = part.trim();

            // Parse language and quality factor
            if let Some((lang, q_part)) = part.split_once(';') {
                if let Some(q_str) = q_part.trim().strip_prefix("q=") {
                    if let Ok(quality) = q_str.parse::<f32>() {
                        result.push((lang.trim().to_lowercase(), quality));
                        continue;
                    }
                }
            }

            // Default quality is 1.0 if not specified
            result.push((part.to_lowercase(), 1.0));
        }

        result
    }

    /// Get the supported languages
    pub fn supported_languages(&self) -> &[String] {
        &self.supported_languages
    }

    /// Get the default language
    pub fn default_language(&self) -> &str {
        &self.default_language
    }
}
