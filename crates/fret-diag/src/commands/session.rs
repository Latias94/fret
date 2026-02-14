use std::path::Path;

use crate::compare::{find_latest_export_dir, read_latest_pointer};
use crate::util::touch;

pub(crate) fn cmd_poke(
    rest: &[String],
    pack_after_run: bool,
    trigger_path: &Path,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    if !rest.is_empty() {
        return Err(format!("unexpected arguments: {}", rest.join(" ")));
    }
    touch(trigger_path)?;
    println!("{}", trigger_path.display());
    Ok(())
}

pub(crate) fn cmd_path(
    rest: &[String],
    pack_after_run: bool,
    trigger_path: &Path,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    if !rest.is_empty() {
        return Err(format!("unexpected arguments: {}", rest.join(" ")));
    }
    println!("{}", trigger_path.display());
    Ok(())
}

pub(crate) fn cmd_latest(
    rest: &[String],
    pack_after_run: bool,
    out_dir: &Path,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    if !rest.is_empty() {
        return Err(format!("unexpected arguments: {}", rest.join(" ")));
    }
    if let Some(path) = read_latest_pointer(out_dir).or_else(|| find_latest_export_dir(out_dir)) {
        println!("{}", path.display());
        return Ok(());
    }
    Err(format!("no diagnostics bundle found under {}", out_dir.display()))
}
