use std::path::{Path, PathBuf};

use crate::cli::{help, workspace_root};

mod fs;
mod templates;
mod wizard;

use fs::{
    ensure_dir_is_new_or_empty, sanitize_package_name, workspace_prefix_from_out_dir,
    write_file_if_missing, write_new_file,
};
use templates::{
    empty_template_cargo_toml, empty_template_main_rs, empty_template_readme_md,
    hello_template_cargo_toml, hello_template_main_rs, hello_template_readme_md,
    template_gitignore, todo_template_cargo_toml, todo_template_main_rs, todo_template_readme_md,
};

pub(crate) fn init_cmd(args: Vec<String>) -> Result<(), String> {
    new_template_cmd("init", args)
}

pub(crate) fn new_cmd(args: Vec<String>) -> Result<(), String> {
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        return help();
    }
    if args.is_empty() {
        return wizard::new_wizard();
    }
    new_template_cmd("new", args)
}

fn new_template_cmd(invoked_as: &str, args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    let Some(template) = it.next() else {
        return Err(format!("missing template (try: {invoked_as} todo)"));
    };
    if matches!(template.as_str(), "--help" | "-h") {
        return help();
    }

    match template.as_str() {
        "empty" => init_empty(it.collect()),
        "todo" => init_todo(it.collect()),
        "hello" | "hello-world" => init_hello(it.collect()),
        other => Err(format!("unknown template for {invoked_as}: {other}")),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NewTemplate {
    Empty,
    Hello,
    Todo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IconPack {
    None,
    Lucide,
    Radix,
}

impl IconPack {
    fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "none" | "no" | "off" | "false" => Ok(Self::None),
            "lucide" => Ok(Self::Lucide),
            "radix" => Ok(Self::Radix),
            other => Err(format!(
                "unknown icon pack: {other} (expected: lucide|radix|none)"
            )),
        }
    }

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

fn init_empty(args: Vec<String>) -> Result<(), String> {
    let root = workspace_root()?;

    let mut out_path: Option<PathBuf> = None;
    let mut name: Option<String> = None;

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--path" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--path requires a value".to_string())?;
                out_path = Some(PathBuf::from(raw));
            }
            "--name" => {
                name = Some(
                    it.next()
                        .ok_or_else(|| "--name requires a value".to_string())?,
                );
            }
            "--help" | "-h" => return help(),
            other => return Err(format!("unknown argument for init empty: {other}")),
        }
    }

    let package_name = sanitize_package_name(name.as_deref().unwrap_or("my-app"))?;

    let out_dir = out_path.unwrap_or_else(|| root.join("local").join(&package_name));
    init_empty_at(&out_dir, &package_name)
}

fn init_todo(args: Vec<String>) -> Result<(), String> {
    let root = workspace_root()?;

    let mut out_path: Option<PathBuf> = None;
    let mut name: Option<String> = None;
    let mut ui_assets = false;
    let mut icon_pack = IconPack::Lucide;
    let mut command_palette = false;

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--path" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--path requires a value".to_string())?;
                out_path = Some(PathBuf::from(raw));
            }
            "--name" => {
                name = Some(
                    it.next()
                        .ok_or_else(|| "--name requires a value".to_string())?,
                );
            }
            "--ui-assets" => ui_assets = true,
            "--icons" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--icons requires a value".to_string())?;
                icon_pack = IconPack::parse(&raw)?;
            }
            "--no-icons" => icon_pack = IconPack::None,
            "--command-palette" => command_palette = true,
            "--help" | "-h" => return help(),
            other => return Err(format!("unknown argument for init todo: {other}")),
        }
    }

    let package_name = sanitize_package_name(name.as_deref().unwrap_or("todo-app"))?;

    let out_dir = out_path.unwrap_or_else(|| root.join("local").join(&package_name));
    init_todo_at(
        &root,
        &out_dir,
        &package_name,
        ScaffoldOptions {
            icon_pack,
            command_palette,
            ui_assets,
        },
    )
}

fn init_todo_at(
    workspace_root: &Path,
    out_dir: &Path,
    package_name: &str,
    opts: ScaffoldOptions,
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

    println!("Initialized todo template at: {}", out_dir.display());
    println!("Next:");
    println!(
        "  cargo run --manifest-path {}",
        out_dir.join("Cargo.toml").display()
    );
    Ok(())
}

fn init_hello(args: Vec<String>) -> Result<(), String> {
    let root = workspace_root()?;

    let mut out_path: Option<PathBuf> = None;
    let mut name: Option<String> = None;
    let mut icon_pack = IconPack::Lucide;
    let mut command_palette = false;

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--path" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--path requires a value".to_string())?;
                out_path = Some(PathBuf::from(raw));
            }
            "--name" => {
                name = Some(
                    it.next()
                        .ok_or_else(|| "--name requires a value".to_string())?,
                );
            }
            "--icons" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--icons requires a value".to_string())?;
                icon_pack = IconPack::parse(&raw)?;
            }
            "--no-icons" => icon_pack = IconPack::None,
            "--command-palette" => command_palette = true,
            "--help" | "-h" => return help(),
            other => return Err(format!("unknown argument for init hello: {other}")),
        }
    }

    let package_name = sanitize_package_name(name.as_deref().unwrap_or("hello-world"))?;

    let out_dir = out_path.unwrap_or_else(|| root.join("local").join(&package_name));
    init_hello_at(
        &root,
        &out_dir,
        &package_name,
        ScaffoldOptions {
            icon_pack,
            command_palette,
            ui_assets: false,
        },
    )
}

fn init_hello_at(
    workspace_root: &Path,
    out_dir: &Path,
    package_name: &str,
    opts: ScaffoldOptions,
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

    println!("Initialized hello template at: {}", out_dir.display());
    println!("Next:");
    println!(
        "  cargo run --manifest-path {}",
        out_dir.join("Cargo.toml").display()
    );
    Ok(())
}

fn init_empty_at(out_dir: &Path, package_name: &str) -> Result<(), String> {
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

    println!("Initialized empty template at: {}", out_dir.display());
    println!("Next:");
    println!(
        "  cargo run --manifest-path {}",
        out_dir.join("Cargo.toml").display()
    );
    Ok(())
}
