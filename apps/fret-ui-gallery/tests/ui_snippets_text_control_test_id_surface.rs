mod support;

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use support::{manifest_path, read_path, rust_sources};

fn canonicalize_rust_fragment(fragment: &str) -> String {
    fragment.split_whitespace().collect()
}

fn visit_json_files(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("read_dir failed for {}: {err}", dir.display()));

    for entry in entries {
        let path = entry.expect("read_dir entry").path();
        if path.is_dir() {
            visit_json_files(&path, files);
            continue;
        }

        if path.extension() == Some(OsStr::new("json")) {
            files.push(path);
        }
    }
}

fn collect_set_text_value_target_ids(script: &Value, target_ids: &mut BTreeSet<String>) {
    let Some(steps) = script.get("steps").and_then(Value::as_array) else {
        return;
    };

    for step in steps {
        if step.get("type").and_then(Value::as_str) != Some("set_text_value") {
            continue;
        }

        let Some(id) = step
            .get("target")
            .and_then(|target| target.get("id"))
            .and_then(Value::as_str)
        else {
            continue;
        };

        target_ids.insert(id.to_owned());
    }
}

fn ui_gallery_set_text_value_target_ids() -> BTreeSet<String> {
    let scripts_root = manifest_path("../../tools/diag-scripts/ui-gallery");
    let mut script_paths = Vec::new();
    visit_json_files(&scripts_root, &mut script_paths);
    script_paths.sort();

    let mut target_ids = BTreeSet::new();
    for path in script_paths {
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read_to_string failed for {}: {err}", path.display()));
        let script: Value = serde_json::from_str(&text)
            .unwrap_or_else(|err| panic!("invalid diag script JSON in {}: {err}", path.display()));
        collect_set_text_value_target_ids(&script, &mut target_ids);
    }

    target_ids
}

#[test]
fn set_text_value_targets_are_not_stamped_after_landing_snippet_controls() {
    let target_ids = ui_gallery_set_text_value_target_ids();
    assert!(
        target_ids.contains("ui-gallery-input-basic-control"),
        "the Input long-text diagnostics gate should keep the Basic Input direct-control target visible to this authoring-surface test",
    );
    assert!(
        !target_ids.is_empty(),
        "UI Gallery diagnostics should expose at least one set_text_value target for direct-control authoring checks",
    );

    let mut violations = Vec::new();
    for path in rust_sources("src/ui/snippets") {
        let source = read_path(&path);
        let canonical_source = canonicalize_rust_fragment(&source);

        for id in &target_ids {
            let forbidden =
                canonicalize_rust_fragment(&format!(".into_element(cx).test_id(\"{id}\")"));
            if canonical_source.contains(&forbidden) {
                violations.push(format!(
                    "{} uses post-landing test_id for `{id}`",
                    path.display()
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "editable diagnostics targets must stamp test ids on the direct control builder surface, not after `.into_element(cx)`: {}",
        violations.join("; ")
    );
}
