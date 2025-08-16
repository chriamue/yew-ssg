use about_page::app::App;
use about_page::route::LocalizedRoute;
use env_logger::{Builder, Env};
use log::{error, info};
use std::env;
use std::error::Error;
use yew_router::LanguageUtils;
use yew_ssg::config_loader::load_config;
use yew_ssg::StaticSiteGenerator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    Builder::from_env(Env::default().default_filter_or("info")).init();

    info!("🏗️ Loading configuration from config.yaml...");

    // Load configuration from file
    let config_path = "config.yaml";
    let config = match load_config(config_path) {
        Ok(config) => {
            info!("✅ Configuration loaded successfully");
            config
        }
        Err(e) => {
            error!("❌ Failed to load configuration: {}", e);
            if env::var("BASE_URL").is_err() {
                let base_url =
                    env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
                info!("Using base URL: {}", base_url);
                env::set_var("BASE_URL", base_url);
            }
            return Err(e);
        }
    };

    // Initialize the Generator
    info!("🚀 Initializing static site generator...");
    let generator = StaticSiteGenerator::new(config)?;

    // Generate Standard Routes
    info!("📄 Generating standard routes with localization...");
    generator.generate::<LocalizedRoute, App>().await?;

    // Generate Parameterized Routes with different language contexts
    info!("📝 Generating parameterized routes with localization...");

    // Example of using the language utilities for specific routes
    LanguageUtils::with_language("en", || {
        info!("Generating English parameterized routes...");
        // This will execute with English as the current language
    });

    LanguageUtils::with_language("de", || {
        info!("Generating German parameterized routes...");
        // This will execute with German as the current language
    });

    generator
        .generate_parameterized_routes::<LocalizedRoute, App>()
        .await?;

    info!("\n✅ Static site generation complete! Check the 'dist' directory.");

    Ok(())
}
