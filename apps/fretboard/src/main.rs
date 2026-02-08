#![recursion_limit = "512"]

use std::process::ExitCode;

mod cli;
mod config;
mod demos;
mod dev;
mod diag;
mod hotpatch;
mod scaffold;

fn main() -> ExitCode {
    cli::main()
}
