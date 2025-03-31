# yew-ssg

A static site generator for Yew applications that helps you pre-render your Yew apps for better SEO and load times.

## Features

- 🚀 Pre-renders Yew applications to static HTML
- 🔄 Works with yew-router for multi-page applications
- 📝 Customizable HTML templates with variable substitution
- 🎯 Advanced attribute-based templating system
- 🧩 Extensible generator plugin system
- 🔍 Built-in SEO generators (meta tags, Open Graph, Twitter Cards)
- 🤖 Robots meta tag support
- 🔀 Flexible processing pipeline

## Installation

Add `yew-ssg` to your `Cargo.toml`:

```toml
[dependencies]
yew-ssg = "0.1.0"
```

For SSG functionality, enable the necessary features:

```toml
[dependencies]
yew = { version = "0.21", features = ["csr"] }
yew-ssg = { optional = true }

[features]
ssg = ["yew/ssr", "yew-ssg"]
```

## Quick Start

1. Create a standard Yew application with routing
2. Add an SSG binary to your project
3. Run the SSG binary to generate static HTML

Example SSG binary:

```rust
use my_app::route::Route;
use my_app::switch_route::switch_route;
use yew_ssg::prelude::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    // Configure the SSG
    let config = SsgConfigBuilder::new()
        .output_dir("dist")
        // Add built-in generators
        .add_generator(MetaTagGenerator {
            default_description: "My site description".to_string(),
            default_keywords: vec!["yew".to_string(), "rust".to_string()],
        })
        .add_generator(OpenGraphGenerator {
            site_name: "My Site".to_string(),
            default_image: "/images/default.jpg".to_string(),
        })
        .build();

    // Initialize the generator
    let generator = StaticSiteGenerator::new(config)
        .expect("Failed to create generator");

    // Generate static files
    generator.generate::<Route, _>(switch_route).await
        .expect("Failed to generate static site");

    println!("✅ Static site generated successfully!");
}
```

## Template System

### Basic Template Variables

Use double curly braces for variable substitution:

```html
<title>{{ title }}</title>
<meta name="description" content="{{ description }}">
```

### Attribute-Based Templating

The attribute processor provides three powerful ways to template your HTML:

1. `data-ssg`: Direct content replacement
2. `data-ssg-a`: Attribute value replacement
3. `data-ssg-placeholder`: Generator output placement

#### Content Replacement with data-ssg

Replace element content with metadata values:

```html
<title data-ssg="title">Default Title</title>
<h1 data-ssg="page_heading">Default Heading</h1>
```

#### Attribute Replacement with data-ssg-a

Update specific attributes with metadata values:

```html
<meta name="description"
      data-ssg="description"
      data-ssg-a="content"
      content="Default description">

<link rel="canonical"
      data-ssg="canonical_url"
      data-ssg-a="href"
      href="https://example.com">
```

#### Generator Output Placement with data-ssg-placeholder

Place generator outputs in specific locations:

```html
<!-- Meta tags will be inserted here -->
<meta data-ssg-placeholder="meta_tags" content="">

<!-- Open Graph tags will be inserted here -->
<meta data-ssg-placeholder="open_graph" content="">

<!-- Twitter Card tags will be inserted here -->
<meta data-ssg-placeholder="twitter_card" content="">
```

### Metadata Configuration

Add global and route-specific metadata:

```rust
let config = SsgConfigBuilder::new()
    .output_dir("dist")
    .global_metadata(HashMap::from([
        ("site_name".to_string(), "My Awesome Site".to_string()),
        ("author".to_string(), "Jane Doe".to_string()),
    ]))
    .route_metadata("/about", HashMap::from([
        ("title".to_string(), "About Us".to_string()),
        ("description".to_string(), "Learn about our company".to_string()),
        ("canonical_url".to_string(), "https://example.com/about".to_string()),
    ]))
    .build();
```

### Built-in Generators

yew-ssg includes several built-in generators:

- `MetaTagGenerator`: Basic meta tags
- `OpenGraphGenerator`: Open Graph protocol tags
- `TwitterCardGenerator`: Twitter Card meta tags
- `RobotsMetaGenerator`: Robots meta tag
- `TitleGenerator`: HTML title tag

Example configuration:

```rust
let config = SsgConfigBuilder::new()
    .add_generator(MetaTagGenerator {
        default_description: "Site description".to_string(),
        default_keywords: vec!["rust".to_string(), "yew".to_string()],
    })
    .add_generator(OpenGraphGenerator {
        site_name: "My Site".to_string(),
        default_image: "/images/og-default.jpg".to_string(),
    })
    .add_generator(TwitterCardGenerator {
        twitter_site: Some("@mysite".to_string()),
        default_card_type: "summary_large_image".to_string(),
    })
    .build();
```

### Custom Generators

Implement your own generators:

```rust
#[derive(Debug, Clone)]
pub struct CustomGenerator;

impl Generator for CustomGenerator {
    fn name(&self) -> &'static str {
        "custom_generator"
    }

    fn generate(
        &self,
        route: &str,
        content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        Ok(format!("<custom-element>{}</custom-element>",
                  metadata.get("custom").unwrap_or(&String::new())))
    }

    fn clone_box(&self) -> Box<dyn Generator> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
```

## Example Projects

Check out the examples directory for complete project examples:

- `examples/about-page`: Basic example

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
