use std::path::{Path, PathBuf};

use crate::cli::workspace_root;

pub(crate) fn hotpatch_cmd(args: Vec<String>) -> Result<(), String> {
    let mut it = args.into_iter();
    match it.next().as_deref() {
        Some("poke") => {
            let path = parse_hotpatch_path_arg(&mut it)?;
            hotpatch_poke(path.as_deref())
        }
        Some("path") => {
            let root = workspace_root()?;
            let path = parse_hotpatch_path_arg(&mut it)?;
            let path = path
                .as_deref()
                .map(|p| resolve_workspace_relative(&root, p))
                .unwrap_or_else(|| hotpatch_trigger_path(&root));
            println!("{}", path.display());
            Ok(())
        }
        Some("status") => hotpatch_status(it.collect()),
        Some("watch") => hotpatch_watch(it.collect()),
        Some("help") | Some("-h") | Some("--help") | None => {
            println!(
                r#"Usage:
  fretboard hotpatch poke [--path <path>]   # update the trigger file (causes runner reload when enabled)
  fretboard hotpatch path [--path <path>]   # print the trigger file path
  fretboard hotpatch status [--tail <n>]    # show hotpatch-related log tails (read-only)
  fretboard hotpatch watch [--path <path>...] [--trigger-path <path>] [--poll-ms <ms>] [--debounce-ms <ms>]

Notes:
  - Requires running the app with `--hotpatch` (sets `FRET_HOTPATCH=1`).
  - The runner watches `FRET_HOTPATCH_TRIGGER_PATH` (default: `.fret/hotpatch.touch`).
  - `watch` is polling-based and ignores `target/`, `.git/`, `.fret/`, and `repo-ref/`."#
            );
            Ok(())
        }
        Some(other) => Err(format!("unknown hotpatch subcommand: {other}")),
    }
}

fn hotpatch_trigger_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".fret").join("hotpatch.touch")
}

fn hotpatch_runner_log_paths(workspace_root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    paths.push(workspace_root.join(".fret").join("hotpatch_runner.log"));

    let tmp = std::env::temp_dir();
    if !tmp.as_os_str().is_empty() {
        paths.push(tmp.join("fret").join("hotpatch_runner.log"));
    }
    paths
}

fn hotpatch_bootstrap_log_paths(workspace_root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    paths.push(workspace_root.join(".fret").join("hotpatch_bootstrap.log"));

    let tmp = std::env::temp_dir();
    if !tmp.as_os_str().is_empty() {
        paths.push(tmp.join("fret").join("hotpatch_bootstrap.log"));
    }
    paths
}

fn hotpatch_status(args: Vec<String>) -> Result<(), String> {
    let root = workspace_root()?;

    let mut tail: usize = 40;

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--tail" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--tail requires a value".to_string())?;
                tail = raw.parse::<usize>().map_err(|e| e.to_string())?;
            }
            "--help" | "-h" => return Ok(()),
            other => return Err(format!("unknown argument for hotpatch status: {other}")),
        }
    }

    println!("Hotpatch status (read-only):");
    println!("  workspace: {}", root.display());
    println!("  tail: {tail}");

    let legacy_direct =
        std::env::var_os("FRET_HOTPATCH_VIEW_CALL_DIRECT").is_some_and(|v| !v.is_empty());
    let strategy_env = std::env::var("FRET_HOTPATCH_VIEW_CALL_STRATEGY").ok();
    if legacy_direct {
        println!("  view_call_strategy: direct (legacy; FRET_HOTPATCH_VIEW_CALL_DIRECT=1)");
    } else if let Some(raw) = strategy_env.as_deref() {
        println!("  view_call_strategy: {raw} (FRET_HOTPATCH_VIEW_CALL_STRATEGY)");
    } else {
        println!("  view_call_strategy: auto (default)");
        if cfg!(windows) {
            println!(
                "  windows note: auto defaults to `direct` when running with `--hotpatch` (ADR 0105)"
            );
        }
    }

    // Prefer the runtime's own trace if available (this reflects the effective strategy even when
    // the env vars are unset in the current shell).
    if let Some(path) = hotpatch_bootstrap_log_paths(&root)
        .into_iter()
        .find(|p| p.is_file())
        && let Ok(lines) = read_tail_lines(&path, 200, 256 * 1024)
    {
        let needle = "ui_app_render: view call strategy=";
        let last = lines.iter().rev().find_map(|line| {
            line.find(needle)
                .map(|idx| line[(idx + needle.len())..].trim().to_string())
        });
        if let Some(strategy) = last
            && (strategy == "direct" || strategy == "hotfn")
        {
            println!("  last_view_call: {strategy} (from bootstrap log)");
        }

        let reload_needle = "dev_reload:";
        if let Some(line) = lines.iter().rev().find(|line| line.contains(reload_needle)) {
            println!("  last_dev_reload: {line}");
        }
    }

    print_log_tail_group(
        "runner",
        &hotpatch_runner_log_paths(&root),
        tail,
        256 * 1024,
    )?;
    print_log_tail_group(
        "bootstrap",
        &hotpatch_bootstrap_log_paths(&root),
        tail,
        256 * 1024,
    )?;

    Ok(())
}

fn print_log_tail_group(
    name: &str,
    candidates: &[PathBuf],
    tail_lines: usize,
    max_bytes: usize,
) -> Result<(), String> {
    let existing: Vec<&PathBuf> = candidates.iter().filter(|p| p.is_file()).collect();

    println!();
    println!("{name} log candidates:");
    for p in candidates {
        let exists = if p.is_file() { "yes" } else { "no" };
        println!("  - {} (exists={exists})", p.display());
    }

    let Some(path) = existing.first() else {
        println!("  (no {name} log found)");
        return Ok(());
    };

    println!();
    println!("{name} log tail: {}", path.display());
    match read_tail_lines(path, tail_lines, max_bytes) {
        Ok(lines) => {
            for line in lines {
                println!("  {line}");
            }
        }
        Err(err) => {
            println!("  (failed to read: {err})");
        }
    }
    Ok(())
}

fn read_tail_lines(
    path: &Path,
    tail_lines: usize,
    max_bytes: usize,
) -> Result<Vec<String>, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let start = bytes.len().saturating_sub(max_bytes);
    let slice = &bytes[start..];
    let text = String::from_utf8_lossy(slice);

    let mut out: Vec<String> = Vec::new();
    for line in text.lines().rev() {
        if out.len() >= tail_lines {
            break;
        }
        out.push(line.to_string());
    }
    out.reverse();
    Ok(out)
}

fn hotpatch_poke(path: Option<&str>) -> Result<(), String> {
    let root = workspace_root()?;
    let path = match path {
        Some(path) => resolve_workspace_relative(&root, path),
        None => hotpatch_trigger_path(&root),
    };
    ensure_hotpatch_trigger_file_poked(&path)?;
    println!("{}", path.display());
    Ok(())
}

pub(crate) fn ensure_hotpatch_trigger_file_initialized(path: &Path) -> Result<(), String> {
    if path.is_file() {
        return Ok(());
    }
    ensure_hotpatch_trigger_file_poked(path)
}

pub(crate) fn ensure_hotpatch_trigger_file_poked(path: &Path) -> Result<(), String> {
    let dir = path
        .parent()
        .ok_or_else(|| "invalid hotpatch path".to_string())?;
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?;
    let marker = format!("{}", now.as_nanos());

    std::fs::write(path, marker).map_err(|e| e.to_string())?;
    Ok(())
}

fn hotpatch_watch(args: Vec<String>) -> Result<(), String> {
    let root = workspace_root()?;

    let mut watch_paths: Vec<String> = Vec::new();
    let mut trigger_path: Option<String> = None;
    let mut poll_ms: u64 = 500;
    let mut debounce_ms: u64 = 200;

    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--path" => {
                watch_paths.push(
                    it.next()
                        .ok_or_else(|| "--path requires a value".to_string())?,
                );
            }
            "--trigger-path" => {
                trigger_path = Some(
                    it.next()
                        .ok_or_else(|| "--trigger-path requires a value".to_string())?,
                );
            }
            "--poll-ms" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--poll-ms requires a value".to_string())?;
                poll_ms = raw.parse::<u64>().map_err(|e| e.to_string())?;
            }
            "--debounce-ms" => {
                let raw = it
                    .next()
                    .ok_or_else(|| "--debounce-ms requires a value".to_string())?;
                debounce_ms = raw.parse::<u64>().map_err(|e| e.to_string())?;
            }
            "--help" | "-h" => return Ok(()),
            other => return Err(format!("unknown argument for hotpatch watch: {other}")),
        }
    }

    let trigger_path = trigger_path.as_deref().unwrap_or(".fret/hotpatch.touch");
    let trigger_path = resolve_workspace_relative(&root, trigger_path);
    ensure_hotpatch_trigger_file_initialized(&trigger_path)?;

    let watch_roots: Vec<PathBuf> = if watch_paths.is_empty() {
        vec![
            root.join("crates"),
            root.join("ecosystem"),
            root.join("apps"),
        ]
    } else {
        watch_paths
            .iter()
            .map(|p| resolve_workspace_relative(&root, p))
            .collect()
    };

    eprintln!("Hotpatch watch: polling sources and poking trigger file on change");
    eprintln!("  trigger: {}", trigger_path.display());
    eprintln!("  poll_ms: {poll_ms}");
    eprintln!("  debounce_ms: {debounce_ms}");
    for p in &watch_roots {
        eprintln!("  watch: {}", p.display());
    }

    let mut last_sig = scan_watch_signature(&watch_roots)?;
    let mut last_poke_at: Option<std::time::Instant> = None;

    loop {
        std::thread::sleep(std::time::Duration::from_millis(poll_ms));
        let sig = scan_watch_signature(&watch_roots)?;
        if sig == last_sig {
            continue;
        }
        last_sig = sig;

        let now = std::time::Instant::now();
        if last_poke_at
            .is_some_and(|t| now.duration_since(t) < std::time::Duration::from_millis(debounce_ms))
        {
            continue;
        }

        ensure_hotpatch_trigger_file_poked(&trigger_path)?;
        last_poke_at = Some(now);
        eprintln!("poked: {}", trigger_path.display());
    }
}

fn scan_watch_signature(roots: &[PathBuf]) -> Result<u64, String> {
    let mut sig: u64 = 0;
    for root in roots {
        sig ^= scan_watch_root_signature(root)?;
    }
    Ok(sig)
}

fn scan_watch_root_signature(root: &Path) -> Result<u64, String> {
    let mut sig: u64 = 0;
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];

    while let Some(path) = stack.pop() {
        let md = match std::fs::symlink_metadata(&path) {
            Ok(md) => md,
            Err(_) => continue,
        };

        let ft = md.file_type();
        if ft.is_symlink() {
            continue;
        }

        if ft.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            let rd = match std::fs::read_dir(&path) {
                Ok(rd) => rd,
                Err(_) => continue,
            };
            for ent in rd {
                let ent = match ent {
                    Ok(ent) => ent,
                    Err(_) => continue,
                };
                stack.push(ent.path());
            }
            continue;
        }

        if !ft.is_file() {
            continue;
        }

        if !should_watch_file(&path) {
            continue;
        }

        let Some(mtime) = md.modified().ok() else {
            continue;
        };
        let ns = system_time_to_ns(mtime);
        let len = md.len();

        use std::hash::{Hash as _, Hasher as _};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        path.hash(&mut h);
        ns.hash(&mut h);
        len.hash(&mut h);
        sig ^= h.finish();
    }

    Ok(sig)
}

fn should_skip_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
        return false;
    };
    matches!(name, "target" | ".git" | ".fret" | "repo-ref")
}

fn should_watch_file(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|s| s.to_str()) else {
        return false;
    };
    if matches!(name, "Cargo.toml" | "Cargo.lock" | "rust-toolchain.toml") {
        return true;
    }

    let Some(ext) = path.extension().and_then(|s| s.to_str()) else {
        return false;
    };
    matches!(ext, "rs" | "toml" | "wgsl" | "md")
}

fn system_time_to_ns(t: std::time::SystemTime) -> u128 {
    t.duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn parse_hotpatch_path_arg(
    it: &mut impl Iterator<Item = String>,
) -> Result<Option<String>, String> {
    let mut path: Option<String> = None;
    while let Some(a) = it.next() {
        match a.as_str() {
            "--path" => {
                path = Some(
                    it.next()
                        .ok_or_else(|| "--path requires a value".to_string())?,
                );
            }
            "--help" | "-h" => return Ok(None),
            other => return Err(format!("unknown argument for hotpatch command: {other}")),
        }
    }
    Ok(path)
}

pub(crate) fn resolve_workspace_relative(workspace_root: &Path, raw: &str) -> PathBuf {
    let path = Path::new(raw);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum HotpatchBuildIdArg {
    Auto,
    None,
    Value(u64),
}

pub(crate) fn parse_hotpatch_build_id(raw: &str) -> Result<HotpatchBuildIdArg, String> {
    match raw {
        "auto" => Ok(HotpatchBuildIdArg::Auto),
        "none" => Ok(HotpatchBuildIdArg::None),
        other => Ok(HotpatchBuildIdArg::Value(
            other.parse::<u64>().map_err(|e| e.to_string())?,
        )),
    }
}
