use std::path::{Path, PathBuf};
use std::process::Command;

use crate::cli::workspace_root;

pub(crate) mod contracts;

mod fs;
mod templates;
mod wizard;

use self::contracts::{
    NewCommandArgs, NewTemplateContract, ScaffoldEmptyCommandArgs, ScaffoldHelloCommandArgs,
    ScaffoldIconArgs, ScaffoldIconPackValue, ScaffoldOutputArgs, ScaffoldTodoCommandArgs,
};
use fs::{
    ensure_dir_is_new_or_empty, sanitize_package_name, workspace_prefix_from_out_dir,
    write_file_if_missing, write_new_file,
};
use templates::{
    empty_template_cargo_toml, empty_template_main_rs, empty_template_readme_md,
    generated_assets_stub_rs, hello_template_cargo_toml, hello_template_main_rs,
    hello_template_readme_md, simple_todo_template_cargo_toml, simple_todo_template_main_rs,
    simple_todo_template_readme_md, template_gitignore, todo_template_cargo_toml,
    todo_template_main_rs, todo_template_readme_md,
};

pub(crate) fn run_new_contract(args: NewCommandArgs) -> Result<(), String> {
    let Some(template) = args.template else {
        return wizard::new_wizard();
    };

    let root = workspace_root()?;
    match template {
        NewTemplateContract::Empty(args) => run_empty_contract(&root, args),
        NewTemplateContract::Hello(args) => run_hello_contract(&root, args),
        NewTemplateContract::SimpleTodo(args) => run_simple_todo_contract(&root, args),
        NewTemplateContract::Todo(args) => run_todo_contract(&root, args),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NewTemplate {
    Empty,
    Hello,
    SimpleTodo,
    Todo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IconPack {
    None,
    Lucide,
    Radix,
}

impl IconPack {
    fn as_str(self) -> &'static str {
        match self {
            IconPack::None => "none",
            IconPack::Lucide => "lucide",
            IconPack::Radix => "radix",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScaffoldOptions {
    icon_pack: IconPack,
    command_palette: bool,
    ui_assets: bool,
}

fn run_empty_contract(workspace_root: &Path, args: ScaffoldEmptyCommandArgs) -> Result<(), String> {
    let (out_dir, package_name, run_check) =
        resolve_scaffold_output(workspace_root, args.output, "my-app")?;
    init_empty_at(&out_dir, &package_name, run_check)
}

fn run_todo_contract(workspace_root: &Path, args: ScaffoldTodoCommandArgs) -> Result<(), String> {
    let (out_dir, package_name, run_check) =
        resolve_scaffold_output(workspace_root, args.output, "todo-app")?;
    init_todo_at(
        workspace_root,
        &out_dir,
        &package_name,
        scaffold_options_from_icon_args(args.icons, args.ui_assets),
        run_check,
    )
}

fn run_simple_todo_contract(
    workspace_root: &Path,
    args: ScaffoldTodoCommandArgs,
) -> Result<(), String> {
    let (out_dir, package_name, run_check) =
        resolve_scaffold_output(workspace_root, args.output, "simple-todo-app")?;
    init_simple_todo_at(
        workspace_root,
        &out_dir,
        &package_name,
        scaffold_options_from_icon_args(args.icons, args.ui_assets),
        run_check,
    )
}

fn run_hello_contract(workspace_root: &Path, args: ScaffoldHelloCommandArgs) -> Result<(), String> {
    let (out_dir, package_name, run_check) =
        resolve_scaffold_output(workspace_root, args.output, "hello-world")?;
    init_hello_at(
        workspace_root,
        &out_dir,
        &package_name,
        scaffold_options_from_icon_args(args.icons, false),
        run_check,
    )
}

fn resolve_scaffold_output(
    workspace_root: &Path,
    args: ScaffoldOutputArgs,
    default_name: &str,
) -> Result<(PathBuf, String, bool), String> {
    let package_name = sanitize_package_name(args.name.as_deref().unwrap_or(default_name))?;
    let out_dir = args
        .path
        .unwrap_or_else(|| workspace_root.join("local").join(&package_name));
    Ok((out_dir, package_name, !args.no_check))
}

fn scaffold_options_from_icon_args(args: ScaffoldIconArgs, ui_assets: bool) -> ScaffoldOptions {
    ScaffoldOptions {
        icon_pack: icon_pack_from_args(&args),
        command_palette: args.command_palette,
        ui_assets,
    }
}

fn icon_pack_from_args(args: &ScaffoldIconArgs) -> IconPack {
    if args.no_icons {
        return IconPack::None;
    }

    match args.icons.unwrap_or(ScaffoldIconPackValue::Lucide) {
        ScaffoldIconPackValue::Lucide => IconPack::Lucide,
        ScaffoldIconPackValue::Radix => IconPack::Radix,
        ScaffoldIconPackValue::None => IconPack::None,
    }
}

fn init_simple_todo_at(
    workspace_root: &Path,
    out_dir: &Path,
    package_name: &str,
    opts: ScaffoldOptions,
    run_check: bool,
) -> Result<(), String> {
    ensure_dir_is_new_or_empty(out_dir)?;

    let workspace_prefix = workspace_prefix_from_out_dir(workspace_root, out_dir)?;

    let cargo_toml = simple_todo_template_cargo_toml(package_name, opts, &workspace_prefix);
    write_new_file(&out_dir.join("Cargo.toml"), &cargo_toml)?;
    write_file_if_missing(&out_dir.join(".gitignore"), template_gitignore())?;

    let src_dir = out_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;
    write_new_file(
        &src_dir.join("main.rs"),
        &simple_todo_template_main_rs(package_name, opts),
    )?;
    write_new_file(
        &out_dir.join("README.md"),
        &simple_todo_template_readme_md(package_name, opts),
    )?;
    maybe_init_asset_scaffold(out_dir, package_name, opts)?;

    maybe_cargo_check(out_dir, run_check)?;

    println!("Initialized simple-todo template at: {}", out_dir.display());
    println!("Next:");
    println!(
        "  cargo run --manifest-path {}",
        out_dir.join("Cargo.toml").display()
    );
    Ok(())
}

fn init_todo_at(
    workspace_root: &Path,
    out_dir: &Path,
    package_name: &str,
    opts: ScaffoldOptions,
    run_check: bool,
) -> Result<(), String> {
    ensure_dir_is_new_or_empty(out_dir)?;

    let workspace_prefix = workspace_prefix_from_out_dir(workspace_root, out_dir)?;

    let cargo_toml = todo_template_cargo_toml(package_name, opts, &workspace_prefix);
    write_new_file(&out_dir.join("Cargo.toml"), &cargo_toml)?;
    write_file_if_missing(&out_dir.join(".gitignore"), template_gitignore())?;

    let src_dir = out_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;
    write_new_file(
        &src_dir.join("main.rs"),
        &todo_template_main_rs(package_name, opts),
    )?;
    write_new_file(
        &out_dir.join("README.md"),
        &todo_template_readme_md(package_name, opts),
    )?;
    maybe_init_asset_scaffold(out_dir, package_name, opts)?;

    maybe_cargo_check(out_dir, run_check)?;

    println!("Initialized todo template at: {}", out_dir.display());
    println!("Next:");
    println!(
        "  cargo run --manifest-path {}",
        out_dir.join("Cargo.toml").display()
    );
    Ok(())
}

fn init_hello_at(
    workspace_root: &Path,
    out_dir: &Path,
    package_name: &str,
    opts: ScaffoldOptions,
    run_check: bool,
) -> Result<(), String> {
    ensure_dir_is_new_or_empty(out_dir)?;

    let workspace_prefix = workspace_prefix_from_out_dir(workspace_root, out_dir)?;

    let cargo_toml = hello_template_cargo_toml(package_name, opts, &workspace_prefix);
    write_new_file(&out_dir.join("Cargo.toml"), &cargo_toml)?;
    write_file_if_missing(&out_dir.join(".gitignore"), template_gitignore())?;

    let src_dir = out_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;
    write_new_file(
        &src_dir.join("main.rs"),
        &hello_template_main_rs(package_name, opts),
    )?;
    write_new_file(
        &out_dir.join("README.md"),
        &hello_template_readme_md(package_name, opts),
    )?;

    maybe_cargo_check(out_dir, run_check)?;

    println!("Initialized hello template at: {}", out_dir.display());
    println!("Next:");
    println!(
        "  cargo run --manifest-path {}",
        out_dir.join("Cargo.toml").display()
    );
    Ok(())
}

fn init_empty_at(out_dir: &Path, package_name: &str, run_check: bool) -> Result<(), String> {
    ensure_dir_is_new_or_empty(out_dir)?;

    let cargo_toml = empty_template_cargo_toml(package_name);
    write_new_file(&out_dir.join("Cargo.toml"), &cargo_toml)?;
    write_file_if_missing(&out_dir.join(".gitignore"), template_gitignore())?;

    let src_dir = out_dir.join("src");
    std::fs::create_dir_all(&src_dir).map_err(|e| e.to_string())?;
    write_new_file(&src_dir.join("main.rs"), empty_template_main_rs())?;
    write_new_file(
        &out_dir.join("README.md"),
        &empty_template_readme_md(package_name),
    )?;

    maybe_cargo_check(out_dir, run_check)?;

    println!("Initialized empty template at: {}", out_dir.display());
    println!("Next:");
    println!(
        "  cargo run --manifest-path {}",
        out_dir.join("Cargo.toml").display()
    );
    Ok(())
}

fn maybe_cargo_check(out_dir: &Path, run_check: bool) -> Result<(), String> {
    if !run_check {
        return Ok(());
    }

    println!("Running cargo check...");
    let status = Command::new("cargo")
        .arg("check")
        .current_dir(out_dir)
        .status()
        .map_err(|e| format!("failed to spawn cargo check: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo check failed with status: {status}"))
    }
}

fn maybe_init_asset_scaffold(
    out_dir: &Path,
    package_name: &str,
    opts: ScaffoldOptions,
) -> Result<(), String> {
    if !opts.ui_assets {
        return Ok(());
    }

    std::fs::create_dir_all(out_dir.join("assets")).map_err(|e| {
        format!(
            "failed to create default asset directory `{}`: {e}",
            out_dir.join("assets").display()
        )
    })?;

    write_new_file(
        &out_dir.join("src/generated_assets.rs"),
        &generated_assets_stub_rs(package_name),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("{prefix}-{nonce}"));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn repo_workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("repo workspace root should resolve")
    }

    fn make_repo_local_dir(prefix: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let dir = repo_workspace_root()
            .join("local")
            .join(format!("{prefix}-{nonce}"));
        std::fs::create_dir_all(&dir).expect("create repo-local test dir");
        dir
    }

    fn cargo_check_generated_app(out_dir: &Path) {
        let repo_root = repo_workspace_root();
        let target_name = out_dir
            .file_name()
            .and_then(OsStr::to_str)
            .expect("generated app dir should have a final path segment");
        let target_dir = repo_root
            .join("target")
            .join("fretboard-generated-app-checks")
            .join(target_name);

        let status = Command::new("cargo")
            .arg("check")
            .arg("--quiet")
            .current_dir(out_dir)
            .env("CARGO_TARGET_DIR", &target_dir)
            .status()
            .expect("spawn cargo check for generated app");

        assert!(
            status.success(),
            "generated app cargo check failed for {} with status {status}",
            out_dir.display()
        );
    }

    #[derive(Debug, Clone, Copy)]
    struct ScaffoldCompileCase {
        template: NewTemplate,
        package_name: &'static str,
        opts: ScaffoldOptions,
    }

    fn scaffold_template_case(
        workspace_root: &Path,
        suite_root: &Path,
        case: ScaffoldCompileCase,
    ) -> PathBuf {
        let out_dir = suite_root.join(case.package_name);
        let result = match case.template {
            NewTemplate::Empty => init_empty_at(&out_dir, case.package_name, false),
            NewTemplate::Hello => init_hello_at(
                workspace_root,
                &out_dir,
                case.package_name,
                case.opts,
                false,
            ),
            NewTemplate::SimpleTodo => init_simple_todo_at(
                workspace_root,
                &out_dir,
                case.package_name,
                case.opts,
                false,
            ),
            NewTemplate::Todo => init_todo_at(
                workspace_root,
                &out_dir,
                case.package_name,
                case.opts,
                false,
            ),
        };

        result.unwrap_or_else(|err| {
            panic!(
                "scaffold should succeed for {:?} at {}: {err}",
                case.template,
                out_dir.display()
            )
        });
        out_dir
    }

    fn opts_with_ui_assets() -> ScaffoldOptions {
        ScaffoldOptions {
            icon_pack: IconPack::Lucide,
            command_palette: false,
            ui_assets: true,
        }
    }

    #[test]
    fn todo_scaffold_with_ui_assets_creates_generated_assets_stub() {
        let workspace_root = make_temp_dir("fretboard-scaffold-todo-assets");
        let out_dir = workspace_root.join("local").join("todo-app");

        init_todo_at(
            &workspace_root,
            &out_dir,
            "todo-app",
            opts_with_ui_assets(),
            false,
        )
        .expect("todo scaffold should succeed");

        assert!(out_dir.join("assets").is_dir());
        let main_rs = std::fs::read_to_string(out_dir.join("src/main.rs"))
            .expect("generated main.rs should exist");
        assert!(main_rs.contains("mod generated_assets;"));
        assert!(main_rs.contains("generated_assets::mount(builder)?"));

        let generated_assets = std::fs::read_to_string(out_dir.join("src/generated_assets.rs"))
            .expect("generated assets stub should exist");
        assert!(generated_assets.contains("AssetBundleId::app(\"todo-app\")"));
        assert!(
            generated_assets.contains(
                "pub fn mount<S: 'static>(builder: fret::UiAppBuilder<S>) -> fret::Result<fret::UiAppBuilder<S>>"
            )
        );
        assert!(generated_assets.contains("pub fn preferred_startup_plan() -> AssetStartupPlan"));
        assert!(
            generated_assets.contains("pub const fn preferred_startup_mode() -> AssetStartupMode")
        );

        let readme = std::fs::read_to_string(out_dir.join("README.md"))
            .expect("generated README.md should exist");
        assert!(readme.contains("`generated_assets::mount(builder)?`"));
        assert!(readme.contains("`preferred_startup_plan()` / `preferred_startup_mode()`"));
        assert!(readme.contains(
            "`fretboard assets rust write --dir assets --out src/generated_assets.rs --app-bundle todo-app --force`"
        ));
        assert!(readme.contains("`AssetBundleId::app(\"todo-app\")`"));
    }

    #[test]
    fn simple_todo_scaffold_without_ui_assets_skips_default_assets_dir() {
        let workspace_root = make_temp_dir("fretboard-scaffold-simple-todo-no-assets");
        let out_dir = workspace_root.join("local").join("simple-todo-app");

        init_simple_todo_at(
            &workspace_root,
            &out_dir,
            "simple-todo-app",
            ScaffoldOptions {
                icon_pack: IconPack::Lucide,
                command_palette: false,
                ui_assets: false,
            },
            false,
        )
        .expect("simple todo scaffold should succeed");

        assert!(!out_dir.join("assets").exists());
    }

    #[test]
    fn default_onboarding_templates_generate_projects_that_compile() {
        // Generated app manifests rely on repo-relative path dependencies, so this test scaffolds
        // inside the real workspace under the ignored `local/` tree and then runs cargo check
        // against the generated manifests.
        let workspace_root = repo_workspace_root();
        let suite_root = make_repo_local_dir("fretboard-scaffold-compile");
        let cases = [
            ScaffoldCompileCase {
                template: NewTemplate::Hello,
                package_name: "hello-app",
                opts: ScaffoldOptions {
                    icon_pack: IconPack::Lucide,
                    command_palette: false,
                    ui_assets: false,
                },
            },
            ScaffoldCompileCase {
                template: NewTemplate::SimpleTodo,
                package_name: "simple-todo-app",
                opts: ScaffoldOptions {
                    icon_pack: IconPack::Lucide,
                    command_palette: false,
                    ui_assets: false,
                },
            },
            ScaffoldCompileCase {
                template: NewTemplate::Todo,
                package_name: "todo-app",
                opts: ScaffoldOptions {
                    icon_pack: IconPack::Lucide,
                    command_palette: false,
                    ui_assets: false,
                },
            },
        ];

        for case in cases {
            let out_dir = scaffold_template_case(&workspace_root, &suite_root, case);
            cargo_check_generated_app(&out_dir);
        }
    }

    #[test]
    fn key_scaffold_variants_generate_projects_that_compile() {
        let workspace_root = repo_workspace_root();
        let suite_root = make_repo_local_dir("fretboard-scaffold-variants");
        let cases = [
            ScaffoldCompileCase {
                template: NewTemplate::Hello,
                package_name: "hello-radix-palette",
                opts: ScaffoldOptions {
                    icon_pack: IconPack::Radix,
                    command_palette: true,
                    ui_assets: false,
                },
            },
            ScaffoldCompileCase {
                template: NewTemplate::SimpleTodo,
                package_name: "simple-todo-assets-palette",
                opts: ScaffoldOptions {
                    icon_pack: IconPack::Lucide,
                    command_palette: true,
                    ui_assets: true,
                },
            },
            ScaffoldCompileCase {
                template: NewTemplate::Todo,
                package_name: "todo-radix-assets-palette",
                opts: ScaffoldOptions {
                    icon_pack: IconPack::Radix,
                    command_palette: true,
                    ui_assets: true,
                },
            },
        ];

        for case in cases {
            let out_dir = scaffold_template_case(&workspace_root, &suite_root, case);
            cargo_check_generated_app(&out_dir);
        }
    }
}
