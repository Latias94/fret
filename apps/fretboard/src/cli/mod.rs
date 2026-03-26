use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::error::ErrorKind;

mod contracts;
mod cutover;
mod help;

#[derive(Debug)]
enum CliError {
    Message(String),
    Clap(clap::Error),
}

pub(crate) fn main() -> ExitCode {
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
    let args: Vec<OsString> = std::env::args_os().collect();
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

    cutover::dispatch(contract).map_err(CliError::Message)
}

fn should_render_root_help(args: &[OsString]) -> bool {
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

pub(crate) fn help() -> Result<(), String> {
    help::print_root_help();
    Ok(())
}

pub(crate) fn workspace_root() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    for dir in cwd.ancestors() {
        if dir.join("Cargo.toml").is_file() {
            return Ok(dir.to_path_buf());
        }
    }
    Err("failed to locate workspace root (Cargo.toml not found in ancestors)".to_string())
}
