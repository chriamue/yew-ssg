use about_page::app::App;
use about_page::route::Route;
use env_logger::{Builder, Env};
use log::{error, info};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use strum::IntoEnumIterator;
use yew_router::Routable;
use yew_ssg::generators::{MetaTagGenerator, OpenGraphGenerator};
use yew_ssg::{SsgConfigBuilder, StaticSiteGenerator};

// Environment variable names
const ENV_BASE_URL: &str = "BASE_URL";
const ENV_SITE_NAME: &str = "SITE_NAME";
const ENV_OG_IMAGE: &str = "DEFAULT_OG_IMAGE";
const ENV_TWITTER: &str = "TWITTER_HANDLE";

// Default values
const DEFAULT_SITE_NAME: &str = "Yew SSG Example";
const DEFAULT_KEYWORDS: &str = "yew, rust, ssg, webdev, spa, seo";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger with custom format
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use chrono::Local;
            use std::io::Write;

            let level_style = buf.default_level_style(record.level());
            writeln!(
                buf,
                "{} {} {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                level_style.value(record.level()),
                record.args()
            )
        })
        .init();

    info!("🏗️ Configuring static site generator...");

    // --- Load Environment Configuration ---
    let config = load_config()?;

    // --- Configure the SSG Builder ---
    let mut builder = SsgConfigBuilder::new()
        .output_dir("dist")
        .template("dist/index.html");

    // 1. Add global metadata
    builder = builder.global_metadata(create_global_metadata(&config));

    // 2. Add route-specific metadata
    builder = add_route_metadata(builder, &config.base_url);

    // 3. Add SEO generators
    builder = add_generators(builder);

    // 4. Add processors
    builder = add_processors(builder);

    // --- Build and Run the Generator ---
    let ssg_config = builder.build();
    let generator = StaticSiteGenerator::new(ssg_config.clone())?;

    info!("🚀 Starting static site generation...");

    // 5. Generate the static site
    generator.generate::<Route, App>().await?;

    // Output success information
    print_success_info(&generator);

    Ok(())
}

#[derive(Debug)]
struct SiteConfig {
    base_url: String,
    site_name: String,
    default_og_image: String,
    twitter_handle: String,
}

/// Load configuration from environment variables
fn load_config() -> Result<SiteConfig, Box<dyn Error>> {
    // Get required BASE_URL
    let base_url = env::var(ENV_BASE_URL).map_err(|_| {
        error!(
            "Environment variable {} must be set (e.g., https://example.com)",
            ENV_BASE_URL
        );
        format!(
            "Environment variable {} must be set (e.g., https://example.com)",
            ENV_BASE_URL
        )
    })?;
    let base_url = base_url.trim_end_matches('/').to_string();

    // Get optional values with defaults
    let site_name = env::var(ENV_SITE_NAME).unwrap_or_else(|_| DEFAULT_SITE_NAME.to_string());
    let default_og_image = env::var(ENV_OG_IMAGE)
        .unwrap_or_else(|_| format!("{}/images/default-social-cover.jpg", base_url));
    let twitter_handle = env::var(ENV_TWITTER).unwrap_or_default();

    // Log configuration
    info!("Configuration:");
    info!("  Base URL: {}", base_url);
    info!("  Site Name: {}", site_name);
    info!("  Default OG Image: {}", default_og_image);
    if !twitter_handle.is_empty() {
        info!("  Twitter Handle: {}", twitter_handle);
    }

    Ok(SiteConfig {
        base_url,
        site_name,
        default_og_image,
        twitter_handle,
    })
}

/// Create global metadata from configuration
fn create_global_metadata(config: &SiteConfig) -> HashMap<String, String> {
    let mut metadata = HashMap::new();
    metadata.insert("site_name".to_string(), config.site_name.clone());
    metadata.insert("base_url".to_string(), config.base_url.clone());
    metadata.insert(
        "default_og_image".to_string(),
        config.default_og_image.clone(),
    );
    metadata.insert("default_keywords".to_string(), DEFAULT_KEYWORDS.to_string());

    // Add shared meta values
    metadata.insert("charset".to_string(), "utf-8".to_string());
    metadata.insert(
        "viewport".to_string(),
        "width=device-width, initial-scale=1.0".to_string(),
    );
    metadata.insert("og:type".to_string(), "website".to_string());

    if !config.twitter_handle.is_empty() {
        metadata.insert("twitter_site".to_string(), config.twitter_handle.clone());
    }

    metadata
}

/// Get title and description for a specific route
fn get_route_meta_content(route: &Route) -> (&'static str, &'static str) {
    match route {
        Route::Home => (
            "Home | Yew SSG Example",
            "Welcome to the Yew Static Site Generator example homepage. See how Yew SPAs can be SEO-friendly.",
        ),
        Route::About => (
            "About Us | Yew SSG Example",
            "Learn about the yew-ssg project and how it helps pre-render Yew applications.",
        ),
        Route::Readme => (
            "README | Yew SSG Documentation",
            "Explore the documentation and usage examples for yew-ssg static site generation.",
        ),
        Route::Crate { .. } => (
            "Crate | Yew SSG Crates",
            "Yew SSG Crates"
        ),
        Route::NotFound => (
            "Page Not Found (404) | Yew SSG Example",
            "Oops! The page you are looking for doesn't exist. Please check the URL.",
        ),
    }
}

/// Add SEO generators to the configuration
fn add_generators(builder: SsgConfigBuilder) -> SsgConfigBuilder {
    info!("Using default generators with customization...");

    // Default generators will be added, we just need to add our custom ones or override defaults
    builder
        // Override the meta tag generator with our specific defaults
        .add_generator(MetaTagGenerator {
            default_description: "A statically generated Yew application.".to_string(),
            default_keywords: vec!["yew".to_string(), "rust".to_string()],
        })
        // Override the Open Graph generator with site name
        .add_generator(OpenGraphGenerator {
            site_name: DEFAULT_SITE_NAME.to_string(),
            default_image: "".to_string(),
        })
}

fn add_route_metadata(builder: SsgConfigBuilder, base_url: &str) -> SsgConfigBuilder {
    let mut builder = builder;

    info!("Configuring route metadata...");

    for route in Route::iter() {
        let path = route.to_path();
        let mut route_meta = HashMap::new();

        // Set title and description based on route
        let (title, description) = get_route_meta_content(&route);
        route_meta.insert("title".to_string(), title.to_string());
        route_meta.insert("description".to_string(), description.to_string());
        route_meta.insert("keywords".to_string(), DEFAULT_KEYWORDS.to_string());

        // Set canonical URL and OG URL
        let absolute_url = format!("{}{}", base_url, path);
        route_meta.insert("canonical".to_string(), absolute_url.clone());
        route_meta.insert("url".to_string(), absolute_url);

        // Add og:type for each route
        route_meta.insert("og:type".to_string(), "website".to_string());

        // Handle robots directive
        if route == Route::NotFound {
            route_meta.insert("robots".to_string(), "noindex, nofollow".to_string());
        } else {
            route_meta.insert("robots".to_string(), "index, follow".to_string());
        }

        builder = builder.route_metadata(&path, route_meta);
    }

    builder
}

fn add_processors(builder: SsgConfigBuilder) -> SsgConfigBuilder {
    info!("Using default processors...");
    builder
}

fn print_success_info(generator: &StaticSiteGenerator) {
    info!("\n✅ Static site generation complete! Check the 'dist' directory.");

    // Show page summary
    info!("\n📊 Generated Pages:");
    for route in Route::iter() {
        let path = route.to_path();
        let meta = generator.config.get_metadata_for_route(&path);
        let title = meta
            .get("title")
            .cloned()
            .unwrap_or_else(|| "N/A".to_string());
        info!("  📄 {} - '{}'", path, title);
    }

    // Show enabled generators
    info!("\n✨ SEO Generators Enabled:");
    for generator in &generator.config.generators {
        info!("  ✓ {}", generator.name());
    }

    // Show enabled processors
    info!("\n🔧 Content Processors Enabled:");
    for processor in &generator.config.processors {
        info!("  ✓ {}", processor.name());
    }
}
