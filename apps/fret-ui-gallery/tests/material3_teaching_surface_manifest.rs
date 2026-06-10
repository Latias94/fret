#![cfg(feature = "gallery-material3")]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

const MATERIAL3_RECIPE_PROOF_MANIFEST_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../ecosystem/fret-ui-material3/tests/fixtures/material3_recipe_proof_manifest_v1.json"
));
const MATERIAL3_TEACHING_SURFACE_MANIFEST_V1: &str =
    include_str!("fixtures/material3_teaching_surface_manifest_v1.json");
const MATERIAL3_STATE_MATRIX_COMPACT_VISUALS_DIAG: &str = include_str!(
    "../../../tools/diag-scripts/ui-gallery/material3/ui-gallery-material3-state-matrix-compact-visuals.json"
);
const MATERIAL3_SNIPPET_MOD: &str = include_str!("../src/ui/snippets/material3/mod.rs");
const GALLERY_SPEC: &str = include_str!("../src/spec.rs");
const GALLERY_CONTENT: &str = include_str!("../src/ui/content.rs");

#[test]
fn material3_teaching_surface_manifest_tracks_recipe_proof_manifest_v1() {
    let proof = parse_json(MATERIAL3_RECIPE_PROOF_MANIFEST_V1);
    let teaching = parse_json(MATERIAL3_TEACHING_SURFACE_MANIFEST_V1);

    assert_eq!(teaching["schema_version"], 1);
    assert_eq!(teaching["suite"], "material3-teaching-surface-manifest-v1");

    let proof_ids = rendered_recipe_ids(&proof);
    let teaching_entries = entries(&teaching);
    let teaching_ids: BTreeSet<_> = teaching_entries
        .iter()
        .map(|entry| string_field(entry, "id").to_string())
        .collect();

    assert_eq!(
        teaching_ids, proof_ids,
        "Material3 gallery teaching surface manifest must cover every rendered public recipe proof entry; supporting APIs are excluded"
    );

    assert_sorted_unique(teaching_entries, "teaching surface manifest");

    for entry in teaching_entries {
        assert_valid_coverage(entry);

        let id = string_field(entry, "id");
        let snippet_modules = string_array_field(entry, "snippet_modules");
        let page_ids = string_array_field(entry, "page_ids");
        let evidence_terms = string_array_field(entry, "evidence_terms");

        assert!(
            !snippet_modules.is_empty(),
            "{id} must name at least one Material3 snippet module"
        );
        assert!(
            !page_ids.is_empty(),
            "{id} must name at least one Material3 gallery page"
        );
        assert!(
            !evidence_terms.is_empty(),
            "{id} must name at least one source evidence term"
        );

        let snippet_source = assert_snippet_modules_exist(id, &snippet_modules);
        for term in evidence_terms {
            assert!(
                snippet_source.contains(term),
                "{id} teaching surface should contain evidence term `{term}` in one of {:?}",
                snippet_modules
            );
        }

        for page_id in page_ids {
            assert_material3_page_is_registered(id, page_id);
        }
    }
}

#[test]
fn material3_state_matrix_compact_visuals_diag_covers_text_gated_surfaces() {
    let script = parse_json(MATERIAL3_STATE_MATRIX_COMPACT_VISUALS_DIAG);

    assert_eq!(script["schema_version"], 2);
    assert_array_contains(
        &script["meta"]["required_capabilities"],
        "diag.screenshot_png",
        "state matrix compact visuals diag capabilities",
    );
    assert_array_contains(
        &script["meta"]["required_launch_features"],
        "gallery-material3",
        "state matrix compact visuals launch features",
    );
    assert_eq!(
        script["meta"]["env_defaults"]["FRET_UI_GALLERY_START_PAGE"],
        "material3_state_matrix"
    );

    let steps = script["steps"]
        .as_array()
        .expect("diag script steps must be an array");
    assert!(
        steps
            .iter()
            .any(|step| step["type"] == "capture_layout_sidecar"),
        "state matrix compact visuals diag should capture layout sidecars"
    );
    assert!(
        steps
            .iter()
            .any(|step| step["type"] == "capture_screenshot"),
        "state matrix compact visuals diag should capture screenshots"
    );
    assert!(
        steps.iter().any(|step| step["type"] == "capture_bundle"),
        "state matrix compact visuals diag should capture a bundle"
    );

    for needle in [
        "ui-gallery-material3-carousel-item-standard",
        "ui-gallery-material3-carousel-item-outlined",
        "ui-gallery-material3-carousel-item-disabled",
        "ui-gallery-material3-divider-horizontal",
        "ui-gallery-material3-divider-vertical",
        "ui-gallery-material3-linear-progress",
        "ui-gallery-material3-linear-progress-indeterminate",
        "ui-gallery-material3-circular-progress-four-color",
        "ui-gallery-material3-search-bar",
        "ui-gallery-material3-state-matrix-carousel-items.layout",
        "ui-gallery-material3-state-matrix-dividers.layout",
        "ui-gallery-material3-state-matrix-progress-indicators.layout",
        "ui-gallery-material3-state-matrix-search-bar.layout",
        "ui-gallery-material3-state-matrix-compact-visuals",
    ] {
        assert!(
            MATERIAL3_STATE_MATRIX_COMPACT_VISUALS_DIAG.contains(needle),
            "state matrix compact visuals diag should cover `{needle}`"
        );
    }
}

fn parse_json(source: &str) -> Value {
    serde_json::from_str(source).expect("manifest json must parse")
}

fn entries(manifest: &Value) -> &[Value] {
    manifest["entries"]
        .as_array()
        .expect("manifest entries must be an array")
}

fn rendered_recipe_ids(proof: &Value) -> BTreeSet<String> {
    entries(proof)
        .iter()
        .filter(|entry| string_field(entry, "proof_status") != "supporting_api")
        .map(|entry| string_field(entry, "id").to_string())
        .collect()
}

fn string_field<'a>(entry: &'a Value, field: &str) -> &'a str {
    entry[field]
        .as_str()
        .unwrap_or_else(|| panic!("manifest entry must have string field `{field}`: {entry:?}"))
}

fn string_array_field<'a>(entry: &'a Value, field: &str) -> Vec<&'a str> {
    entry[field]
        .as_array()
        .unwrap_or_else(|| panic!("manifest entry must have array field `{field}`: {entry:?}"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("manifest field `{field}` must contain strings"))
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

fn assert_sorted_unique(entries: &[Value], label: &str) {
    for pair in entries.windows(2) {
        let a = string_field(&pair[0], "id");
        let b = string_field(&pair[1], "id");
        assert!(
            a < b,
            "{label} entries must be sorted and unique: {a} then {b}"
        );
    }
}

fn assert_valid_coverage(entry: &Value) {
    let id = string_field(entry, "id");
    let coverage = string_field(entry, "coverage");
    assert!(
        matches!(coverage, "dedicated_page" | "family_page" | "state_matrix"),
        "{id} uses unknown teaching coverage `{coverage}`"
    );
}

fn assert_snippet_modules_exist(entry_id: &str, modules: &[&str]) -> String {
    let mut combined = String::new();

    for module in modules {
        assert!(
            MATERIAL3_SNIPPET_MOD.contains(&format!("pub mod {module};")),
            "{entry_id} references missing snippets::material3::{module} module declaration"
        );

        let source_path = material3_snippet_path(module);
        let source = fs::read_to_string(&source_path).unwrap_or_else(|err| {
            panic!("read_to_string failed for {}: {err}", source_path.display())
        });

        assert!(
            source.contains(&format!("include_str!(\"{module}.rs\")")),
            "{entry_id} snippet module {module} must expose its own SOURCE via include_str"
        );
        assert!(
            source.contains("pub fn render"),
            "{entry_id} snippet module {module} must expose a render function"
        );

        combined.push_str(&source);
        combined.push('\n');
    }

    combined
}

fn material3_snippet_path(module: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/ui/snippets/material3")
        .join(format!("{module}.rs"))
}

fn assert_material3_page_is_registered(entry_id: &str, page_id: &str) {
    let page_const = material3_page_const(page_id);
    let normalized_spec = normalize_ws(GALLERY_SPEC);
    assert!(
        GALLERY_SPEC.contains(&format!("const {page_const}: &str"))
            && GALLERY_SPEC.contains(&format!("\"{page_id}\"")),
        "{entry_id} references missing Material3 gallery page id {page_id}"
    );
    assert!(
        normalized_spec.contains(&format!("PageSpec::new({page_const},")),
        "{entry_id} page id {page_id} should be declared and included in Material3 PageSpec entries"
    );
    assert!(
        GALLERY_CONTENT.contains(&format!("{page_const} =>")),
        "{entry_id} page id {page_id} should have a render dispatch in ui/content.rs"
    );
}

fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

fn material3_page_const(page_id: &str) -> String {
    let suffix = page_id
        .strip_prefix("material3_")
        .unwrap_or_else(|| panic!("Material3 page id must start with material3_: {page_id}"))
        .to_ascii_uppercase();
    format!("PAGE_MATERIAL3_{suffix}")
}
