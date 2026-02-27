use std::path::Path;

use crate::script_registry::{PromotedScriptRegistry, promoted_registry_default_path};

#[derive(Debug, Default)]
struct ListScriptsOptions {
    contains: Option<String>,
    case_sensitive: bool,
    all: bool,
}

pub(crate) fn cmd_list(
    rest: &[String],
    workspace_root: &Path,
    json: bool,
    top_override: Option<usize>,
) -> Result<(), String> {
    let Some(kind) = rest.first().map(|s| s.as_str()) else {
        return Err("missing list target (try: fretboard diag list scripts)".to_string());
    };

    match kind {
        "scripts" | "script" => cmd_list_scripts(&rest[1..], workspace_root, json, top_override),
        other => Err(format!("unknown diag list target: {other}")),
    }
}

fn cmd_list_scripts(
    rest: &[String],
    workspace_root: &Path,
    json: bool,
    top_override: Option<usize>,
) -> Result<(), String> {
    let opts = parse_list_scripts_options(rest)?;

    let registry_path = promoted_registry_default_path(workspace_root);
    if !registry_path.is_file() {
        return Err(format!(
            "promoted scripts registry is missing: {}\n\
hint: generate it via `python tools/check_diag_scripts_registry.py --write`",
            registry_path.display()
        ));
    }

    let registry = PromotedScriptRegistry::load_from_path(&registry_path)?;
    let mut entries: Vec<_> = registry.entries().to_vec();
    entries.sort_by(|a, b| a.id.cmp(&b.id));

    if let Some(needle) = opts.contains.as_deref() {
        let needle_lower = needle.to_ascii_lowercase();
        entries.retain(|e| {
            if opts.case_sensitive {
                e.id.contains(needle) || e.path.contains(needle)
            } else {
                e.id.to_ascii_lowercase().contains(&needle_lower)
                    || e.path.to_ascii_lowercase().contains(&needle_lower)
            }
        });
    }

    if !opts.all {
        let limit = top_override.unwrap_or(50);
        entries.truncate(limit);
    }

    if json {
        let payload = serde_json::json!({
            "scripts": entries.iter().map(|e| serde_json::json!({
                "id": e.id,
                "path": e.path,
            })).collect::<Vec<_>>(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
        return Ok(());
    }

    for e in entries {
        println!("{} -> {}", e.id, e.path);
    }

    Ok(())
}

fn parse_list_scripts_options(rest: &[String]) -> Result<ListScriptsOptions, String> {
    let mut out = ListScriptsOptions::default();

    let mut i: usize = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        match arg {
            "--case-sensitive" => {
                out.case_sensitive = true;
                i += 1;
            }
            "--all" => {
                out.all = true;
                i += 1;
            }
            "--contains" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value after --contains".to_string())?;
                out.contains = Some(v.to_string());
                i += 2;
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown diag list scripts flag: {other}"));
            }
            other => {
                return Err(format!(
                    "unexpected positional for `diag list scripts`: {other}\n\
hint: use flags like --contains <needle>, --all, or global flags like --top <n> / --json"
                ));
            }
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_list_scripts_options_happy_path() {
        let opts = parse_list_scripts_options(&s(&["--contains", "ui-gallery"])).unwrap();
        assert_eq!(opts.contains.as_deref(), Some("ui-gallery"));
        assert!(!opts.case_sensitive);
        assert!(!opts.all);
    }

    #[test]
    fn parse_list_scripts_options_rejects_unknown_flag() {
        let err = parse_list_scripts_options(&s(&["--nope"])).unwrap_err();
        assert!(err.contains("unknown"));
    }
}
