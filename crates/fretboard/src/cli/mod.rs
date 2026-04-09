use std::process::ExitCode;

use clap::error::ErrorKind;

mod contracts;
mod help;

#[derive(Debug)]
enum CliError {
    Message(String),
    Clap(clap::Error),
}

pub fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(CliError::Message(err)) => {
            eprintln!("error: {err}");
            ExitCode::from(2)
        }
        Err(CliError::Clap(err)) => {
            eprint!("{err}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), CliError> {
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    if should_render_root_help(&args) {
        return help().map_err(CliError::Message);
    }

    let contract = match contracts::try_parse_contract(args) {
        Ok(contract) => contract,
        Err(err) if is_display_only_error(err.kind()) => {
            print!("{err}");
            return Ok(());
        }
        Err(err) => return Err(CliError::Clap(err)),
    };

    match contract.command {
        contracts::FretboardCommandContract::Assets(args) => {
            crate::assets::run_assets_contract(args).map_err(CliError::Message)
        }
        contracts::FretboardCommandContract::Config(args) => {
            crate::config::run_config_contract(args).map_err(CliError::Message)
        }
    }
}

fn should_render_root_help(args: &[std::ffi::OsString]) -> bool {
    if args.len() <= 1 {
        return true;
    }

    matches!(
        args.get(1).and_then(|arg| arg.to_str()),
        Some("help" | "-h" | "--help")
    )
}

fn is_display_only_error(kind: ErrorKind) -> bool {
    matches!(kind, ErrorKind::DisplayHelp | ErrorKind::DisplayVersion)
}

pub fn help() -> Result<(), String> {
    help::print_root_help()
}
