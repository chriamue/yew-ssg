use super::SsgAttribute;
use lol_html::html_content::{ContentType, Element};
use std::collections::HashMap;

/// Process an element with the given attribute and content
pub fn process_element(
    attribute: SsgAttribute,
    element: &mut Element,
    generated_content: &str,
    _metadata: &HashMap<String, String>,
) {
    // Remove SSG attributes
    element.remove_attribute("data-ssg");
    element.remove_attribute("data-ssg-placeholder");

    // Collect data-ssg-* attributes first
    let attrs_to_remove: Vec<String> = element
        .attributes()
        .into_iter()
        .filter(|attr| attr.name().starts_with("data-ssg-"))
        .map(|attr| attr.name().to_string())
        .collect();

    // Now remove them
    for attr_name in attrs_to_remove {
        element.remove_attribute(&attr_name);
    }

    // Process based on attribute type
    match attribute {
        SsgAttribute::Content => {
            // Special flag to preserve original content
            if generated_content == "{{__PRESERVE_ORIGINAL__}}" {
                // Do nothing, keeping original content
            } else if !generated_content.is_empty() {
                // For meta tags with content attribute, update the content attribute instead
                if element.tag_name() == "meta" && element.has_attribute("content") {
                    let _ = element.set_attribute("content", generated_content);
                } else {
                    // Replace inner content with generated content for other elements
                    element.set_inner_content(generated_content, ContentType::Html);
                }
            }
            // If empty, we still want to replace (for cases like data-ssg="content")
            else if element.get_attribute("data-ssg") == Some("content".to_string()) {
                element.set_inner_content(generated_content, ContentType::Html);
            }
        }
        SsgAttribute::Attribute(attr_name) => {
            // Set the target attribute value
            let _ = element.set_attribute(&attr_name, generated_content);
        }
        SsgAttribute::Placeholder => {
            // For placeholders, completely replace the element with the generated content
            element.replace(generated_content, ContentType::Html);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lol_html::{element, HtmlRewriter, Settings};
    use std::collections::HashMap;

    fn test_rewriting(
        input: &str,
        selector: &str,
        attribute: SsgAttribute,
        content: &str,
    ) -> String {
        let metadata = HashMap::new();
        let mut output = Vec::new();

        let mut rewriter = HtmlRewriter::new(
            Settings {
                element_content_handlers: vec![element!(selector, move |el| {
                    process_element(attribute.clone(), el, content, &metadata);
                    Ok(())
                })],
                ..Settings::default()
            },
            |c: &[u8]| output.extend_from_slice(c),
        );

        rewriter.write(input.as_bytes()).unwrap();
        rewriter.end().unwrap();

        String::from_utf8(output).unwrap()
    }

    #[test]
    fn test_process_element_content() {
        let html = r#"<div data-ssg="content">Old content</div>"#;
        let result = test_rewriting(
            html,
            "div[data-ssg=content]",
            SsgAttribute::Content,
            "<p>New content</p>",
        );

        println!("Result: {}", result);

        // Verify the result doesn't contain data-ssg attribute
        assert!(!result.contains("data-ssg="));

        // Check for the div with new content
        assert!(result.contains("<div><p>New content</p></div>"));
    }

    #[test]
    fn test_process_element_attribute() {
        let html = r#"<div data-ssg-title="title" title="Old title">Content</div>"#;
        let result = test_rewriting(
            html,
            "div[data-ssg-title=title]",
            SsgAttribute::Attribute("title".to_string()),
            "New title",
        );

        println!("Result: {}", result);

        // Verify the result contains the new title
        assert!(result.contains(r#"title="New title""#));

        // Verify the result doesn't contain data-ssg-title attribute
        assert!(!result.contains("data-ssg-title"));
    }

    #[test]
    fn test_process_element_title() {
        let html = r#"<title data-ssg="title">Default Title</title>"#;
        let result = test_rewriting(
            html,
            "title[data-ssg=title]",
            SsgAttribute::Content,
            "Generated Title",
        );

        println!("Title Result: {}", result);

        // Verify the result doesn't contain data-ssg attribute
        assert!(!result.contains("data-ssg="));

        // Check for the title with new content
        assert!(result.contains("<title>Generated Title</title>"));
    }

    #[test]
    fn test_process_element_placeholder() {
        let html = r#"<div data-ssg-placeholder="meta">Original placeholder content</div>"#;
        let result = test_rewriting(
            html,
            "div[data-ssg-placeholder=meta]",
            SsgAttribute::Placeholder,
            "<meta name=\"description\" content=\"Generated description\">",
        );

        println!("Placeholder Result: {}", result);

        // Verify the result doesn't contain data-ssg-placeholder attribute
        assert!(!result.contains("data-ssg-placeholder"));

        // Check that the original element was completely replaced
        assert!(!result.contains("<div"));
        assert!(result.contains("<meta name=\"description\" content=\"Generated description\">"));
    }
}
