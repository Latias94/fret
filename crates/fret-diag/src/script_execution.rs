use std::path::Path;

use serde_json::Value;

use crate::script_tooling;

pub(crate) type ScriptFailureWriter = fn(&Path, &str, &str, &str, Option<String>);

#[derive(Debug, Clone)]
pub(crate) struct ScriptLoadPolicy {
    /// When true, schema v1 scripts are rejected (no auto-upgrade).
    pub tool_launched: bool,
    pub write_failure: ScriptFailureWriter,
    pub failure_note: Option<String>,
    pub include_stage_in_note: bool,
}

fn note_for_stage(policy: &ScriptLoadPolicy, stage: &'static str) -> Option<String> {
    if policy.include_stage_in_note {
        if let Some(base) = policy.failure_note.as_deref() {
            Some(format!("{base}:{stage}"))
        } else {
            Some(stage.to_string())
        }
    } else {
        policy.failure_note.clone()
    }
}

pub(crate) fn load_script_json_for_execution(
    src: &Path,
    policy: ScriptLoadPolicy,
    script_result_path: &Path,
) -> Result<(Value, bool), String> {
    let bytes = std::fs::read(src).map_err(|e| {
        let err = e.to_string();
        (policy.write_failure)(
            script_result_path,
            "tooling.script.read_failed",
            &err,
            "tooling_error",
            note_for_stage(&policy, "read script json"),
        );
        err
    })?;

    let script_value: Value = serde_json::from_slice(&bytes).map_err(|e| {
        let err = e.to_string();
        (policy.write_failure)(
            script_result_path,
            "tooling.script.parse_failed",
            &err,
            "tooling_error",
            note_for_stage(&policy, "parse script json"),
        );
        err
    })?;

    let resolved = script_tooling::resolve_script_json_redirects_from_value(src, script_value)
        .map_err(|e| {
            let err = e.to_string();
            (policy.write_failure)(
                script_result_path,
                "tooling.script.redirect_failed",
                &err,
                "tooling_error",
                note_for_stage(&policy, "resolve_script_json_redirects"),
            );
            err
        })?;

    let schema_version = crate::compat::script::script_schema_version_from_value(&resolved.value);
    if schema_version == 1 && policy.tool_launched {
        let msg = format!(
            "script schema_version=1 is disabled for tool-launched runs (--launch/--reuse-launch); upgrade to schema_version=2 (tip: fretboard diag script upgrade --write {})",
            src.display()
        );
        (policy.write_failure)(
            script_result_path,
            "script.schema_v1_disabled",
            &msg,
            "tooling_error",
            note_for_stage(&policy, "script_schema_v1_disabled"),
        );
        return Err(msg);
    }

    let (mut script_json, upgraded) =
        crate::compat::script::upgrade_script_json_value_to_v2_if_needed(resolved.value)
            .inspect_err(|e| {
                (policy.write_failure)(
                    script_result_path,
                    "tooling.script.upgrade_failed",
                    e,
                    "tooling_error",
                    note_for_stage(&policy, "upgrade_script_json_value_to_v2_if_needed"),
                );
            })?;

    script_tooling::canonicalize_json_value(&mut script_json);
    Ok((script_json, upgraded))
}
