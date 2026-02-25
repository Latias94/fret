use std::path::Path;

use crate::util::touch;

use super::args::resolve_latest_bundle_dir_path;

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
    let path = resolve_latest_bundle_dir_path(out_dir)?;
    println!("{}", path.display());
    Ok(())
}
