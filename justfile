# List available commands
default:
    @just --list

# Build the core library
build:
    cargo build

# Run tests for the core library
test:
    cargo test

# Run tests with all features
test-all:
    cargo test --all-features

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Format all code
fmt:
    cargo fmt --all

# Run clippy lints
clippy:
    cargo clippy -- -D warnings

# Build example
build-example:
    cd examples/about-page && trunk build

# Run SSG for the example
build-example-ssg: build-example
    cd examples/about-page && cargo run --bin ssg --features ssg

# Serve the generated example site using simple HTTP server (requires Python)
serve-example:
    cd examples/about-page/dist && python3 -m http.server 8080

# Clean all build artifacts
clean:
    cargo clean
    cd examples/about-page && cargo clean

# Full build and test workflow
check-all: fmt clippy test-all build-example

# Build example site and serve it
example: build-example-ssg serve-example
