#![cfg(feature = "gallery-material3")]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

const MATERIAL3_RUNTIME_EVIDENCE_SUITE_ID: &str = "ui-gallery-material3-runtime-evidence";
const MATERIAL3_RUNTIME_EVIDENCE_SUITE: &str = include_str!(
    "../../../tools/diag-scripts/suites/ui-gallery-material3-runtime-evidence/suite.json"
);
const MATERIAL3_RUNTIME_EVIDENCE_CAMPAIGN: &str =
    include_str!("../../../tools/diag-campaigns/ui-gallery-material3-runtime-evidence.json");

#[test]
fn material3_runtime_evidence_suite_is_curated_script_v2_evidence() {
    let suite = parse_json(MATERIAL3_RUNTIME_EVIDENCE_SUITE);

    assert_eq!(suite["schema_version"], 1);
    assert_eq!(suite["kind"], "diag_script_suite_manifest");

    let scripts = string_array_field(&suite, "scripts");
    assert!(
        scripts.len() >= 30,
        "Material3 runtime evidence should cover the broad promoted script set"
    );
    assert_sorted_unique(&scripts, "Material3 runtime evidence suite");

    let script_set: BTreeSet<_> = scripts.iter().copied().collect();
    for expected in [
        "tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-state-matrix-chip-action-state.json",
        "tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-state-matrix-compact-visuals.json",
        "tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-chip-visual-chrome.json",
        "tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-segmented-button-roving-semantics-screenshots.json",
        "tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-dialog-focus-trap-restore.json",
        "tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-dialog-select-nested-overlay.json",
        "tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-autocomplete-dialog-nested-overlay.json",
        "tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-bottom-sheet-fields-nested-overlays.json",
        "tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-menu-focus-dismiss.json",
        "tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-search-menu-sibling-popovers.json",
        "tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-search-view-edge-fullscreen-composition.json",
        "tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-navigation-routed-content.json",
        "tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-modal-navigation-drawer-routed-content.json",
        "tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-text-field-hover-label-color-expressive-screenshots.json",
        "tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-switch-icons-state-matrix-screenshots.json",
    ] {
        assert!(
            script_set.contains(expected),
            "Material3 runtime evidence suite should include `{expected}`"
        );
    }

    for script_path in scripts {
        assert!(
            script_path.starts_with("tools/diag-scripts/ui-gallery/material3/"),
            "Material3 runtime evidence should only include Material3 gallery scripts: {script_path}"
        );

        let script_source = read_repo_file(script_path);
        let script = parse_json(&script_source);

        assert_eq!(
            script["schema_version"], 2,
            "{script_path} should stay on diag script v2"
        );
        assert_array_contains(
            &script["meta"]["required_launch_features"],
            "gallery-material3",
            &format!("{script_path} launch features"),
        );
        assert_array_contains(
            &script["meta"]["required_capabilities"],
            "diag.script_v2",
            &format!("{script_path} required capabilities"),
        );

        let step_types = collect_step_types(&script["steps"]);
        assert!(
            step_types.contains("capture_bundle"),
            "{script_path} should capture a diagnostics bundle"
        );
        if step_types.contains("capture_screenshot") {
            assert_array_contains(
                &script["meta"]["required_capabilities"],
                "diag.screenshot_png",
                &format!("{script_path} screenshot capability"),
            );
        }
    }
}

#[test]
fn material3_runtime_evidence_suite_is_promoted_in_registry() {
    let suite = parse_json(MATERIAL3_RUNTIME_EVIDENCE_SUITE);
    let registry = parse_json(&read_repo_file("tools/diag-scripts/index.json"));
    let registry_entries = registry["scripts"]
        .as_array()
        .expect("registry scripts must be an array");

    for script_path in string_array_field(&suite, "scripts") {
        let entry = registry_entries
            .iter()
            .find(|entry| entry["path"].as_str() == Some(script_path))
            .unwrap_or_else(|| panic!("registry should promote suite script `{script_path}`"));
        assert_array_contains(
            &entry["suite_memberships"],
            MATERIAL3_RUNTIME_EVIDENCE_SUITE_ID,
            &format!("{script_path} suite memberships"),
        );
    }
}

#[test]
fn material3_runtime_evidence_campaign_targets_the_suite() {
    let campaign = parse_json(MATERIAL3_RUNTIME_EVIDENCE_CAMPAIGN);

    assert_eq!(campaign["schema_version"], 1);
    assert_eq!(campaign["kind"], "diag_campaign_manifest");
    assert_eq!(campaign["id"], MATERIAL3_RUNTIME_EVIDENCE_SUITE_ID);
    assert_eq!(campaign["lane"], "correctness");
    assert_eq!(campaign["profile"], "bounded");
    assert_eq!(campaign["tier"], "correctness");

    let items = campaign["items"]
        .as_array()
        .expect("campaign items must be an array");
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["kind"], "suite");
    assert_eq!(items[0]["value"], MATERIAL3_RUNTIME_EVIDENCE_SUITE_ID);

    for expected_tag in ["ui-gallery", "material3", "runtime-evidence"] {
        assert_array_contains(
            &campaign["tags"],
            expected_tag,
            "Material3 runtime evidence campaign tags",
        );
    }
}

fn parse_json(source: &str) -> Value {
    serde_json::from_str(source).expect("json should parse")
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_repo_file(path: &str) -> String {
    let path = repo_root().join(path);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read_to_string failed for {}: {err}", path.display()))
}

fn string_array_field<'a>(entry: &'a Value, field: &str) -> Vec<&'a str> {
    entry[field]
        .as_array()
        .unwrap_or_else(|| panic!("json field `{field}` must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("json field `{field}` must contain strings"))
        })
        .collect()
}

fn assert_array_contains(value: &Value, expected: &str, label: &str) {
    let values = value
        .as_array()
        .unwrap_or_else(|| panic!("{label} must be an array"));
    assert!(
        values.iter().any(|value| value.as_str() == Some(expected)),
        "{label} should contain `{expected}`"
    );
}

fn assert_sorted_unique(values: &[&str], label: &str) {
    for pair in values.windows(2) {
        assert!(
            pair[0] < pair[1],
            "{label} entries must be sorted and unique: {} then {}",
            pair[0],
            pair[1]
        );
    }
}

fn collect_step_types(value: &Value) -> BTreeSet<String> {
    let mut types = BTreeSet::new();
    collect_step_types_inner(value, &mut types);
    types
}

fn collect_step_types_inner(value: &Value, types: &mut BTreeSet<String>) {
    match value {
        Value::Object(object) => {
            if let Some(step_type) = object.get("type").and_then(Value::as_str) {
                types.insert(step_type.to_string());
            }
            for child in object.values() {
                collect_step_types_inner(child, types);
            }
        }
        Value::Array(values) => {
            for child in values {
                collect_step_types_inner(child, types);
            }
        }
        _ => {}
    }
}
