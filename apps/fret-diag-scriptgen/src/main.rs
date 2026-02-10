use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use fret_diag_protocol::UiActionScriptV2;
use fret_diag_protocol::builder::{ScriptV2Builder, role_and_name, test_id};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let Some(cmd) = args.next() else {
        return help();
    };

    match cmd.as_str() {
        "help" | "-h" | "--help" => help(),
        "list" => {
            for name in template_names() {
                println!("{name}");
            }
            Ok(())
        }
        "print" => {
            let name = args
                .next()
                .ok_or_else(|| "missing template name (try: list)".to_string())?;
            let script = template_v2(&name)?;
            println!("{}", to_pretty_json(&script)?);
            Ok(())
        }
        "write" => {
            let name = args
                .next()
                .ok_or_else(|| "missing template name (try: list)".to_string())?;

            let mut out: Option<PathBuf> = None;
            let mut overwrite = false;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--out" => {
                        let path = args
                            .next()
                            .ok_or_else(|| "missing value for --out".to_string())?;
                        out = Some(PathBuf::from(path));
                    }
                    "--overwrite" => overwrite = true,
                    other => return Err(format!("unknown arg: {other}")),
                }
            }

            let workspace_root = workspace_root()?;
            let out = out.unwrap_or_else(|| {
                workspace_root
                    .join(".fret")
                    .join("diag")
                    .join("scripts")
                    .join(format!("{name}.json"))
            });

            let script = template_v2(&name)?;
            write_json_file(&out, &script, overwrite)?;
            println!("{}", out.display());
            Ok(())
        }
        "check" => {
            let name = args
                .next()
                .ok_or_else(|| "missing template name (try: list)".to_string())?;
            let path = args
                .next()
                .ok_or_else(|| "missing json path to compare".to_string())?;

            let script = template_v2(&name)?;
            let expected = serde_json::to_value(&script).map_err(|e| e.to_string())?;
            let actual_text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let actual: serde_json::Value = serde_json::from_str(&actual_text).map_err(|e| {
                format!(
                    "failed to parse json at {path}: {e}",
                    path = Path::new(&path).display()
                )
            })?;

            if expected != actual {
                return Err("json differs (compare with: print)".to_string());
            }
            Ok(())
        }
        other => Err(format!("unknown command: {other}")),
    }
}

fn help() -> Result<(), String> {
    println!(
        r#"fret-diag-scriptgen - generate typed UI diag scripts (JSON) from Rust templates

Usage:
  fret-diag-scriptgen help
  fret-diag-scriptgen list
  fret-diag-scriptgen print <template>
  fret-diag-scriptgen write <template> [--out <path>] [--overwrite]
  fret-diag-scriptgen check <template> <json_path>

Notes:
  - The default output path is `.fret/diag/scripts/<template>.json` under the workspace root.
  - These scripts are compatible with `fretboard diag run <script.json>`.
"#
    );
    Ok(())
}

fn template_names() -> &'static [&'static str] {
    &[
        "todo-baseline-v2",
        "ui-gallery-command-palette-shortcut-primary-v2",
    ]
}

fn template_v2(name: &str) -> Result<UiActionScriptV2, String> {
    match name {
        "todo-baseline-v2" => Ok(todo_baseline_v2()),
        "ui-gallery-command-palette-shortcut-primary-v2" => {
            Ok(ui_gallery_command_palette_shortcut_primary_v2())
        }
        other => Err(format!("unknown template: {other} (try: list)")),
    }
}

fn todo_baseline_v2() -> UiActionScriptV2 {
    ScriptV2Builder::new()
        .type_text_into(test_id("todo-input"), "Automated task")
        .wait_frames(2)
        .press_key("enter")
        .wait_exists(test_id("todo-item-4-done"), 60)
        .capture_bundle(Some("todo-after-add".to_string()))
        .click(test_id("todo-item-4-done"))
        .wait_frames(2)
        .capture_bundle(Some("todo-after-toggle-done".to_string()))
        .click(test_id("todo-item-4-remove"))
        .wait_frames(2)
        .capture_bundle(Some("todo-after-remove".to_string()))
        .build()
}

fn ui_gallery_command_palette_shortcut_primary_v2() -> UiActionScriptV2 {
    let dialog = role_and_name("dialog", "Command palette");
    ScriptV2Builder::new()
        .press_key("escape")
        .wait_frames(2)
        .press_shortcut("primary+p")
        .wait_exists(dialog.clone(), 240)
        .assert_exists(dialog.clone())
        .press_key("escape")
        .wait_not_exists(dialog, 240)
        .capture_bundle(Some(
            "ui-gallery-command-palette-shortcut-primary".to_string(),
        ))
        .build()
}

fn to_pretty_json(script: &UiActionScriptV2) -> Result<String, String> {
    serde_json::to_string_pretty(script).map_err(|e| e.to_string())
}

fn write_json_file(path: &Path, script: &UiActionScriptV2, overwrite: bool) -> Result<(), String> {
    if path.exists() && !overwrite {
        return Err(format!(
            "refusing to overwrite existing file: {path} (pass --overwrite)",
            path = path.display()
        ));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let mut text = to_pretty_json(script)?;
    text.push('\n');
    fs::write(path, text).map_err(|e| e.to_string())?;
    Ok(())
}

fn workspace_root() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    for dir in cwd.ancestors() {
        if dir.join("Cargo.toml").is_file() {
            return Ok(dir.to_path_buf());
        }
    }
    Err("failed to locate workspace root (Cargo.toml not found in ancestors)".to_string())
}
