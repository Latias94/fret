use clap::{CommandFactory, Parser, Subcommand};

use crate::assets::contracts::AssetsCommandArgs;
use crate::config::contracts::ConfigCommandArgs;

#[derive(Debug, Parser)]
#[command(
    name = "fretboard",
    about = "CLI tooling for Fret apps and assets.",
    disable_help_subcommand = true,
    subcommand_required = true
)]
pub struct FretboardCliContract {
    #[command(subcommand)]
    pub command: FretboardCommandContract,
}

#[derive(Debug, Subcommand)]
pub enum FretboardCommandContract {
    /// Manage generated asset manifests and Rust glue.
    Assets(AssetsCommandArgs),
    /// Configure project-local settings and generated config files.
    Config(ConfigCommandArgs),
}

pub fn try_parse_contract<I, T>(args: I) -> Result<FretboardCliContract, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    FretboardCliContract::try_parse_from(args)
}

pub fn render_command_help_path(path: &[&str]) -> Result<String, String> {
    let mut cmd = FretboardCliContract::command();
    let mut current = &mut cmd;
    for segment in path {
        current = current
            .find_subcommand_mut(segment)
            .ok_or_else(|| format!("missing clap help for {}", full_bin_name(path)))?;
    }

    let mut renderable = current.clone().bin_name(full_bin_name(path));
    let mut out = Vec::new();
    renderable
        .write_long_help(&mut out)
        .map_err(|err| err.to_string())?;
    String::from_utf8(out).map_err(|err| err.to_string())
}

fn full_bin_name(path: &[&str]) -> String {
    if path.is_empty() {
        "fretboard".to_string()
    } else {
        format!("fretboard {}", path.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::{FretboardCommandContract, render_command_help_path, try_parse_contract};

    #[test]
    fn assets_help_lists_manifest_and_rust_targets() {
        let help = render_command_help_path(&["assets"]).expect("assets help should render");
        assert!(help.contains("manifest"));
        assert!(help.contains("rust"));
    }

    #[test]
    fn config_help_lists_menubar_target() {
        let help = render_command_help_path(&["config"]).expect("config help should render");
        assert!(help.contains("menubar"));
    }

    #[test]
    fn root_contract_parses_assets_subcommand() {
        let cli = try_parse_contract([
            "fretboard",
            "assets",
            "manifest",
            "write",
            "--dir",
            "assets",
            "--out",
            "assets.manifest.json",
            "--app-bundle",
            "demo",
        ])
        .expect("assets command should parse");
        matches!(cli.command, FretboardCommandContract::Assets(_));
    }
}
