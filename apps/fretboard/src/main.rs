#![recursion_limit = "512"]

#[cfg(target_arch = "wasm32")]
fn main() {
    // This binary is not supported on wasm targets.
}

#[cfg(not(target_arch = "wasm32"))]
use std::process::ExitCode;

#[cfg(not(target_arch = "wasm32"))]
mod cli;
#[cfg(not(target_arch = "wasm32"))]
mod config;
#[cfg(not(target_arch = "wasm32"))]
mod demos;
#[cfg(not(target_arch = "wasm32"))]
mod dev;
#[cfg(not(target_arch = "wasm32"))]
mod diag;
#[cfg(not(target_arch = "wasm32"))]
mod hotpatch;
#[cfg(not(target_arch = "wasm32"))]
mod scaffold;
#[cfg(not(target_arch = "wasm32"))]
mod theme;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> ExitCode {
    cli::main()
}
