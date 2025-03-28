# yew-ssg

A static site generator for Yew applications that helps you pre-render your Yew apps for better SEO and load times.

## Features

- 🚀 Pre-renders Yew applications to static HTML
- 🔄 Works with yew-router for multi-page applications
- 📝 Customizable HTML templates
- 🧩 Plugin system for extensibility
- 🔍 SEO-friendly output with metadata support

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
use std::fs;
use yew_ssg::StaticSiteGenerator;

#[tokio::main]
async fn main() {
    // Load template or use default
    let template = fs::read_to_string("dist/index.html")
        .unwrap_or_else(|_| {
            println!("Using default template");
            r#"<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8">
        <title>{{ title }}</title>
        <link rel="stylesheet" href="/styles.css">
        <script defer src="/app.js"></script>
    </head>
    <body>
        <div id="app">{{ content }}</div>
    </body>
</html>"#.to_string()
        });

    // Initialize the generator
    let generator = StaticSiteGenerator::with_template("dist", &template)
        .expect("Failed to create generator");

    // Generate static files
    generator.generate::<Route, _>(switch_route).await
        .expect("Failed to generate static site");

    println!("✅ Static site generated successfully!");
}
```

## Advanced Usage

### Custom Templates

You can provide a custom HTML template with placeholders:

- `{{ content }}` - The rendered Yew content
- `{{ title }}` - Page title (from route)
- `{{ description }}` - Page description
- `{{ path }}` - Route path

### Metadata Support

Using the `SsgConfigBuilder`, you can add global and per-route metadata:

```rust
let config = SsgConfigBuilder::new()
    .output_dir("dist")
    .global_metadata(HashMap::from([
        ("site_name".to_string(), "My Awesome Site".to_string()),
    ]))
    .route_metadata("/about", HashMap::from([
        ("title".to_string(), "About Us".to_string()),
        ("description".to_string(), "Learn about our company".to_string()),
    ]))
    .build();
```

### Generator Plugins

Implement custom generators to extend functionality:

```rust
#[derive(Debug, Clone)]
pub struct MyCustomGenerator;

#[async_trait]
impl Generator for MyCustomGenerator {
    fn name(&self) -> &'static str {
        "custom"
    }

    async fn generate(
        &self,
        route: &str,
        content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error>> {
        // Custom generation logic
        Ok(format!("<custom>{}</custom>", metadata.get("custom").unwrap_or(&String::new())))
    }

    fn box_clone(&self) -> Box<dyn Generator + Send + Sync> {
        Box::new(self.clone())
    }
}

// Add to config
let config = SsgConfigBuilder::new()
    .add_generator(MyCustomGenerator)
    .build();
```

## Example Projects

Check out the examples directory for complete project examples:

- `examples/about-page`: Simple multi-page site with home and about pages
- `examples/blog`: Blog with markdown content
- `examples/portfolio`: Portfolio site with dynamic routing

## Contributing

Contributions welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
