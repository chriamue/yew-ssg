use about_page::route::Route;
use about_page::switch_route::switch_route;
use std::fs;
use yew_ssg::StaticSiteGenerator;

const DEFAULT_TEMPLATE: &str = r#"<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="utf-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <title>About Page</title>
        <meta name="description" content="About Page application">
        <meta property="og:title" content="About Page">
        <meta property="og:description" content="About Page application">
        <meta property="og:type" content="website">
        <link rel="stylesheet" href="/styles.css">
        <script defer src="/app.js"></script>
    </head>
    <body>
        <div id="app">{{ content }}</div>
    </body>
</html>"#;

#[tokio::main]
async fn main() {
    // Try to load template from file, fall back to default
    let template = fs::read_to_string("dist/index.html").unwrap_or_else(|_| {
        println!("ℹ️ No template.html found, using default template");
        DEFAULT_TEMPLATE.to_string()
    });

    // Initialize the static site generator with template
    let generator = match StaticSiteGenerator::with_template("dist", &template) {
        Ok(generator) => generator,
        Err(e) => {
            eprintln!("❌ Failed to initialize generator: {}", e);
            std::process::exit(1);
        }
    };

    println!("🏗️ Generating static site...");

    // Generate static files for all routes
    match generator.generate::<Route, _>(switch_route).await {
        Ok(_) => println!("✅ Static site generation complete! Check the 'dist' directory."),
        Err(e) => {
            eprintln!("❌ Error generating static site: {}", e);
            std::process::exit(1);
        }
    }
}
