use clap::{CommandFactory, Parser, Subcommand};

use crate::assets::contracts::AssetsCommandArgs;
use crate::config::contracts::ConfigCommandArgs;
use crate::dev::contracts::DevCommandArgs;
use crate::icons::contracts::IconsCommandArgs;
use crate::scaffold::contracts::NewCommandArgs;

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
    /// Capture and inspect project diagnostics bundles.
    Diag(ForwardedSubcommandArgs),
    /// Run project-native and web app targets.
    Dev(DevCommandArgs),
    /// Generate icon-pack crates from local icon sources.
    Icons(IconsCommandArgs),
    /// Create a new app from a starter template.
    New(NewCommandArgs),
}

#[derive(Debug, Clone, clap::Args, PartialEq, Eq)]
#[command(disable_help_flag = true, disable_help_subcommand = true)]
pub struct ForwardedSubcommandArgs {
    /// Remaining arguments forwarded to the diagnostics handler.
    #[arg(
        value_name = "ARG",
        num_args = 0..,
        allow_hyphen_values = true,
        trailing_var_arg = true
    )]
    pub args: Vec<String>,
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
    use crate::dev::contracts::DevTargetContract;

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
    fn diag_contract_forwards_help_flags_to_fret_diag() {
        let cli = try_parse_contract(["fretboard", "diag", "--help"])
            .expect("diag --help should forward to fret-diag");

        let FretboardCommandContract::Diag(args) = cli.command else {
            panic!("expected diag command");
        };

        assert_eq!(args.args, vec!["--help"]);
    }

    #[test]
    fn dev_help_lists_native_and_web_targets() {
        let help = render_command_help_path(&["dev"]).expect("dev help should render");
        assert!(help.contains("native"));
        assert!(help.contains("web"));
    }

    #[test]
    fn icons_help_lists_import_sources() {
        let help = render_command_help_path(&["icons"]).expect("icons help should render");
        assert!(help.contains("import"));

        let import_help = render_command_help_path(&["icons", "import"])
            .expect("icons import help should render");
        assert!(import_help.contains("svg-dir"));
        assert!(import_help.contains("iconify-collection"));

        let svg_help = render_command_help_path(&["icons", "import", "svg-dir"])
            .expect("icons import svg-dir help should render");
        assert!(svg_help.contains("semantic-aliases"));
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
        assert!(matches!(cli.command, FretboardCommandContract::Assets(_)));
    }

    #[test]
    fn root_contract_parses_diag_forwarded_args() {
        let cli = try_parse_contract(["fretboard", "diag", "latest", "--dir", "target/fret-diag"])
            .expect("diag command should parse");

        let FretboardCommandContract::Diag(args) = cli.command else {
            panic!("expected diag command");
        };

        assert_eq!(args.args, vec!["latest", "--dir", "target/fret-diag"]);
    }

    #[test]
    fn root_contract_parses_dev_native_subcommand() {
        let cli = try_parse_contract([
            "fretboard",
            "dev",
            "native",
            "--manifest-path",
            "./Cargo.toml",
            "--bin",
            "todo_demo",
            "--watch",
            "--",
            "--help",
        ])
        .expect("dev native should parse");

        let FretboardCommandContract::Dev(dev) = cli.command else {
            panic!("expected dev command");
        };

        let DevTargetContract::Native(args) = dev.target else {
            panic!("expected native dev target");
        };

        assert_eq!(args.bin.as_deref(), Some("todo_demo"));
        assert!(args.watch);
        assert_eq!(args.passthrough, vec!["--help"]);
    }

    #[test]
    fn root_contract_parses_icons_svg_dir_subcommand() {
        let cli = try_parse_contract([
            "fretboard",
            "icons",
            "import",
            "svg-dir",
            "--source",
            "./icons",
            "--crate-name",
            "demo-icons",
            "--vendor-namespace",
            "demo",
        ])
        .expect("icons svg-dir command should parse");

        assert!(matches!(cli.command, FretboardCommandContract::Icons(_)));
    }

    #[test]
    fn root_contract_parses_icons_iconify_collection_subcommand() {
        let cli = try_parse_contract([
            "fretboard",
            "icons",
            "import",
            "iconify-collection",
            "--source",
            "./lucide.json",
            "--crate-name",
            "lucide-icons",
            "--vendor-namespace",
            "lucide",
        ])
        .expect("icons iconify-collection command should parse");

        assert!(matches!(cli.command, FretboardCommandContract::Icons(_)));
    }

    #[test]
    fn new_help_lists_scaffold_templates() {
        let help = render_command_help_path(&["new"]).expect("new help should render");
        assert!(help.contains("hello"));
        assert!(help.contains("simple-todo"));
        assert!(help.contains("todo"));
    }

    #[test]
    fn root_contract_parses_new_subcommand() {
        let cli = try_parse_contract(["fretboard", "new", "hello", "--name", "hello-world"])
            .expect("new command should parse");
        assert!(matches!(cli.command, FretboardCommandContract::New(_)));
    }
}
