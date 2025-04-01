#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SsgAttribute {
    /// Represents an attribute that replaces the value of an existing attribute.
    Attribute(String),
    /// Represents an attribute that replaces the content of an element.
    Content,
}

impl SsgAttribute {
    /// Parses a string into an `SsgAttribute` enum.
    ///
    /// # Arguments
    ///
    /// * `attr` - The attribute string to parse.
    ///
    /// # Returns
    ///
    /// An `SsgAttribute` enum variant.
    pub fn from_str(attr: &str) -> Option<Self> {
        if attr == "data-ssg" {
            Some(SsgAttribute::Content)
        } else if attr.starts_with("data-ssg-") {
            let key = attr.strip_prefix("data-ssg-").unwrap().to_string();
            Some(SsgAttribute::Attribute(key))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        assert_eq!(
            SsgAttribute::from_str("data-ssg"),
            Some(SsgAttribute::Content)
        );
        assert_eq!(
            SsgAttribute::from_str("data-ssg-title"),
            Some(SsgAttribute::Attribute("title".to_string()))
        );
        assert_eq!(SsgAttribute::from_str("invalid"), None);
    }
}
