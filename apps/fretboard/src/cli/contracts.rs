use clap::{Args, CommandFactory, Parser, Subcommand};

use crate::assets::contracts::AssetsCommandArgs;
use crate::config::contracts::ConfigCommandArgs;
use crate::dev::contracts::{DevNativeCommandArgs, DevWebCommandArgs};
use crate::hotpatch::contracts::HotpatchCommandArgs;
use crate::scaffold::contracts::NewCommandArgs;
use crate::theme::contracts::ThemeCommandArgs;

#[derive(Debug, Parser)]
#[command(
    name = "fretboard-dev",
    about = "Dev tooling for the Fret workspace.",
    disable_help_subcommand = true,
    subcommand_required = true
)]
pub(crate) struct FretboardCliContract {
    #[command(subcommand)]
    pub command: FretboardCommandContract,
}

#[derive(Debug, Subcommand)]
pub(crate) enum FretboardCommandContract {
    /// Manage generated asset manifests and Rust glue.
    Assets(AssetsCommandArgs),
    /// Configure workspace-local settings and generated config files.
    Config(ConfigCommandArgs),
    /// Run workspace demos and shells.
    Dev(DevCommandArgs),
    /// Run diagnostics tooling.
    Diag(ForwardedSubcommandArgs),
    /// Manage developer hotpatch helpers.
    Hotpatch(HotpatchCommandArgs),
    /// List discoverable demos and cookbook examples.
    List(ListCommandArgs),
    /// Create a new app from a starter template.
    New(NewCommandArgs),
    /// Import and convert theme sources.
    Theme(ThemeCommandArgs),
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
#[command(disable_help_flag = true, disable_help_subcommand = true)]
pub(crate) struct ForwardedSubcommandArgs {
    /// Remaining arguments forwarded to the legacy handler.
    #[arg(
        value_name = "ARG",
        num_args = 0..,
        allow_hyphen_values = true,
        trailing_var_arg = true
    )]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
#[command(arg_required_else_help = true)]
pub(crate) struct DevCommandArgs {
    #[command(subcommand)]
    pub target: DevTargetContract,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub(crate) enum DevTargetContract {
    /// Run native workspace apps and demos.
    Native(DevNativeCommandArgs),
    /// Run the web demo shell.
    Web(DevWebCommandArgs),
}

#[derive(Debug, Clone, Args, PartialEq, Eq)]
#[command(arg_required_else_help = true)]
pub(crate) struct ListCommandArgs {
    #[command(subcommand)]
    pub target: ListTargetContract,
}

#[derive(Debug, Clone, Subcommand, PartialEq, Eq)]
pub(crate) enum ListTargetContract {
    /// List native demos.
    NativeDemos(ListAllArgs),
    /// List web demos.
    WebDemos(NoArgs),
    /// List cookbook examples.
    CookbookExamples(ListAllArgs),
}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub(crate) struct NoArgs {}

#[derive(Debug, Clone, Args, PartialEq, Eq, Default)]
pub(crate) struct ListAllArgs {
    /// Include maintainer-only or lab targets.
    #[arg(long)]
    pub all: bool,
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn try_parse_contract<I, T>(args: I) -> Result<FretboardCliContract, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    FretboardCliContract::try_parse_from(args)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn render_command_help_path(path: &[&str]) -> Result<String, String> {
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
        "fretboard-dev".to_string()
    } else {
        format!("fretboard-dev {}", path.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use clap::error::ErrorKind;

    use crate::scaffold::contracts::{NewTemplateContract, ScaffoldIconPackValue};

    use super::{
        DevTargetContract, FretboardCommandContract, ListTargetContract, render_command_help_path,
        try_parse_contract,
    };

    #[test]
    fn diag_contract_forwards_help_flags_to_fret_diag() {
        let cli = try_parse_contract(["fretboard-dev", "diag", "--help"])
            .expect("diag --help should forward to fret-diag");

        let FretboardCommandContract::Diag(args) = cli.command else {
            panic!("expected diag command");
        };

        assert_eq!(args.args, vec!["--help"]);
    }

    #[test]
    fn dev_native_contract_captures_selection_and_passthrough_args() {
        let cli = try_parse_contract([
            "fretboard-dev",
            "dev",
            "native",
            "--bin",
            "todo_demo",
            "--watch",
            "--",
            "--help",
        ])
        .expect("dev native should parse typed args");

        let FretboardCommandContract::Dev(dev) = cli.command else {
            panic!("expected dev command");
        };

        let DevTargetContract::Native(args) = dev.target else {
            panic!("expected native dev target");
        };

        assert_eq!(args.bin.as_deref(), Some("todo_demo"));
        assert!(!args.no_strict_runtime);
        assert!(args.watch);
        assert_eq!(args.passthrough, vec!["--help"]);
    }

    #[test]
    fn dev_web_contract_captures_targeting_args() {
        let cli = try_parse_contract([
            "fretboard-dev",
            "dev",
            "web",
            "--port",
            "9001",
            "--demo",
            "plot_demo",
            "--no-strict-runtime",
            "--no-open",
        ])
        .expect("dev web should parse typed args");

        let FretboardCommandContract::Dev(dev) = cli.command else {
            panic!("expected dev command");
        };

        let DevTargetContract::Web(args) = dev.target else {
            panic!("expected web dev target");
        };

        assert_eq!(args.port, Some(9001));
        assert_eq!(args.demo.as_deref(), Some("plot_demo"));
        assert!(args.no_strict_runtime);
        assert!(args.no_open);
    }

    #[test]
    fn list_native_demos_contract_captures_all_flag() {
        let cli = try_parse_contract(["fretboard-dev", "list", "native-demos", "--all"])
            .expect("list native-demos --all should parse");

        let FretboardCommandContract::List(list) = cli.command else {
            panic!("expected list command");
        };

        let ListTargetContract::NativeDemos(args) = list.target else {
            panic!("expected native-demos target");
        };

        assert!(args.all);
    }

    #[test]
    fn list_contract_requires_a_target() {
        let err = try_parse_contract(["fretboard-dev", "list"])
            .expect_err("list should require a target");
        assert_eq!(
            err.kind(),
            ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
        );
    }

    #[test]
    fn dev_help_lists_native_and_web_targets() {
        let help = render_command_help_path(&["dev"]).expect("dev help should render");
        assert!(help.contains("native"));
        assert!(help.contains("web"));
    }

    #[test]
    fn dev_native_help_lists_hotpatch_flags() {
        let help =
            render_command_help_path(&["dev", "native"]).expect("dev native help should render");
        assert!(help.contains("--hotpatch"));
        assert!(help.contains("--hotpatch-dx"));
        assert!(help.contains("--no-strict-runtime"));
        assert!(help.contains("--profile"));
    }

    #[test]
    fn assets_help_lists_manifest_and_rust_targets() {
        let help = render_command_help_path(&["assets"]).expect("assets help should render");
        assert!(help.contains("manifest"));
        assert!(help.contains("rust"));
    }

    #[test]
    fn hotpatch_help_lists_watch_and_status_targets() {
        let help = render_command_help_path(&["hotpatch"]).expect("hotpatch help should render");
        assert!(help.contains("watch"));
        assert!(help.contains("status"));
    }

    #[test]
    fn assets_manifest_write_contract_captures_bundle_selector() {
        let cli = try_parse_contract([
            "fretboard-dev",
            "assets",
            "manifest",
            "write",
            "--dir",
            "assets",
            "--out",
            "assets.manifest.json",
            "--app-bundle",
            "demo-app",
        ])
        .expect("assets manifest write should parse");

        let FretboardCommandContract::Assets(_) = cli.command else {
            panic!("expected assets command");
        };
    }

    #[test]
    fn new_help_lists_scaffold_templates() {
        let help = render_command_help_path(&["new"]).expect("new help should render");
        assert!(help.contains("hello"));
        assert!(help.contains("simple-todo"));
        assert!(help.contains("todo"));
        assert!(help.contains("empty"));
    }

    #[test]
    fn new_contract_without_template_parses_as_wizard_entry() {
        let cli = try_parse_contract(["fretboard-dev", "new"]).expect("new should parse");

        let FretboardCommandContract::New(args) = cli.command else {
            panic!("expected new command");
        };

        assert_eq!(args.template, None);
    }

    #[test]
    fn new_todo_contract_captures_scaffold_flags() {
        let cli = try_parse_contract([
            "fretboard-dev",
            "new",
            "todo",
            "--name",
            "my-todo",
            "--ui-assets",
            "--icons",
            "radix",
            "--command-palette",
            "--no-check",
        ])
        .expect("new todo should parse");

        let FretboardCommandContract::New(args) = cli.command else {
            panic!("expected new command");
        };

        let Some(NewTemplateContract::Todo(args)) = args.template else {
            panic!("expected todo template");
        };

        assert_eq!(args.output.name.as_deref(), Some("my-todo"));
        assert!(args.ui_assets);
        assert_eq!(args.icons.icons, Some(ScaffoldIconPackValue::Radix));
        assert!(args.icons.command_palette);
        assert!(args.output.no_check);
    }

    #[test]
    fn deleted_init_alias_is_rejected() {
        let err = try_parse_contract(["fretboard-dev", "init", "todo"])
            .expect_err("init should be rejected as a deleted alias");
        assert_eq!(err.kind(), ErrorKind::InvalidSubcommand);
    }

    #[test]
    fn config_help_lists_menubar_target() {
        let help = render_command_help_path(&["config"]).expect("config help should render");
        assert!(help.contains("menubar"));
    }

    #[test]
    fn theme_help_lists_import_vscode_target() {
        let help = render_command_help_path(&["theme"]).expect("theme help should render");
        assert!(help.contains("import-vscode"));
    }

    #[test]
    fn theme_import_vscode_contract_captures_positional_input_and_sets() {
        let cli = try_parse_contract([
            "fretboard-dev",
            "theme",
            "import-vscode",
            "theme.json",
            "--set",
            "color.syntax.keyword=#ff00aa",
            "--all-tags",
        ])
        .expect("theme import-vscode should parse");

        let FretboardCommandContract::Theme(_) = cli.command else {
            panic!("expected theme command");
        };
    }
}
