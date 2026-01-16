use std::process::ExitCode;

mod cli;
mod demos;
mod dev;
mod diag;
mod hotpatch;
mod scaffold;

fn main() -> ExitCode {
    cli::main()
}
