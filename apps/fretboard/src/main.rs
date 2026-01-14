use std::process::ExitCode;

mod cli;
mod demos;
mod dev;
mod hotpatch;
mod scaffold;

fn main() -> ExitCode {
    cli::main()
}
