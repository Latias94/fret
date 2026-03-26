use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::registry::campaigns::{campaigns_dir_from_workspace_root, load_manifest_campaign};
use crate::util::read_json_value;

#[derive(Debug, Default)]
struct CampaignsDoctorCounts {
    manifest_json_total: u64,
    manifest_valid_total: u64,
    manifest_invalid_total: u64,
    duplicate_id_total: u64,
    legacy_shape_total: u64,
    campaigns_total: u64,
    suite_items_total: u64,
    script_items_total: u64,
}

#[derive(Debug, Default)]
struct CampaignsDoctorExamples {
    invalid_manifest_paths: Vec<String>,
    duplicate_ids: Vec<String>,
    legacy_shape_paths: Vec<String>,
}

fn push_bounded(entries: &mut Vec<String>, max_examples: usize, value: String) {
    if entries.len() < max_examples {
        entries.push(value);
    }
}

fn manifest_uses_legacy_shape(value: &Value) -> bool {
    let has_nonempty_items = value
        .get("items")
        .and_then(|v| v.as_array())
        .is_some_and(|items| !items.is_empty());
    if has_nonempty_items {
        return false;
    }

    let has_legacy_suites = value
        .get("suites")
        .and_then(|v| v.as_array())
        .is_some_and(|items| !items.is_empty());
    let has_legacy_scripts = value
        .get("scripts")
        .and_then(|v| v.as_array())
        .is_some_and(|items| !items.is_empty());

    has_legacy_suites || has_legacy_scripts
}

fn collect_campaign_manifest_paths(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut paths = std::fs::read_dir(dir)
        .map_err(|e| {
            format!(
                "failed to read campaign manifests dir {}: {}",
                dir.display(),
                e
            )
        })?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

fn build_doctor_campaigns_payload(
    workspace_root: &Path,
    strict: bool,
    max_examples: usize,
) -> Result<Value, String> {
    let campaigns_root = campaigns_dir_from_workspace_root(workspace_root);
    let mut errors = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();
    let mut repairs = Vec::<Value>::new();
    let mut counts = CampaignsDoctorCounts::default();
    let mut examples = CampaignsDoctorExamples::default();

    if !campaigns_root.is_dir() {
        errors.push(format!(
            "diag campaign manifests directory not found: {}",
            campaigns_root.display()
        ));
        repairs.push(json!({
            "code": "diag-campaigns.dir-missing",
            "note": "Create repo-owned campaign manifests under tools/diag-campaigns/ or run from the workspace root.",
        }));
    } else {
        let manifest_paths = collect_campaign_manifest_paths(&campaigns_root)?;
        if manifest_paths.is_empty() {
            warnings.push(format!(
                "no campaign manifests found under {}",
                campaigns_root.display()
            ));
        }

        let mut seen_paths_by_id = BTreeMap::<String, PathBuf>::new();

        for manifest_path in manifest_paths {
            counts.manifest_json_total = counts.manifest_json_total.saturating_add(1);

            if let Some(raw) = read_json_value(&manifest_path) {
                if manifest_uses_legacy_shape(&raw) {
                    counts.legacy_shape_total = counts.legacy_shape_total.saturating_add(1);
                    push_bounded(
                        &mut examples.legacy_shape_paths,
                        max_examples,
                        manifest_path.display().to_string(),
                    );
                }
            }

            match load_manifest_campaign(&manifest_path) {
                Ok(campaign) => {
                    counts.manifest_valid_total = counts.manifest_valid_total.saturating_add(1);
                    counts.campaigns_total = counts.campaigns_total.saturating_add(1);
                    counts.suite_items_total = counts
                        .suite_items_total
                        .saturating_add(campaign.suite_count() as u64);
                    counts.script_items_total = counts
                        .script_items_total
                        .saturating_add(campaign.script_count() as u64);

                    if let Some(previous) =
                        seen_paths_by_id.insert(campaign.id.clone(), manifest_path.clone())
                    {
                        counts.duplicate_id_total = counts.duplicate_id_total.saturating_add(1);
                        let example = format!(
                            "{} -> {} | {}",
                            campaign.id,
                            previous.display(),
                            manifest_path.display()
                        );
                        push_bounded(&mut examples.duplicate_ids, max_examples, example);
                        errors.push(format!(
                            "duplicate campaign id `{}` in manifests: {} and {}",
                            campaign.id,
                            previous.display(),
                            manifest_path.display()
                        ));
                    }
                }
                Err(error) => {
                    counts.manifest_invalid_total = counts.manifest_invalid_total.saturating_add(1);
                    push_bounded(
                        &mut examples.invalid_manifest_paths,
                        max_examples,
                        manifest_path.display().to_string(),
                    );
                    errors.push(error);
                }
            }
        }

        if counts.manifest_invalid_total > 0 {
            repairs.push(json!({
                "code": "diag-campaigns.validate",
                "note": "Inspect repo-owned campaign manifests with the validation command.",
                "command": "cargo run -p fretboard -- diag campaign validate --json",
            }));
        }

        if counts.legacy_shape_total > 0 {
            let message = format!(
                "some campaign manifests still use legacy top-level `suites`/`scripts` shape: {}",
                counts.legacy_shape_total
            );
            if strict {
                errors.push(format!("strict: {message}"));
            } else {
                warnings.push(message);
            }
            repairs.push(json!({
                "code": "diag-campaigns.migrate-items",
                "note": "Prefer ordered `items` in repo-owned campaign manifests; keep legacy top-level `suites`/`scripts` only for temporary compatibility.",
            }));
        }
    }

    let ok = errors.is_empty();

    let mut counts_obj = BTreeMap::<String, Value>::new();
    counts_obj.insert(
        "manifest_json_total".to_string(),
        json!(counts.manifest_json_total),
    );
    counts_obj.insert(
        "manifest_valid_total".to_string(),
        json!(counts.manifest_valid_total),
    );
    counts_obj.insert(
        "manifest_invalid_total".to_string(),
        json!(counts.manifest_invalid_total),
    );
    counts_obj.insert(
        "duplicate_id_total".to_string(),
        json!(counts.duplicate_id_total),
    );
    counts_obj.insert(
        "legacy_shape_total".to_string(),
        json!(counts.legacy_shape_total),
    );
    counts_obj.insert("campaigns_total".to_string(), json!(counts.campaigns_total));
    counts_obj.insert(
        "suite_items_total".to_string(),
        json!(counts.suite_items_total),
    );
    counts_obj.insert(
        "script_items_total".to_string(),
        json!(counts.script_items_total),
    );

    Ok(json!({
        "ok": ok,
        "required_ok": ok,
        "campaigns_root": campaigns_root.display().to_string(),
        "strict": strict,
        "errors": errors,
        "warnings": warnings,
        "repairs": repairs,
        "counts": Value::Object(counts_obj.into_iter().collect()),
        "examples": {
            "invalid_manifest_paths": examples.invalid_manifest_paths,
            "duplicate_ids": examples.duplicate_ids,
            "legacy_shape_paths": examples.legacy_shape_paths,
        }
    }))
}

pub(crate) fn cmd_doctor_campaigns(
    rest: &[String],
    workspace_root: &Path,
    stats_json: bool,
) -> Result<(), String> {
    let mut strict = false;

    let mut index = 0usize;
    while index < rest.len() {
        match rest[index].as_str() {
            "--strict" => {
                strict = true;
                index += 1;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag for doctor campaigns: {other}"));
            }
            other => {
                return Err(format!("unexpected argument for doctor campaigns: {other}"));
            }
        }
    }

    let payload = build_doctor_campaigns_payload(workspace_root, strict, 20)?;
    let ok = payload
        .get("ok")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);

    if stats_json {
        let pretty = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string());
        println!("{pretty}");
        if !ok {
            std::process::exit(1);
        }
        return Ok(());
    }

    let campaigns_root = payload
        .get("campaigns_root")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    println!("campaigns_root: {campaigns_root}");
    println!("ok: {ok}");

    for error in payload
        .get("errors")
        .and_then(|value| value.as_array())
        .map(|value| value.as_slice())
        .unwrap_or(&[])
    {
        if let Some(error) = error.as_str() {
            println!("error: {error}");
        }
    }

    for warning in payload
        .get("warnings")
        .and_then(|value| value.as_array())
        .map(|value| value.as_slice())
        .unwrap_or(&[])
    {
        if let Some(warning) = warning.as_str() {
            println!("warning: {warning}");
        }
    }

    for repair in payload
        .get("repairs")
        .and_then(|value| value.as_array())
        .map(|value| value.as_slice())
        .unwrap_or(&[])
    {
        let code = repair
            .get("code")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        let note = repair
            .get("note")
            .and_then(|value| value.as_str())
            .unwrap_or("");
        if let Some(command) = repair.get("command").and_then(|value| value.as_str()) {
            println!("repair: {code} ({note})");
            println!("  command: {command}");
        } else if !note.is_empty() {
            println!("repair: {code} ({note})");
        }
    }

    if !ok {
        println!("tip: re-run with `--json` for a structured report");
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_test_root(label: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "fret-diag-doctor-campaigns-{label}-{}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");
        root
    }

    #[test]
    fn doctor_campaigns_payload_reports_valid_repo_manifests() {
        let root = temp_test_root("valid");
        let manifests_dir = campaigns_dir_from_workspace_root(&root);
        std::fs::create_dir_all(&manifests_dir).expect("create manifests dir");
        std::fs::write(
            manifests_dir.join("ui-gallery-smoke.json"),
            r#"{
  "schema_version": 1,
  "kind": "diag_campaign_manifest",
  "id": "ui-gallery-smoke",
  "description": "Valid campaign manifest.",
  "lane": "smoke",
  "items": [
    { "kind": "suite", "value": "ui-gallery-lite-smoke" },
    { "kind": "script", "value": "tools/diag-scripts/ui-gallery-layout.json" }
  ]
}"#,
        )
        .expect("write manifest");

        let payload = build_doctor_campaigns_payload(&root, false, 20).expect("payload");

        let _ = std::fs::remove_dir_all(&root);
        assert_eq!(
            payload.get("ok").and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            payload
                .pointer("/counts/manifest_valid_total")
                .and_then(|value| value.as_u64()),
            Some(1)
        );
        assert_eq!(
            payload
                .pointer("/counts/suite_items_total")
                .and_then(|value| value.as_u64()),
            Some(1)
        );
        assert_eq!(
            payload
                .pointer("/counts/script_items_total")
                .and_then(|value| value.as_u64()),
            Some(1)
        );
    }

    #[test]
    fn doctor_campaigns_payload_reports_duplicate_invalid_and_legacy_shapes() {
        let root = temp_test_root("mixed");
        let manifests_dir = campaigns_dir_from_workspace_root(&root);
        std::fs::create_dir_all(&manifests_dir).expect("create manifests dir");

        std::fs::write(
            manifests_dir.join("legacy.json"),
            r#"{
  "schema_version": 1,
  "kind": "diag_campaign_manifest",
  "id": "legacy-shape",
  "description": "Legacy top-level shape.",
  "lane": "smoke",
  "suites": ["ui-gallery-lite-smoke"]
}"#,
        )
        .expect("write legacy manifest");

        std::fs::write(
            manifests_dir.join("duplicate-a.json"),
            r#"{
  "schema_version": 1,
  "kind": "diag_campaign_manifest",
  "id": "duplicate-id",
  "description": "Duplicate A.",
  "lane": "smoke",
  "items": [{ "kind": "suite", "value": "ui-gallery-lite-smoke" }]
}"#,
        )
        .expect("write duplicate a");

        std::fs::write(
            manifests_dir.join("duplicate-b.json"),
            r#"{
  "schema_version": 1,
  "kind": "diag_campaign_manifest",
  "id": "duplicate-id",
  "description": "Duplicate B.",
  "lane": "smoke",
  "items": [{ "kind": "script", "value": "tools/diag-scripts/ui-gallery-layout.json" }]
}"#,
        )
        .expect("write duplicate b");

        std::fs::write(
            manifests_dir.join("invalid.json"),
            r#"{
  "schema_version": 99,
  "kind": "diag_campaign_manifest",
  "id": "invalid",
  "description": "Invalid schema.",
  "lane": "smoke",
  "items": [{ "kind": "suite", "value": "ui-gallery-lite-smoke" }]
}"#,
        )
        .expect("write invalid");

        let payload = build_doctor_campaigns_payload(&root, false, 20).expect("payload");

        let _ = std::fs::remove_dir_all(&root);
        assert_eq!(
            payload.get("ok").and_then(|value| value.as_bool()),
            Some(false)
        );
        assert_eq!(
            payload
                .pointer("/counts/manifest_invalid_total")
                .and_then(|value| value.as_u64()),
            Some(1)
        );
        assert_eq!(
            payload
                .pointer("/counts/duplicate_id_total")
                .and_then(|value| value.as_u64()),
            Some(1)
        );
        assert_eq!(
            payload
                .pointer("/counts/legacy_shape_total")
                .and_then(|value| value.as_u64()),
            Some(1)
        );
        assert!(
            payload
                .get("warnings")
                .and_then(|value| value.as_array())
                .is_some_and(|warnings| warnings.iter().any(|entry| {
                    entry
                        .as_str()
                        .is_some_and(|entry| entry.contains("legacy top-level `suites`/`scripts`"))
                }))
        );
    }

    #[test]
    fn doctor_campaigns_payload_treats_legacy_shape_as_error_in_strict_mode() {
        let root = temp_test_root("strict");
        let manifests_dir = campaigns_dir_from_workspace_root(&root);
        std::fs::create_dir_all(&manifests_dir).expect("create manifests dir");
        std::fs::write(
            manifests_dir.join("legacy.json"),
            r#"{
  "schema_version": 1,
  "kind": "diag_campaign_manifest",
  "id": "legacy-shape",
  "description": "Legacy top-level shape.",
  "lane": "smoke",
  "scripts": ["tools/diag-scripts/ui-gallery-layout.json"]
}"#,
        )
        .expect("write legacy manifest");

        let payload = build_doctor_campaigns_payload(&root, true, 20).expect("payload");

        let _ = std::fs::remove_dir_all(&root);
        assert_eq!(
            payload.get("ok").and_then(|value| value.as_bool()),
            Some(false)
        );
        assert!(
            payload
                .get("errors")
                .and_then(|value| value.as_array())
                .is_some_and(|errors| errors.iter().any(|entry| {
                    entry.as_str().is_some_and(|entry| {
                        entry.contains("strict: some campaign manifests still use legacy")
                    })
                }))
        );
    }
}
