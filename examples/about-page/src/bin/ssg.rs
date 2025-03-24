use about_page::route::Route;
use about_page::switch_route::switch_route;
use yew_ssg::StaticSiteGenerator;

#[tokio::main]
async fn main() {
    // Initialize the static site generator with output directory
    let generator = StaticSiteGenerator::new("dist");

    println!("🏗️ Generating static site...");

    // Generate static files for all routes
    match generator.generate::<Route, _>(switch_route).await {
        Ok(_) => println!("✅ Static site generation complete! Check the 'dist' directory."),
        Err(e) => eprintln!("❌ Error generating static site: {}", e),
    }
}
