use about_page::app::App;
use about_page::route::Route;
use env_logger::{Builder, Env};
use log::{error, info};
use std::env;
use std::error::Error;
use yew_ssg::config_loader::load_config;
use yew_ssg::StaticSiteGenerator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger with custom format
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use std::io::Write;
            let level_style = buf.default_level_style(record.level());
            writeln!(
                buf,
                "{} {}",
                level_style.value(record.level()),
                record.args()
            )
        })
        .init();

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
            // Set BASE_URL environment variable for proper URL generation
            if env::var("BASE_URL").is_err() {
                let base_url =
                    env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string());
                info!("Using base URL: {}", base_url);
                env::set_var("BASE_URL", base_url);
            }
            return Err(e);
        }
    };

    // --- Initialize the Generator ---
    info!("🚀 Initializing static site generator...");
    let generator = StaticSiteGenerator::new(config)?;

    // --- Generate Standard Routes ---
    info!("📄 Generating standard routes...");
    generator.generate::<Route, App>().await?;

    // --- Generate Parameterized Routes ---
    info!("📝 Generating parameterized routes...");

    // Generate parameterized routes automatically - no lambda function needed!
    generator
        .generate_parameterized_routes::<Route, App>()
        .await?;

    info!("\n✅ Static site generation complete! Check the 'dist' directory.");

    Ok(())
}
