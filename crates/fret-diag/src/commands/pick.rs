use std::path::{Path, PathBuf};

use crate::stats::{
    apply_pick_to_script, report_pick_result_and_exit, run_pick_and_wait, write_pick_script,
};
use crate::util::touch;

pub(crate) fn cmd_pick_arm(rest: &[String], pick_trigger_path: &Path) -> Result<(), String> {
    if !rest.is_empty() {
        return Err(format!("unexpected arguments: {}", rest.join(" ")));
    }
    touch(pick_trigger_path)?;
    println!("{}", pick_trigger_path.display());
    Ok(())
}

pub(crate) fn cmd_pick(
    rest: &[String],
    pick_trigger_path: &Path,
    pick_result_path: &Path,
    pick_result_trigger_path: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<(), String> {
    if !rest.is_empty() {
        return Err(format!("unexpected arguments: {}", rest.join(" ")));
    }
    let result = run_pick_and_wait(
        pick_trigger_path,
        pick_result_path,
        pick_result_trigger_path,
        timeout_ms,
        poll_ms,
    )?;
    report_pick_result_and_exit(&result)
}

pub(crate) fn cmd_pick_script(
    rest: &[String],
    pick_trigger_path: &Path,
    pick_result_path: &Path,
    pick_result_trigger_path: &Path,
    pick_script_out: &Path,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<(), String> {
    if !rest.is_empty() {
        return Err(format!("unexpected arguments: {}", rest.join(" ")));
    }
    let result = run_pick_and_wait(
        pick_trigger_path,
        pick_result_path,
        pick_result_trigger_path,
        timeout_ms,
        poll_ms,
    )?;

    let Some(selector) = result.selector.clone() else {
        return Err("pick succeeded but no selector was returned".to_string());
    };

    write_pick_script(&selector, pick_script_out)?;
    println!("{}", pick_script_out.display());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_pick_apply(
    rest: &[String],
    workspace_root: &Path,
    pick_trigger_path: &Path,
    pick_result_path: &Path,
    pick_result_trigger_path: &Path,
    pick_apply_pointer: Option<&str>,
    pick_apply_out: Option<PathBuf>,
    timeout_ms: u64,
    poll_ms: u64,
) -> Result<(), String> {
    let Some(script) = rest.first().cloned() else {
        return Err(
            "missing script path (try: fretboard diag pick-apply ./script.json --ptr /steps/0/target)".to_string(),
        );
    };
    if rest.len() != 1 {
        return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
    }
    let Some(ptr) = pick_apply_pointer else {
        return Err("missing --ptr (example: --ptr /steps/0/target)".to_string());
    };

    let result = run_pick_and_wait(
        pick_trigger_path,
        pick_result_path,
        pick_result_trigger_path,
        timeout_ms,
        poll_ms,
    )?;

    let Some(selector) = result.selector.clone() else {
        return Err("pick succeeded but no selector was returned".to_string());
    };

    let script_path = crate::resolve_path(workspace_root, PathBuf::from(script));
    let out_path = pick_apply_out
        .map(|p| crate::resolve_path(workspace_root, p))
        .unwrap_or_else(|| script_path.clone());

    apply_pick_to_script(&script_path, &out_path, ptr, selector)?;
    println!("{}", out_path.display());
    Ok(())
}
