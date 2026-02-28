use std::path::Path;

use crate::session::SessionInfo;
use crate::util::now_unix_ms;

#[derive(Debug, Default)]
struct SessionsCleanOptions {
    keep: Option<usize>,
    older_than_days: Option<u64>,
    apply: bool,
}

pub(crate) fn cmd_sessions(
    rest: &[String],
    out_dir: &Path,
    json: bool,
    top_override: Option<usize>,
) -> Result<(), String> {
    let Some(action) = rest.first().map(|s| s.as_str()) else {
        return Err(
            "missing sessions subcommand (try: fretboard diag sessions clean --keep 50)"
                .to_string(),
        );
    };
    match action {
        "clean" => cmd_sessions_clean(&rest[1..], out_dir, json, top_override),
        other => Err(format!("unknown diag sessions subcommand: {other}")),
    }
}

fn parse_sessions_clean_options(rest: &[String]) -> Result<SessionsCleanOptions, String> {
    let mut out = SessionsCleanOptions::default();
    let mut i = 0usize;
    while i < rest.len() {
        let arg = rest[i].as_str();
        match arg {
            "--apply" => {
                out.apply = true;
                i += 1;
            }
            "--keep" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value after --keep".to_string())?;
                let parsed = v
                    .parse::<usize>()
                    .map_err(|_| "invalid value for --keep".to_string())?;
                out.keep = Some(parsed);
                i += 2;
            }
            "--older-than-days" => {
                let v = rest
                    .get(i + 1)
                    .ok_or_else(|| "missing value after --older-than-days".to_string())?;
                let parsed = v
                    .parse::<u64>()
                    .map_err(|_| "invalid value for --older-than-days".to_string())?;
                out.older_than_days = Some(parsed);
                i += 2;
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown diag sessions clean flag: {other}"));
            }
            other => {
                return Err(format!(
                    "unexpected positional for `diag sessions clean`: {other}\n\
hint: use flags like --keep <n>, --older-than-days <n>, and --apply"
                ));
            }
        }
    }
    Ok(out)
}

fn ms_per_day() -> u64 {
    86_400_000
}

fn select_sessions_to_delete(
    sessions: &[SessionInfo],
    keep: Option<usize>,
    older_than_days: Option<u64>,
) -> (Vec<SessionInfo>, Vec<SessionInfo>, Vec<SessionInfo>) {
    let mut kept: Vec<SessionInfo> = Vec::new();
    let mut delete: Vec<SessionInfo> = Vec::new();
    let mut skipped_unknown_age: Vec<SessionInfo> = Vec::new();

    let keep_n = keep.unwrap_or(0);
    for (idx, s) in sessions.iter().cloned().enumerate() {
        if idx < keep_n {
            kept.push(s);
            continue;
        }

        if let Some(days) = older_than_days {
            let threshold = now_unix_ms().saturating_sub(days.saturating_mul(ms_per_day()));
            let Some(created) = s.created_unix_ms else {
                skipped_unknown_age.push(s);
                continue;
            };
            if created <= threshold {
                delete.push(s);
            } else {
                kept.push(s);
            }
        } else {
            delete.push(s);
        }
    }

    (kept, delete, skipped_unknown_age)
}

fn cmd_sessions_clean(
    rest: &[String],
    out_dir: &Path,
    json: bool,
    top_override: Option<usize>,
) -> Result<(), String> {
    let opts = parse_sessions_clean_options(rest)?;
    if opts.keep.is_none() && opts.older_than_days.is_none() {
        return Err(
            "sessions clean requires a selection criterion (use --keep <n> and/or --older-than-days <n>)"
                .to_string(),
        );
    }

    let mut sessions = crate::session::collect_sessions(out_dir)?;
    if let Some(limit) = top_override {
        sessions.truncate(limit);
    }

    let (kept, to_delete, skipped_unknown_age) =
        select_sessions_to_delete(&sessions, opts.keep, opts.older_than_days);

    let sessions_root = out_dir.join(crate::session::SESSIONS_DIRNAME);

    if json {
        let payload = serde_json::json!({
            "base_out_dir": out_dir.display().to_string(),
            "sessions_root": sessions_root.display().to_string(),
            "apply": opts.apply,
            "keep": opts.keep,
            "older_than_days": opts.older_than_days,
            "seen_total": sessions.len(),
            "kept_total": kept.len(),
            "delete_total": to_delete.len(),
            "skipped_unknown_age_total": skipped_unknown_age.len(),
            "delete": to_delete.iter().map(|s| serde_json::json!({
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

    if to_delete.is_empty() {
        println!(
            "sessions clean: nothing to delete (base_out_dir={})",
            out_dir.display()
        );
        return Ok(());
    }

    println!(
        "sessions clean: base_out_dir={} sessions_root={} would_delete_total={} apply={}",
        out_dir.display(),
        sessions_root.display(),
        to_delete.len(),
        opts.apply
    );
    for s in &to_delete {
        println!("  delete {} -> {}", s.session_id, s.session_dir.display());
    }

    if !opts.apply {
        println!("sessions clean: dry-run (pass --apply to delete)");
        return Ok(());
    }

    for s in &to_delete {
        let meta = std::fs::symlink_metadata(&s.session_dir).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() {
            return Err(format!(
                "refusing to delete symlink session dir: {}",
                s.session_dir.display()
            ));
        }
        if !s.session_dir.starts_with(&sessions_root) {
            return Err(format!(
                "refusing to delete path outside sessions root: {}",
                s.session_dir.display()
            ));
        }
        std::fs::remove_dir_all(&s.session_dir).map_err(|e| {
            format!(
                "failed to delete session dir {}: {e}",
                s.session_dir.display()
            )
        })?;
    }

    println!("sessions clean: deleted_total={}", to_delete.len());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parse_sessions_clean_options_happy_path() {
        let opts =
            parse_sessions_clean_options(&["--keep", "10", "--apply"].map(|s| s.to_string()))
                .unwrap();
        assert_eq!(opts.keep, Some(10));
        assert!(opts.apply);
    }

    #[test]
    fn select_sessions_to_delete_respects_keep() {
        let base = std::env::temp_dir().join(format!(
            "fret-diag-sessions-clean-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ));
        let sessions = vec![
            SessionInfo {
                session_id: "3".to_string(),
                session_dir: base.join("3"),
                created_unix_ms: Some(3),
                pid: None,
                diag_subcommand: None,
            },
            SessionInfo {
                session_id: "2".to_string(),
                session_dir: base.join("2"),
                created_unix_ms: Some(2),
                pid: None,
                diag_subcommand: None,
            },
            SessionInfo {
                session_id: "1".to_string(),
                session_dir: base.join("1"),
                created_unix_ms: Some(1),
                pid: None,
                diag_subcommand: None,
            },
        ];
        let (_kept, delete, _skipped) = select_sessions_to_delete(&sessions, Some(1), None);
        assert_eq!(delete.len(), 2);
        assert_eq!(delete[0].session_id, "2");
        assert_eq!(delete[1].session_id, "1");
    }
}
