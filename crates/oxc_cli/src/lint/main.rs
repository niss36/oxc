#![cfg(not(miri))] // Miri does not support custom allocators

#[cfg(not(debug_assertions))]
#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[cfg(not(debug_assertions))]
#[cfg(target_os = "windows")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use oxc_cli::{CliRunResult, LintRunner, Runner};

fn main() -> CliRunResult {
    init_tracing();
    init_miette();

    let command = oxc_cli::lint_command().run();
    command.handle_threads();
    LintRunner::new(command.lint_options).run()
}

// Initialize the data which relies on `is_atty` system calls so they don't block subsequent threads.
fn init_miette() {
    miette::set_hook(Box::new(|_| Box::new(miette::MietteHandlerOpts::new().build()))).unwrap();
}

/// To debug `oxc_resolver`:
/// `OXC_LOG=oxc_resolver oxlint --import-plugin`
fn init_tracing() {
    use tracing_subscriber::{filter::Targets, prelude::*};

    // Usage without the `regex` feature.
    // <https://github.com/tokio-rs/tracing/issues/1436#issuecomment-918528013>
    tracing_subscriber::registry()
        .with(std::env::var("OXC_LOG").map_or_else(
            |_| Targets::new(),
            |env_var| {
                use std::str::FromStr;
                Targets::from_str(&env_var).unwrap()
            },
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
