use std::path::{Path, PathBuf};

use crate::compare::{maybe_launch_demo, stop_launched_demo};
use crate::compat::script::upgrade_script_json_value_to_v2_if_needed;
use crate::script_tooling::{
    NormalizedScript, ScriptLintReport, ScriptSchemaReport, lint_scripts,
    normalize_script_from_path, validate_scripts,
};
use crate::shrink;
use crate::stats::{ScriptResultSummary, run_script_and_wait};
use crate::util::{touch, write_json_value};

#[allow(clippy::too_many_arguments)]
pub(crate) fn cmd_script(
    rest: &[String],
    pack_after_run: bool,
    workspace_root: &Path,
    out_dir: &Path,
    script_path: &Path,
    script_trigger_path: &Path,
    script_result_path: &Path,
    script_result_trigger_path: &Path,
    ready_path: &Path,
    exit_path: &Path,
    script_tool_check: bool,
    script_tool_write: bool,
    script_tool_check_out: Option<PathBuf>,
    shrink_out: Option<PathBuf>,
    shrink_any_fail: bool,
    shrink_match_reason_code: Option<String>,
    shrink_match_reason: Option<String>,
    shrink_min_steps: u64,
    shrink_max_iters: u64,
    launch: &Option<Vec<String>>,
    launch_env: &[(String, String)],
    timeout_ms: u64,
    poll_ms: u64,
    stats_json: bool,
) -> Result<(), String> {
    if pack_after_run {
        return Err("--pack is only supported with `diag run`".to_string());
    }
    let Some(op) = rest.first().map(|s| s.as_str()) else {
        return Err("missing script subcommand or script path (try: fretboard diag script ./script.json | fretboard diag script normalize ./script.json)".to_string());
    };

    let shrink_flags_used = shrink_out.is_some()
        || shrink_any_fail
        || shrink_match_reason_code.is_some()
        || shrink_match_reason.is_some()
        || shrink_min_steps != 1
        || shrink_max_iters != 200;
    if shrink_flags_used && op != "shrink" {
        return Err("--shrink-* flags are only supported with `diag script shrink`".to_string());
    }

    match op {
        "normalize" => {
            if script_tool_check && script_tool_write {
                return Err("--check cannot be combined with --write".to_string());
            }
            if script_tool_check_out.is_some() {
                return Err("--check-out is not supported with `diag script normalize`".to_string());
            }

            let inputs: Vec<String> = rest[1..].to_vec();
            if inputs.is_empty() {
                return Err(
                    "missing script path (try: fretboard diag script normalize ./script.json)"
                        .to_string(),
                );
            }
            if inputs.len() != 1 && !script_tool_check && !script_tool_write {
                return Err(
                    "normalize expects exactly one script unless --check or --write is used"
                        .to_string(),
                );
            }

            let scripts = crate::expand_script_inputs(workspace_root, &inputs)?;
            if scripts.len() != 1 && !script_tool_check && !script_tool_write {
                return Err(
                    "normalize expects exactly one script unless --check or --write is used"
                        .to_string(),
                );
            }

            let mut any_changed = false;
            for src in scripts {
                let NormalizedScript {
                    normalized,
                    changed,
                    write_path,
                    redirect_chain,
                    ..
                } = normalize_script_from_path(&src)?;

                if script_tool_check {
                    if changed {
                        any_changed = true;
                        if redirect_chain.is_empty() && write_path == src {
                            eprintln!("not normalized: {}", src.display());
                        } else {
                            eprintln!(
                                "not normalized: {} (resolved: {})",
                                src.display(),
                                write_path.display()
                            );
                        }
                    } else {
                        if redirect_chain.is_empty() && write_path == src {
                            println!("{}", src.display());
                        } else {
                            println!("{} (resolved: {})", src.display(), write_path.display());
                        }
                    }
                    continue;
                }

                if script_tool_write {
                    if changed {
                        any_changed = true;
                        std::fs::write(&write_path, normalized.as_bytes())
                            .map_err(|e| e.to_string())?;
                    }
                    if redirect_chain.is_empty() && write_path == src {
                        println!("{}", src.display());
                    } else {
                        println!("{} (resolved: {})", src.display(), write_path.display());
                    }
                    continue;
                }

                print!("{normalized}");
            }

            if script_tool_check && any_changed {
                std::process::exit(1);
            }
            Ok(())
        }
        "upgrade" => {
            if script_tool_check && script_tool_write {
                return Err("--check cannot be combined with --write".to_string());
            }
            if script_tool_check_out.is_some() {
                return Err("--check-out is not supported with `diag script upgrade`".to_string());
            }

            let inputs: Vec<String> = rest[1..].to_vec();
            if inputs.is_empty() {
                return Err(
                    "missing script path (try: fretboard diag script upgrade ./script.json)"
                        .to_string(),
                );
            }
            if inputs.len() != 1 && !script_tool_check && !script_tool_write {
                return Err(
                    "upgrade expects exactly one script unless --check or --write is used"
                        .to_string(),
                );
            }

            let scripts = crate::expand_script_inputs(workspace_root, &inputs)?;
            if scripts.len() != 1 && !script_tool_check && !script_tool_write {
                return Err(
                    "upgrade expects exactly one script unless --check or --write is used"
                        .to_string(),
                );
            }

            let mut any_needs_upgrade = false;
            for src in scripts {
                let resolved = crate::script_tooling::read_script_json_resolving_redirects(&src)?;
                let schema_version =
                    crate::compat::script::script_schema_version_from_value(&resolved.value);

                let needs_upgrade = schema_version == 1;
                any_needs_upgrade |= needs_upgrade;

                if script_tool_check {
                    if needs_upgrade {
                        if resolved.redirect_chain.is_empty() && resolved.write_path == src {
                            eprintln!("needs upgrade (schema v1): {}", src.display());
                        } else {
                            eprintln!(
                                "needs upgrade (schema v1): {} (resolved: {})",
                                src.display(),
                                resolved.write_path.display()
                            );
                        }
                    } else {
                        if resolved.redirect_chain.is_empty() && resolved.write_path == src {
                            println!("{}", src.display());
                        } else {
                            println!(
                                "{} (resolved: {})",
                                src.display(),
                                resolved.write_path.display()
                            );
                        }
                    }
                    continue;
                }

                let mut value = if needs_upgrade {
                    let (value, _upgraded) =
                        upgrade_script_json_value_to_v2_if_needed(resolved.value)?;
                    value
                } else {
                    resolved.value
                };
                crate::script_tooling::canonicalize_json_value(&mut value);
                let mut pretty = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
                pretty.push('\n');

                if script_tool_write {
                    if needs_upgrade {
                        std::fs::write(&resolved.write_path, pretty.as_bytes())
                            .map_err(|e| e.to_string())?;
                    }
                    if resolved.redirect_chain.is_empty() && resolved.write_path == src {
                        println!("{}", src.display());
                    } else {
                        println!(
                            "{} (resolved: {})",
                            src.display(),
                            resolved.write_path.display()
                        );
                    }
                    continue;
                }

                print!("{pretty}");
            }

            if script_tool_check && any_needs_upgrade {
                std::process::exit(1);
            }
            Ok(())
        }
        "validate" => {
            if script_tool_check || script_tool_write {
                return Err(
                    "--check/--write are not supported with `diag script validate`".to_string(),
                );
            }

            let inputs: Vec<String> = rest[1..].to_vec();
            if inputs.is_empty() {
                return Err(
                    "missing script path (try: fretboard diag script validate ./script.json)"
                        .to_string(),
                );
            }
            let scripts = crate::expand_script_inputs(workspace_root, &inputs)?;

            let ScriptSchemaReport {
                payload,
                error_scripts,
            } = validate_scripts(&scripts);

            let out = script_tool_check_out
                .map(|p| crate::resolve_path(workspace_root, p))
                .unwrap_or_else(|| out_dir.join("check.script_schema.json"));
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let pretty =
                serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
            std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;

            if stats_json {
                println!("{pretty}");
            } else {
                println!("{}", out.display());
            }

            if error_scripts > 0 {
                std::process::exit(1);
            }
            Ok(())
        }
        "lint" => {
            if script_tool_check || script_tool_write {
                return Err("--check/--write are not supported with `diag script lint`".to_string());
            }

            let inputs: Vec<String> = rest[1..].to_vec();
            if inputs.is_empty() {
                return Err(
                    "missing script path (try: fretboard diag script lint ./script.json)"
                        .to_string(),
                );
            }
            let scripts = crate::expand_script_inputs(workspace_root, &inputs)?;

            let ScriptLintReport {
                payload,
                error_scripts,
            } = lint_scripts(&scripts);

            let out = script_tool_check_out
                .map(|p| crate::resolve_path(workspace_root, p))
                .unwrap_or_else(|| out_dir.join("check.script_lint.json"));
            if let Some(parent) = out.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let pretty =
                serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
            std::fs::write(&out, pretty.as_bytes()).map_err(|e| e.to_string())?;

            if stats_json {
                println!("{pretty}");
            } else {
                println!("{}", out.display());
            }

            if error_scripts > 0 {
                std::process::exit(1);
            }
            Ok(())
        }
        "shrink" => {
            if script_tool_check || script_tool_write || script_tool_check_out.is_some() {
                return Err(
                    "--check/--write/--check-out are not supported with `diag script shrink`"
                        .to_string(),
                );
            }
            let inputs: Vec<String> = rest[1..].to_vec();
            if inputs.is_empty() {
                return Err(
                    "missing script path (try: fretboard diag script shrink ./script.json)"
                        .to_string(),
                );
            }

            #[derive(Debug, Clone)]
            enum ActionScript {
                V1(fret_diag_protocol::UiActionScriptV1),
                V2(fret_diag_protocol::UiActionScriptV2),
            }

            impl ActionScript {
                fn steps_len(&self) -> usize {
                    match self {
                        Self::V1(s) => s.steps.len(),
                        Self::V2(s) => s.steps.len(),
                    }
                }

                fn keep_steps(&self, keep: &[usize]) -> Self {
                    match self {
                        Self::V1(s) => {
                            let steps = keep
                                .iter()
                                .filter_map(|&i| s.steps.get(i).cloned())
                                .collect();
                            Self::V1(fret_diag_protocol::UiActionScriptV1 {
                                schema_version: 1,
                                meta: s.meta.clone(),
                                steps,
                            })
                        }
                        Self::V2(s) => {
                            let steps = keep
                                .iter()
                                .filter_map(|&i| s.steps.get(i).cloned())
                                .collect();
                            Self::V2(fret_diag_protocol::UiActionScriptV2 {
                                schema_version: 2,
                                meta: s.meta.clone(),
                                steps,
                            })
                        }
                    }
                }

                fn to_pretty_json(&self) -> Result<String, String> {
                    let mut value = match self {
                        Self::V1(s) => serde_json::to_value(s).map_err(|e| e.to_string())?,
                        Self::V2(s) => serde_json::to_value(s).map_err(|e| e.to_string())?,
                    };
                    crate::script_tooling::canonicalize_json_value(&mut value);
                    let mut s = serde_json::to_string_pretty(&value).map_err(|e| e.to_string())?;
                    s.push('\n');
                    Ok(s)
                }
            }

            fn read_action_script(path: &Path) -> Result<ActionScript, String> {
                let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
                let value: serde_json::Value =
                    serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                let schema_version =
                    crate::compat::script::script_schema_version_from_value(&value);
                match schema_version {
                    1 => Ok(ActionScript::V1(
                        serde_json::from_value(value).map_err(|e| e.to_string())?,
                    )),
                    2 => Ok(ActionScript::V2(
                        serde_json::from_value(value).map_err(|e| e.to_string())?,
                    )),
                    _ => Err(format!(
                        "unknown script schema_version (expected 1 or 2): {}",
                        schema_version
                    )),
                }
            }

            fn matches_failure(
                s: &ScriptResultSummary,
                any_fail: bool,
                reason_code: Option<&str>,
                reason: Option<&str>,
            ) -> bool {
                if s.stage.as_deref() != Some("failed") {
                    return false;
                }
                if any_fail {
                    return true;
                }
                if let Some(code) = reason_code {
                    return s.reason_code.as_deref() == Some(code);
                }
                if let Some(r) = reason {
                    return s.reason.as_deref() == Some(r);
                }
                true
            }

            let scripts = crate::expand_script_inputs(workspace_root, &inputs)?;
            if scripts.len() != 1 {
                return Err("shrink expects exactly one script input".to_string());
            }
            let src = scripts.into_iter().next().unwrap();

            let shrink_dir = out_dir.join("shrink");
            std::fs::create_dir_all(&shrink_dir).map_err(|e| e.to_string())?;

            let out_path = shrink_out
                .clone()
                .map(|p| crate::resolve_path(workspace_root, p))
                .unwrap_or_else(|| shrink_dir.join("script.min.json"));
            let summary_path = shrink_dir.join("shrink.summary.json");
            let candidate_path = shrink_dir.join("script.candidate.json");

            let script = read_action_script(&src)?;
            let total_steps = script.steps_len();
            if total_steps == 0 && shrink_min_steps > 0 {
                return Err("script has no steps; nothing to shrink".to_string());
            }

            let wants_screenshots = crate::script_requests_screenshots(&src);
            let shrink_launch_env = launch_env.to_vec();
            let mut launch_fs_transport_cfg =
                crate::transport::FsDiagTransportConfig::from_out_dir(out_dir.to_path_buf());
            launch_fs_transport_cfg.script_path = script_path.to_path_buf();
            launch_fs_transport_cfg.script_trigger_path = script_trigger_path.to_path_buf();
            launch_fs_transport_cfg.script_result_path = script_result_path.to_path_buf();
            launch_fs_transport_cfg.script_result_trigger_path =
                script_result_trigger_path.to_path_buf();
            let mut child = maybe_launch_demo(
                launch,
                &shrink_launch_env,
                workspace_root,
                ready_path,
                exit_path,
                &launch_fs_transport_cfg,
                wants_screenshots,
                timeout_ms,
                poll_ms,
                false,
            )
            .inspect_err(|err| {
                crate::write_tooling_failure_script_result_if_missing(
                    script_result_path,
                    "tooling.launch.failed",
                    err,
                    "tooling_error",
                    Some("maybe_launch_demo".to_string()),
                );
            })?;

            let baseline = run_script_and_wait(
                &src,
                script_path,
                script_trigger_path,
                script_result_path,
                script_result_trigger_path,
                timeout_ms,
                poll_ms,
            )?;

            if baseline.stage.as_deref() != Some("failed") {
                stop_launched_demo(&mut child, exit_path, poll_ms);
                return Err(format!(
                    "baseline script did not fail (stage={:?}); shrink expects a failing script",
                    baseline.stage
                ));
            }

            let desired_reason_code = shrink_match_reason_code
                .as_deref()
                .or(baseline.reason_code.as_deref());
            let desired_reason = shrink_match_reason
                .as_deref()
                .or(baseline.reason.as_deref());

            let mut attempts_total: u64 = 0;
            let mut attempts_errors: u64 = 0;
            let mut last_error: Option<String> = None;

            let min_steps = usize::try_from(shrink_min_steps)
                .unwrap_or(usize::MAX)
                .min(total_steps);
            let (keep, reductions, iters) =
                shrink::ddmin_keep_indices(total_steps, min_steps, shrink_max_iters, |keep| {
                    attempts_total += 1;
                    let candidate = script.keep_steps(keep);
                    let pretty = match candidate.to_pretty_json() {
                        Ok(s) => s,
                        Err(err) => {
                            attempts_errors += 1;
                            last_error = Some(err);
                            return false;
                        }
                    };
                    if let Err(err) = std::fs::write(&candidate_path, pretty.as_bytes()) {
                        attempts_errors += 1;
                        last_error = Some(err.to_string());
                        return false;
                    }

                    match run_script_and_wait(
                        &candidate_path,
                        script_path,
                        script_trigger_path,
                        script_result_path,
                        script_result_trigger_path,
                        timeout_ms,
                        poll_ms,
                    ) {
                        Ok(s) => matches_failure(
                            &s,
                            shrink_any_fail,
                            desired_reason_code,
                            desired_reason,
                        ),
                        Err(err) => {
                            attempts_errors += 1;
                            last_error = Some(err);
                            false
                        }
                    }
                });

            let candidate = script.keep_steps(&keep);
            let pretty = candidate.to_pretty_json()?;
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(&out_path, pretty.as_bytes()).map_err(|e| e.to_string())?;

            let final_result = run_script_and_wait(
                &out_path,
                script_path,
                script_trigger_path,
                script_result_path,
                script_result_trigger_path,
                timeout_ms,
                poll_ms,
            )?;

            stop_launched_demo(&mut child, exit_path, poll_ms);

            let ok = matches_failure(
                &final_result,
                shrink_any_fail,
                desired_reason_code,
                desired_reason,
            );
            if !ok {
                return Err(format!(
                    "minimized script does not reproduce baseline failure (stage={:?} reason_code={:?} reason={:?})",
                    final_result.stage, final_result.reason_code, final_result.reason
                ));
            }

            let keep_set: std::collections::BTreeSet<usize> = keep.iter().copied().collect();
            let removed: Vec<usize> = (0..total_steps).filter(|i| !keep_set.contains(i)).collect();
            let reductions_json: Vec<serde_json::Value> = reductions
                .into_iter()
                .map(|r| {
                    serde_json::json!({
                        "granularity": r.granularity,
                        "kept_len": r.kept_len,
                        "removed": r.removed,
                    })
                })
                .collect();

            let payload = serde_json::json!({
                "schema_version": 1,
                "status": "passed",
                "script": src.display().to_string(),
                "out": out_path.display().to_string(),
                "params": {
                    "min_steps": shrink_min_steps,
                    "max_iters": shrink_max_iters,
                    "any_fail": shrink_any_fail,
                    "match_reason_code": desired_reason_code,
                    "match_reason": desired_reason,
                },
                "baseline": {
                    "run_id": baseline.run_id,
                    "stage": baseline.stage,
                    "step_index": baseline.step_index,
                    "reason_code": baseline.reason_code,
                    "reason": baseline.reason,
                    "last_bundle_dir": baseline.last_bundle_dir,
                },
                "final": {
                    "run_id": final_result.run_id,
                    "stage": final_result.stage,
                    "step_index": final_result.step_index,
                    "reason_code": final_result.reason_code,
                    "reason": final_result.reason,
                    "last_bundle_dir": final_result.last_bundle_dir,
                },
                "steps": {
                    "original": total_steps,
                    "kept": keep.len(),
                    "removed": removed.len(),
                    "removed_indices": removed,
                },
                "search": {
                    "iters": iters,
                    "attempts_total": attempts_total,
                    "attempts_errors": attempts_errors,
                    "last_error": last_error,
                    "reductions": reductions_json,
                },
            });

            if let Some(parent) = summary_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            let pretty =
                serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
            std::fs::write(&summary_path, pretty.as_bytes()).map_err(|e| e.to_string())?;

            if stats_json {
                println!("{pretty}");
            } else {
                println!("{}", out_path.display());
            }
            Ok(())
        }
        _ => {
            let Some(src) = rest.first().cloned() else {
                return Err(
                    "missing script path (try: fretboard diag script ./script.json)".to_string(),
                );
            };
            if rest.len() != 1 {
                return Err(format!("unexpected arguments: {}", rest[1..].join(" ")));
            }

            let src = crate::resolve_path(workspace_root, PathBuf::from(src));
            let script_value: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&src).map_err(|e| e.to_string())?)
                    .map_err(|e| e.to_string())?;
            let resolved_script =
                crate::script_tooling::resolve_script_json_redirects_from_value(&src, script_value)
                    .map_err(|e| e.to_string())?;
            let (mut script_json, upgraded) =
                upgrade_script_json_value_to_v2_if_needed(resolved_script.value)?;
            crate::script_tooling::canonicalize_json_value(&mut script_json);
            if upgraded {
                eprintln!(
                    "warning: script schema_version=1 detected; tooling upgraded to schema_version=2 for execution (source={})",
                    src.display()
                );
            }
            write_json_value(script_path, &script_json)?;
            touch(script_trigger_path)?;
            println!("{}", script_trigger_path.display());
            Ok(())
        }
    }
}
