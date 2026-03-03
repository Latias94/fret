use std::path::Path;

use fret_diag_protocol::{
    UiScriptEventLogEntryV1, UiScriptEvidenceV1, UiScriptResultV1, UiScriptStageV1,
};

use crate::artifact_store::RunArtifactStore;
use crate::util::{now_unix_ms, read_json_value, write_json_value};

fn script_result_has_stable_reason_code(script_result_path: &Path) -> bool {
    read_json_value(script_result_path)
        .and_then(|v| {
            v.get("reason_code")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .is_some()
}

pub(crate) fn write_tooling_failure_script_result(
    script_result_path: &Path,
    reason_code: &str,
    reason: &str,
    kind: &str,
    note: Option<String>,
) {
    let now = now_unix_ms();
    let evidence = UiScriptEvidenceV1 {
        event_log: vec![UiScriptEventLogEntryV1 {
            unix_ms: now,
            kind: kind.to_string(),
            step_index: None,
            note,
            bundle_dir: None,
            window: None,
            tick_id: None,
            frame_id: None,
            window_snapshot_seq: None,
        }],
        ..UiScriptEvidenceV1::default()
    };
    let result = UiScriptResultV1 {
        schema_version: 1,
        run_id: 0,
        updated_unix_ms: now,
        window: None,
        stage: UiScriptStageV1::Failed,
        step_index: None,
        reason_code: Some(reason_code.to_string()),
        reason: Some(reason.to_string()),
        evidence: Some(evidence),
        last_bundle_dir: None,
        last_bundle_artifact: None,
    };

    let _ = write_json_value(
        script_result_path,
        &serde_json::to_value(&result).unwrap_or_else(|_| serde_json::json!({})),
    );
}

pub(crate) fn write_tooling_failure_script_result_if_missing(
    script_result_path: &Path,
    reason_code: &str,
    reason: &str,
    kind: &str,
    note: Option<String>,
) {
    if script_result_has_stable_reason_code(script_result_path) {
        return;
    }

    write_tooling_failure_script_result(script_result_path, reason_code, reason, kind, note);
}

pub(crate) fn push_tooling_event_log_entry(
    result: &mut UiScriptResultV1,
    kind: &str,
    note: Option<String>,
) {
    let now = now_unix_ms();
    let evidence = result
        .evidence
        .get_or_insert_with(UiScriptEvidenceV1::default);
    evidence.event_log.push(UiScriptEventLogEntryV1 {
        unix_ms: now,
        kind: kind.to_string(),
        step_index: result.step_index,
        note,
        bundle_dir: result.last_bundle_dir.clone(),
        window: result.window,
        tick_id: None,
        frame_id: None,
        window_snapshot_seq: None,
    });
}

pub(crate) fn mark_existing_script_result_tooling_failure(
    out_dir: &Path,
    script_result_path: &Path,
    reason_code: &str,
    reason: &str,
    kind: &str,
    note: Option<String>,
) {
    if let Ok(bytes) = std::fs::read(script_result_path)
        && let Ok(mut parsed) = serde_json::from_slice::<UiScriptResultV1>(&bytes)
    {
        push_tooling_event_log_entry(&mut parsed, kind, note.clone());
        if matches!(parsed.stage, UiScriptStageV1::Passed) {
            parsed.stage = UiScriptStageV1::Failed;
            parsed.reason_code = Some(reason_code.to_string());
            parsed.reason = Some(reason.to_string());
        }
        let _ = write_json_value(
            script_result_path,
            &serde_json::to_value(&parsed).unwrap_or_else(|_| serde_json::json!({})),
        );
        RunArtifactStore::new(out_dir, parsed.run_id).write_script_result(&parsed);
        return;
    }

    write_tooling_failure_script_result(script_result_path, reason_code, reason, kind, note);
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn make_temp_dir(prefix: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!(
            "{prefix}-{}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");
        root
    }

    #[test]
    fn write_tooling_failure_script_result_if_missing_preserves_existing_reason_code() {
        let root = make_temp_dir("fret-diag-tooling-failure-if-missing");
        let path = root.join("script.result.json");

        write_tooling_failure_script_result(
            &path,
            "tooling.old",
            "old failure",
            "tooling_error",
            Some("old".to_string()),
        );
        write_tooling_failure_script_result_if_missing(
            &path,
            "tooling.new",
            "new failure",
            "tooling_error",
            Some("new".to_string()),
        );

        let bytes = std::fs::read(&path).expect("read script.result.json");
        let parsed: UiScriptResultV1 = serde_json::from_slice(&bytes).expect("parse script.result");
        assert_eq!(parsed.reason_code.as_deref(), Some("tooling.old"));
        assert_eq!(parsed.reason.as_deref(), Some("old failure"));
    }

    #[test]
    fn mark_existing_script_result_tooling_failure_flips_pass_to_fail_and_writes_run_id_copy() {
        let root = make_temp_dir("fret-diag-tooling-failure-mark-existing");
        let script_result_path = root.join("script.result.json");

        let initial = UiScriptResultV1 {
            schema_version: 1,
            run_id: 42,
            updated_unix_ms: now_unix_ms(),
            window: None,
            stage: UiScriptStageV1::Passed,
            step_index: None,
            reason_code: None,
            reason: None,
            evidence: Some(UiScriptEvidenceV1::default()),
            last_bundle_dir: None,
            last_bundle_artifact: None,
        };
        write_json_value(
            &script_result_path,
            &serde_json::to_value(&initial).expect("initial json"),
        )
        .expect("write initial script.result.json");

        mark_existing_script_result_tooling_failure(
            &root,
            &script_result_path,
            "tooling.bundle_dump.failed",
            "bundle dump failed",
            "tooling_error",
            Some("note".to_string()),
        );

        let bytes = std::fs::read(&script_result_path).expect("read script.result.json");
        let parsed: UiScriptResultV1 = serde_json::from_slice(&bytes).expect("parse script.result");
        assert!(matches!(parsed.stage, UiScriptStageV1::Failed));
        assert_eq!(
            parsed.reason_code.as_deref(),
            Some("tooling.bundle_dump.failed")
        );
        assert_eq!(parsed.reason.as_deref(), Some("bundle dump failed"));
        assert_eq!(parsed.run_id, 42);
        assert!(
            parsed
                .evidence
                .as_ref()
                .and_then(|e| e.event_log.last())
                .map(|e| e.kind.as_str())
                == Some("tooling_error")
        );

        let run_id_copy = root.join("42").join("script.result.json");
        let bytes = std::fs::read(&run_id_copy).expect("read run_id script.result.json");
        let copied: UiScriptResultV1 =
            serde_json::from_slice(&bytes).expect("parse run_id script.result");
        assert!(matches!(copied.stage, UiScriptStageV1::Failed));
        assert_eq!(
            copied.reason_code.as_deref(),
            Some("tooling.bundle_dump.failed")
        );
    }
}
