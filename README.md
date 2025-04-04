# yew-ssg 🚀

A static site generator for Yew applications that helps you pre-render your Yew apps for better SEO and load times.

> ⚠️ **PERSONAL PROJECT DISCLAIMER**: This is my personal project focused on fulfilling my specific needs. It is currently in alpha stage, not actively maintained, and **should not be used in production**. Feel free to fork, provide feedback, or use for experimental purposes. Parts of the documentation and code were assisted by AI.

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

### Router Integration

yew-ssg includes its own router implementation (`yew-ssg-router`) that's optimized for static site generation. To use it:

```toml
[dependencies]
# Use yew-ssg-router from the repository
yew_router = { git = "https://github.com/chriamue/yew-ssg", package = "yew-ssg-router" }

# Enable SSG features in your project
[features]
ssg = ["yew/ssr", "yew-ssg", "yew_router/ssg"]
```

The router provides SSG-compatible versions of the standard router components:
- `BrowserRouter` - For client-side routing
- `Link` - For navigation links
- `Switch` - For route matching and rendering

During development, these components work like normal client-side routing components. During static site generation, they automatically switch to their static counterparts when the `ssg` feature is enabled.

Your application code remains the same - just use the router components as you normally would:

```rust
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Link<Route> to={Route::Home}>{"Home"}</Link<Route>>
            <Switch<Route> render={switch_route} />
        </BrowserRouter>
    }
}
```

The `yew-ssg-router` package handles all the complexity of static site generation behind the scenes, ensuring your routes work both during development and in the generated static output.

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

### Basic Template Variables ✨

Use double curly braces for variable substitution:

```html
<title>{{ title }}</title>
<meta name="description" content="{{ description }}">
```

### Attribute-Based Templating 🔧

The attribute processor provides three powerful ways to template your HTML:

1. `data-ssg`: Content replacement
2. `data-ssg-*`: Attribute value replacement
3. `data-ssg-placeholder`: Complete element replacement

#### Content Replacement with data-ssg 📄

Replace element content with generated or metadata values:

```html
<!-- Replaces content with generated title or metadata title -->
<title data-ssg="title">Default Title</title>

<!-- Special case for main content -->
<div data-ssg="content">Loading...</div>

<!-- Uses metadata value with fallback to default -->
<h1 data-ssg="page_heading">Default Heading</h1>
```

The processor looks for content in this order:
1. Generated content from generators
2. Metadata values
3. If no replacement is found, the original content is preserved

#### Attribute Replacement with data-ssg-* 🏷️

Update specific attributes with generated or metadata values:

```html
<!-- Update the content attribute -->
<meta name="description"
      data-ssg-content="description"
      content="Default description">

<!-- Update the href attribute -->
<link rel="canonical"
      data-ssg-href="canonical_url"
      href="https://example.com">

<!-- Multiple attributes on one element -->
<meta data-ssg-content="meta-content"
      data-ssg-name="meta-name"
      content="default-content"
      name="default-name">
```

#### Complete Element Replacement with data-ssg-placeholder 🔄

Replace the entire element with generated content:

```html
<!-- Meta tags will be inserted here, replacing the entire div -->
<div data-ssg-placeholder="meta_tags">Loading meta tags...</div>

<!-- Open Graph tags will be inserted here -->
<div data-ssg-placeholder="open_graph">Loading Open Graph tags...</div>

<!-- Twitter Card tags will be inserted here -->
<div data-ssg-placeholder="twitter_card">Loading Twitter Card tags...</div>
```

Placeholders can insert any HTML, including multiple elements, which makes them perfect for injecting blocks of meta tags or other complex structures.

### Metadata Configuration 📋

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

### Built-in Generators 🛠️

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

### Generator Output Priority ⚡

When multiple sources can provide content for a template variable or attribute:

1. Generator outputs have highest priority
2. Generator-computed values come next
3. Metadata values are used as fallback
4. Original content is preserved if no replacement is found

This allows flexibility in combining different content sources while maintaining a clear precedence order.

### Custom Generators 🧰

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
        key: &str,
        route: &str,
        content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        Ok(format!("<custom-element>{}</custom-element>",
                  metadata.get("custom").unwrap_or(&String::new())))
    }

    fn supports_output(&self, key: &str) -> bool {
        key == "custom_element"
    }

    fn clone_box(&self) -> Box<dyn Generator> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
```

## Processing Pipeline 🔄

The yew-ssg processing pipeline consists of:

1. **Content generation** 📝: Pre-renders Yew components to HTML
2. **Template processing** 🧩: Applies HTML templates with variables
3. **Attribute processing** 🔍: Processes special data-ssg attributes
4. **Output generation** 📤: Writes files to the output directory

Each stage can be customized or extended as needed.

## Documentation 📚

For more detailed architecture documentation, check out:
- [Architecture Documentation (arc42)](https://blog.chriamue.de/yew-ssg/docs/)

## Example Projects 🔍

Check out the examples directory for complete project examples:

- `examples/about-page`: Basic example

## Project Status 🚧

- ⚠️ **Alpha Stage**: Expect breaking changes and incomplete features
- 🔬 **Experimental**: Created primarily for personal projects
- 📈 **Development**: Sporadic updates based on personal needs
- 🧪 **Testing**: Limited test coverage, use at your own risk

## Contributing 🤝

While this is primarily a personal project, feedback and contributions are welcome! Feel free to:
- Open issues for bugs or feature suggestions
- Submit pull requests for improvements
- Fork the project for your own needs

## License 📄

This project is licensed under the MIT License - see the LICENSE file for details.
