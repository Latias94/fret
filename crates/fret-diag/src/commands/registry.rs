use std::path::{Path, PathBuf};

use serde_json::json;

use crate::promoted_registry_builder::promoted_registry_expected_bytes;
use crate::script_registry::promoted_registry_default_path;

fn normalize_text_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut s = String::from_utf8_lossy(bytes).to_string();
    s = s.replace("\r\n", "\n");
    if !s.ends_with('\n') {
        s.push('\n');
    }
    s.into_bytes()
}

fn parse_path_flag(rest: &[String]) -> Result<(Option<PathBuf>, Vec<String>), String> {
    let mut out: Vec<String> = Vec::new();
    let mut path: Option<PathBuf> = None;

    let mut i: usize = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--path" => {
                let Some(v) = rest.get(i + 1) else {
                    return Err("missing value for --path".to_string());
                };
                path = Some(PathBuf::from(v));
                i += 2;
            }
            other if other.starts_with("--path=") => {
                let v = other.trim_start_matches("--path=");
                path = Some(PathBuf::from(v));
                i += 1;
            }
            _ => {
                out.push(rest[i].clone());
                i += 1;
            }
        }
    }

    Ok((path, out))
}

pub(crate) fn cmd_registry(
    rest: &[String],
    workspace_root: &Path,
    stats_json: bool,
) -> Result<(), String> {
    let Some(sub) = rest.first().map(|s| s.as_str()) else {
        return Err("internal error: diag registry requires a subcommand".to_string());
    };

    let (path_override, passthrough) = parse_path_flag(&rest[1..])?;
    if !passthrough.is_empty() {
        return Err(format!(
            "unexpected args for diag registry {sub}: {}",
            passthrough.join(" ")
        ));
    }

    let default_path = promoted_registry_default_path(workspace_root);
    let path = path_override.unwrap_or(default_path);

    match sub {
        "print" => {
            let expected = promoted_registry_expected_bytes(workspace_root)?;
            print!("{}", String::from_utf8_lossy(&expected));
            Ok(())
        }
        "write" => {
            let expected = promoted_registry_expected_bytes(workspace_root)?;
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(&path, expected).map_err(|e| e.to_string())?;
            if stats_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "ok": true,
                        "wrote": path.display().to_string(),
                    }))
                    .unwrap_or_else(|_| "{}".to_string())
                );
            } else {
                println!("wrote: {}", path.display());
            }
            Ok(())
        }
        "check" => {
            if !path.is_file() {
                return Err(format!(
                    "promoted scripts registry is missing: {}\n\
hint: generate it via `cargo run -p fretboard-dev -- diag registry write`",
                    path.display()
                ));
            }

            let expected = promoted_registry_expected_bytes(workspace_root)?;
            let actual = std::fs::read(&path).map_err(|e| e.to_string())?;
            let expected_norm = normalize_text_bytes(&expected);
            let actual_norm = normalize_text_bytes(&actual);

            let ok = actual_norm == expected_norm;
            if stats_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "ok": ok,
                        "registry_path": path.display().to_string(),
                    }))
                    .unwrap_or_else(|_| "{}".to_string())
                );
                if ok {
                    return Ok(());
                }
                return Err("diag script registry is out of date".to_string());
            }

            if !ok {
                return Err(format!(
                    "diag script registry is out of date:\n\
- file: {}\n\
hint: run `cargo run -p fretboard-dev -- diag registry write`",
                    path.display()
                ));
            }
            println!("ok: diag script registry is up to date.");
            Ok(())
        }
        other => Err(format!("unknown diag registry subcommand: {other}")),
    }
}
