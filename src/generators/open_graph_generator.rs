use crate::generator::Generator;
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct OpenGraphGenerator {
    site_name: String,
    default_image: String,
}

impl Generator for OpenGraphGenerator {
    fn name(&self) -> &'static str {
        "open_graph"
    }

    fn generate(
        &self,
        _route: &str,
        _content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        let mut tags = String::new();

        // Basic OG tags
        tags.push_str("<meta property=\"og:type\" content=\"website\">\n");

        let title = metadata.get("title").cloned().unwrap_or_default();
        tags.push_str(&format!(
            "<meta property=\"og:title\" content=\"{}\">\n",
            title
        ));

        let description = metadata.get("description").cloned().unwrap_or_default();
        tags.push_str(&format!(
            "<meta property=\"og:description\" content=\"{}\">\n",
            description
        ));

        let url = metadata.get("url").cloned().unwrap_or_default();
        tags.push_str(&format!("<meta property=\"og:url\" content=\"{}\">\n", url));

        let image = metadata
            .get("og:image")
            .cloned()
            .unwrap_or_else(|| self.default_image.clone());
        tags.push_str(&format!(
            "<meta property=\"og:image\" content=\"{}\">\n",
            image
        ));

        tags.push_str(&format!(
            "<meta property=\"og:site_name\" content=\"{}\">\n",
            self.site_name
        ));

        Ok(tags)
    }

    fn clone_box(&self) -> Box<dyn Generator> {
        Box::new(self.clone())
    }
}
