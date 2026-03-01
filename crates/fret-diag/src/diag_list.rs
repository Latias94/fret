use std::path::Path;

use crate::script_registry::{PromotedScriptRegistry, promoted_registry_default_path};

#[derive(Debug, Default)]
struct ListFilterOptions {
    contains: Option<String>,
    case_sensitive: bool,
    all: bool,
}

pub(crate) fn cmd_list(
    rest: &[String],
    workspace_root: &Path,
    out_dir: &Path,
    json: bool,
    top_override: Option<usize>,
) -> Result<(), String> {
    let Some(kind) = rest.first().map(|s| s.as_str()) else {
        return Err("missing list target (try: fretboard diag list scripts)".to_string());
    };

    match kind {
        "scripts" | "script" => cmd_list_scripts(&rest[1..], workspace_root, json, top_override),
        "suites" | "suite" => cmd_list_suites(&rest[1..], workspace_root, json, top_override),
        "sessions" | "session" => cmd_list_sessions(&rest[1..], out_dir, json, top_override),
        other => Err(format!("unknown diag list target: {other}")),
    }
}

fn cmd_list_scripts(
    rest: &[String],
    workspace_root: &Path,
    json: bool,
    top_override: Option<usize>,
) -> Result<(), String> {
    let opts = parse_list_filter_options("scripts", rest)?;

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

fn cmd_list_suites(
    rest: &[String],
    workspace_root: &Path,
    json: bool,
    top_override: Option<usize>,
) -> Result<(), String> {
    let opts = parse_list_filter_options("suites", rest)?;

    let registry_path = promoted_registry_default_path(workspace_root);
    if !registry_path.is_file() {
        return Err(format!(
            "promoted scripts registry is missing: {}\n\
hint: generate it via `python tools/check_diag_scripts_registry.py --write`",
            registry_path.display()
        ));
    }

    let registry = PromotedScriptRegistry::load_from_path(&registry_path)?;

    use std::collections::BTreeMap;
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for e in registry.entries() {
        for s in &e.suite_memberships {
            *counts.entry(s.as_str()).or_insert(0) += 1;
        }
    }
    let mut suites: Vec<(String, usize)> = counts
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();

    if let Some(needle) = opts.contains.as_deref() {
        let needle_lower = needle.to_ascii_lowercase();
        suites.retain(|(suite, _count)| {
            if opts.case_sensitive {
                suite.contains(needle)
            } else {
                suite.to_ascii_lowercase().contains(&needle_lower)
            }
        });
    }

    if !opts.all {
        let limit = top_override.unwrap_or(50);
        suites.truncate(limit);
    }

    if json {
        let payload = serde_json::json!({
            "suites": suites.iter().map(|(suite, scripts_total)| serde_json::json!({
                "suite": suite,
                "scripts_total": scripts_total,
            })).collect::<Vec<_>>(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
        return Ok(());
    }

    for (suite, scripts_total) in suites {
        println!("{suite} ({scripts_total} scripts)");
    }

    Ok(())
}

fn cmd_list_sessions(
    rest: &[String],
    out_dir: &Path,
    json: bool,
    top_override: Option<usize>,
) -> Result<(), String> {
    let opts = parse_list_filter_options("sessions", rest)?;

    let mut sessions = crate::session::collect_sessions(out_dir)?;

    if let Some(needle) = opts.contains.as_deref() {
        let needle_lower = needle.to_ascii_lowercase();
        sessions.retain(|s| {
            if opts.case_sensitive {
                s.session_id.contains(needle)
                    || s.session_dir.to_string_lossy().contains(needle)
                    || s.diag_subcommand
                        .as_deref()
                        .is_some_and(|v| v.contains(needle))
            } else {
                s.session_id.to_ascii_lowercase().contains(&needle_lower)
                    || s.session_dir
                        .to_string_lossy()
                        .to_ascii_lowercase()
                        .contains(&needle_lower)
                    || s.diag_subcommand
                        .as_deref()
                        .is_some_and(|v| v.to_ascii_lowercase().contains(&needle_lower))
            }
        });
    }

    if !opts.all {
        let limit = top_override.unwrap_or(50);
        sessions.truncate(limit);
    }

    if json {
        let payload = serde_json::json!({
            "sessions": sessions.iter().map(|s| serde_json::json!({
                "session_id": s.session_id,
                "session_dir": s.session_dir.display().to_string(),
                "created_unix_ms": s.created_unix_ms,
                "pid": s.pid,
                "diag_subcommand": s.diag_subcommand,
            })).collect::<Vec<_>>(),
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&payload).map_err(|e| e.to_string())?
        );
        return Ok(());
    }

    for s in sessions {
        let created = s
            .created_unix_ms
            .map(|v| v.to_string())
            .unwrap_or_else(|| "?".to_string());
        let pid = s
            .pid
            .map(|v| v.to_string())
            .unwrap_or_else(|| "?".to_string());
        let sub = s.diag_subcommand.unwrap_or_else(|| "?".to_string());
        println!(
            "{} (created_unix_ms={} pid={} sub={}) -> {}",
            s.session_id,
            created,
            pid,
            sub,
            s.session_dir.display()
        );
    }

    Ok(())
}

fn parse_list_filter_options(kind: &str, rest: &[String]) -> Result<ListFilterOptions, String> {
    let mut out = ListFilterOptions::default();

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
                return Err(format!("unknown diag list {kind} flag: {other}"));
            }
            other => {
                return Err(format!(
                    "unexpected positional for `diag list {kind}`: {other}\n\
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
    use std::time::{SystemTime, UNIX_EPOCH};

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn parse_list_filter_options_happy_path() {
        let opts = parse_list_filter_options("scripts", &s(&["--contains", "ui-gallery"])).unwrap();
        assert_eq!(opts.contains.as_deref(), Some("ui-gallery"));
        assert!(!opts.case_sensitive);
        assert!(!opts.all);
    }

    #[test]
    fn parse_list_filter_options_rejects_unknown_flag() {
        let err = parse_list_filter_options("scripts", &s(&["--nope"])).unwrap_err();
        assert!(err.contains("unknown"));
    }

    #[test]
    fn collect_sessions_sorts_by_created_unix_ms_desc() {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-list-sessions-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let sessions_root = root.join(crate::session::SESSIONS_DIRNAME);
        std::fs::create_dir_all(&sessions_root).expect("create sessions dir");

        let a = sessions_root.join("100-1");
        let b = sessions_root.join("200-1");
        std::fs::create_dir_all(&a).expect("create a");
        std::fs::create_dir_all(&b).expect("create b");
        std::fs::write(
            a.join("session.json"),
            br#"{"schema_version":1,"created_unix_ms":100,"pid":1,"diag_subcommand":"run"}"#,
        )
        .expect("write a");
        std::fs::write(
            b.join("session.json"),
            br#"{"schema_version":1,"created_unix_ms":200,"pid":1,"diag_subcommand":"suite"}"#,
        )
        .expect("write b");

        let sessions = crate::session::collect_sessions(&root).expect("collect sessions");
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].session_id, "200-1");
        assert_eq!(sessions[1].session_id, "100-1");
    }
}
