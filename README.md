# yew-ssg 🚀

A static site generator for Yew applications that helps you pre-render your Yew apps for better SEO and load times.

> ⚠️ **PERSONAL PROJECT DISCLAIMER**: This is my personal project focused on fulfilling my specific needs. It is currently in alpha stage, not actively maintained, and **should not be used in production**. Feel free to fork, provide feedback, or use for experimental purposes. Parts of the documentation and code were assisted by AI.

## Features

- 🚀 Pre-renders Yew applications to static HTML (SSR → static)
- 🔄 Works with (and ships an SSG‑optimized fork of) yew-router
- 📝 Customizable HTML templates (MiniJinja + attribute directives)
- 🎯 Advanced attribute-based templating (`data-ssg`, `data-ssg-*`, `data-ssg-placeholder`)
- 🧩 Extensible generator plugin system (meta, OG, Twitter, JSON-LD, etc.)
- 🔍 Built-in SEO generators (meta tags, Open Graph, Twitter Cards, canonical / hreflang, robots)
- 🌐 Internationalization and localization with localized routes + language negotiation
- 🧠 Thread‑local + environment based language context for multi-lingual generation
- 🤖 Robots meta tag support
- 🔀 Flexible, pluggable processing pipeline
- 🧪 Parameterized routes (e.g. `/crate/:id`) with metadata variants
- 🧱 JSON / YAML configuration loader

## Installation

Add `yew-ssg` to your `Cargo.toml` (current workspace version is `0.2.1` – adjust as needed):

```toml
[dependencies]
yew = { version = "0.21", features = ["csr"] }
yew-ssg = "0.2.2"
```

If you only generate static pages in a separate binary, you can make it feature-gated:

```toml
[dependencies]
yew = { version = "0.21", features = ["csr"] }
yew-ssg = { version = "0.2.1", optional = true }

[features]
ssg = ["yew/ssr", "yew-ssg"]
```

## Router Integration

For static generation you can use the provided `yew-ssg-router` (a compatibility shim around `yew-router` that swaps components under the `ssg` feature):

```toml
[dependencies]
# Use the SSG-aware router (workspace path or git)
yew_router = { git = "https://github.com/chriamue/yew-ssg", package = "yew-ssg-router" }

[features]
ssg = ["yew/ssr", "yew-ssg", "yew_router/ssg"]
```

When `feature = "ssg"`:
- `BrowserRouter` → `StaticRouter`
- `Link` → `StaticLink`
- `Switch` → `StaticSwitch`
- Localized versions adapt automatically.

Usage stays the same:

```rust
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq, Debug)]
enum Route {
    #[at("/")] Home,
    #[at("/about")] About,
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html!("Home"),
        Route::About => html!("About"),
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <nav>
                <Link<Route> to={Route::Home}>{"Home"}</Link<Route>>
                <Link<Route> to={Route::About}>{"About"}</Link<Route>>
            </nav>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}
```

## Quick Start

1. Add `yew-ssg` and optional `yew-ssg-router`
2. Create an SSG binary (e.g. `src/bin/ssg.rs`)
3. Run it (e.g. `cargo run --bin ssg --features ssg`)
4. Deploy generated `dist/` directory to static hosting

Example SSG binary:

```rust
use my_app::App;
use my_app::Route;
use yew_ssg::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Build config
    let config = SsgConfigBuilder::new()
        .output_dir("dist")
        .add_generator(MetaTagGenerator {
            default_description: "My site description".into(),
            default_keywords: vec!["yew".into(), "rust".into()],
        })
        .add_generator(OpenGraphGenerator {
            site_name: "My Site".into(),
            default_image: "/images/default.jpg".into(),
        })
        .build();

    // Initialize
    let generator = StaticSiteGenerator::new(config)?;

    // Generate pages for all Route variants
    generator.generate::<Route, App>().await?;

    println!("✅ Static site generated in dist/");
    Ok(())
}
```

## Localization Support

You can produce localized variants of each route (e.g. `/`, `/de/`, `/fr/about`, …). Two approaches:

### 1. Macro `impl_localized_route!`

```rust
use yew_router::prelude::*;
use strum_macros::EnumIter;

#[derive(Clone, Routable, PartialEq, Debug, EnumIter, Default)]
pub enum Route {
    #[at("/")]
    #[default]
    Home,
    #[at("/about")]
    About,
}

pub const SUPPORTED_LANGUAGES: &[&str] = &["en", "de", "fr"];
pub const DEFAULT_LANGUAGE: &str = "en";

impl_localized_route!(Route, LocalizedRoute, SUPPORTED_LANGUAGES, DEFAULT_LANGUAGE);
```

### 2. Derive Macro `LocalizedRoutable`

```rust
use yew_router::prelude::*;
use strum_macros::EnumIter;
use yew_ssg_router_macros::LocalizedRoutable;

#[derive(Clone, Routable, PartialEq, Debug, EnumIter, Default, LocalizedRoutable)]
#[localized(
    languages = ["en", "de", "fr"],
    default = "en",
    wrapper = "LocalizedRoute"
)]
pub enum Route {
    #[at("/")]
    #[default]
    Home,
    #[at("/about")]
    About,
}
```

### Localized App Integration

```rust
#[function_component(App)]
pub fn app() -> Html {
    html! {
        <LocalizedApp<LocalizedRoute>>
            <nav>
                <Link<LocalizedRoute> to={LocalizedRoute::from_route(Route::Home, Some("en"))}>
                    {"English Home"}
                </Link<LocalizedRoute>>
            </nav>
            <LocalizedSwitch<LocalizedRoute> render={switch_route} />
        </LocalizedApp<LocalizedRoute>>
    }
}
```

### Language Negotiation

Use `LanguageNegotiator` to map `Accept-Language` to a supported code:

```rust
let negotiator = LanguageNegotiator::from_static(&["en","de","fr"], "en");
let preferred = negotiator.negotiate_from_header(
    Some("de-CH,de;q=0.8,en-US;q=0.6,en;q=0.5")
);
```

## Language Context & Thread‑Local Support

The router / localization layer maintains language in two coordinated places:

Priority order when reading current language:
1. Thread-local (set by `LanguageProvider`)
2. `YEW_SSG_CURRENT_LANG` environment variable
3. Default fallback (usually `"en"`)

Utilities:

```rust
use yew_ssg_router::LanguageUtils;

// Scoped execution
LanguageUtils::with_language("de", || {
    // Any SSR rendering here sees language = "de"
});

// Manual set/clear
LanguageUtils::set_generation_language("fr");
let current = LanguageUtils::detect_language();
LanguageUtils::clear_generation_language();
```

You rarely need to call these manually if you use `<LocalizedApp<...>>` / `<LanguageProvider>`.

## Parameterized Routes

Define patterns like `/crate/:id` with a list of allowed values & variant metadata in YAML/JSON config. The generator will produce one page per combination.

Example YAML snippet:

```yaml
parameterized_routes:
  - pattern: "/crate/:id"
    parameters:
      - name: "id"
        values: ["yew-ssg", "yew-ssg-router"]
    variants:
      - values: { id: "yew-ssg" }
        metadata:
          title: "yew-ssg | Static Site Generator"
      - values: { id: "yew-ssg-router" }
        metadata:
          title: "yew-ssg-router | Router Integration"
```

Then call:

```rust
generator.generate_parameterized_routes::<LocalizedRoute, App>().await?;
```

## Configuration (YAML / JSON)

Load external config:

```rust
use yew_ssg::config_loader::load_config;

let cfg = load_config("config.yaml")?;
let generator = StaticSiteGenerator::new(cfg)?;
generator.generate::<Route, App>().await?;
```

Supports:
- `general.output_dir`
- `general.template_path` / inline template
- `global_metadata`
- `routes[]`
- `parameterized_routes[]`
- Asset & JSON-LD base directories
- Canonical / alternate language behavior

## Template System

### Variable Substitution

MiniJinja variables inside your HTML template:

```html
<title>{{ title }}</title>
{{ meta_tags | safe }}
```

### Attribute / Element Directives

Directive forms:
1. `data-ssg="key"` – replace element content
2. `data-ssg-ATTR="key"` – replace specific attribute value
3. `data-ssg-placeholder="key"` – replace entire element with generated HTML block

Priority resolution:
1. Generator-produced output
2. Metadata
3. Preserve original content (unless special-case like `content`)

### Example

```html
<title data-ssg="title">Fallback Title</title>
<meta name="description" data-ssg-content="description_content" content="Default desc">
<div data-ssg-placeholder="open_graph"></div>
```

## Built-in Generators

| Generator | Purpose |
|-----------|---------|
| `TitleGenerator` | `<title>` tag + plain text variant |
| `MetaTagGenerator` | description / keywords / canonical |
| `CanonicalLinkGenerator` | canonical + hreflang alternates |
| `OpenGraphGenerator` | OG meta tags |
| `TwitterCardGenerator` | Twitter card tags |
| `RobotsMetaGenerator` | robots meta tag |
| `JsonLdGenerator` | JSON-LD (inline or file-based) |

Add programmatically via `SsgConfigBuilder::add_generator(...)` or rely on defaults.

## Custom Generator Example

```rust
#[derive(Debug, Clone)]
struct CustomGenerator;

impl Generator for CustomGenerator {
    fn name(&self) -> &'static str { "custom_block" }

    fn generate(
        &self,
        key: &str,
        _route: &str,
        _content: &str,
        metadata: &HashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if key == "custom_block" {
            Ok(format!("<section>{}</section>",
                metadata.get("custom").unwrap_or(&"".into())))
        } else {
            Err(format!("Unsupported key {key}").into())
        }
    }

    fn clone_box(&self) -> Box<dyn Generator> { Box::new(self.clone()) }
    fn as_any(&self) -> &dyn std::any::Any { self }
}
```

Use in template: `{{ custom_block | safe }}` or `<div data-ssg-placeholder="custom_block"></div>`.

## Processing Pipeline

1. Render Yew component (SSR → HTML fragment)
2. Fill template (MiniJinja)
3. Variable replacement processor (`{{ var }}`)
4. Attribute / placeholder processor (`data-ssg-*`)
5. Write output to `dist/<route>/index.html`

Both processors can be replaced or supplemented with custom implementations.

## Environment Variables (Build-Time)

| Variable | Purpose |
|----------|---------|
| `YEW_SSG_CURRENT_PATH` | Current route path during generation (internal) |
| `YEW_SSG_CURRENT_PATH_PREFIX` | Optional path prefix for output (e.g. GitHub Pages subdir) |
| `YEW_SSG_CURRENT_LANG` | Current language fallback (if thread-local not set) |
| `YEW_SSG_PARAM_*` | Parameter values during parameterized route generation |
| `BASE_URL` | Used by router utilities to build absolute links |
| `YEW_SSG_PARAM_<name>` | Dynamic route parameter injection |

## Testing

Some tests (not exhaustive):
- Generators (individual outputs & priority)
- Attribute processor scenarios
- Canonical + alternates & translation behavior
- JSON/YAML config loader
- Thread-local + env language context (new tests)
- Localized route macro + iterator

To run:

```bash
cargo test
```

If you add the optional SSR integration test for language context, ensure:
```toml
[dev-dependencies]
tokio = { version = "1", features = ["rt","macros"] }
```

## Example Project

See `examples/about-page` for:
- Localization
- Parameterized routes
- JSON-LD file injection
- Attribute-based SEO tag insertion

## Roadmap / Ideas

- Asset hashing + manifest injection
- Incremental / selective regeneration
- Partial hydration helpers
- More robust error reporting & tracing
- Parallel route rendering

## Project Status 🚧

- ⚠️ **Alpha Stage** – Breaking changes likely
- 🧪 **Experimental** – Built for personal exploration
- 📉 **Limited maintenance** – PRs welcome but not guaranteed
- 🔍 **Partial test coverage** – Use with caution

## Contributing 🤝

Welcome (with the above caveats):
- File issues / feature discussions
- Small focused PRs
- Documentation improvements
- Example expansions

## License 📄

MIT – see [LICENSE](LICENSE).
