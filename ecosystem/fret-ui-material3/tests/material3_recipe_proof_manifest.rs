//! Coverage matrix checks for public Material3 recipe proof artifacts.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

const MATERIAL3_RECIPE_PROOF_MANIFEST_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_recipe_proof_manifest_v1.json"
));
const MATERIAL3_TOKEN_VISUAL_CASES_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_token_visual_cases_v1.json"
));
const MATERIAL3_HEADLESS_GOLDEN_TESTS: &str = include_str!("material3_headless_goldens.rs");
const MATERIAL3_HEADLESS_GOLDEN_SCALE_SEGMENTS: [&str; 3] = ["scale1_0", "scale1_25", "scale2_0"];
const MATERIAL3_HEADLESS_GOLDEN_SCHEME_LABELS: [&str; 4] = [
    "dark.tonal_spot",
    "light.tonal_spot",
    "dark.expressive",
    "light.expressive",
];
const SUPPORTING_API_STATUS_ALLOWLIST: [&str; 1] = ["motion"];

#[derive(Debug, Deserialize)]
struct Material3RecipeProofManifestV1 {
    schema_version: u32,
    suite: String,
    notes: String,
    entries: Vec<Material3RecipeProofEntryV1>,
}

#[derive(Debug, Deserialize)]
struct Material3RecipeProofEntryV1 {
    id: String,
    source: String,
    token_visual_component: Option<String>,
    headless_golden_suites: Vec<String>,
    behavior_tests: Vec<String>,
    proof_status: Material3RecipeProofStatusV1,
    known_gap: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Material3RecipeProofStatusV1 {
    HeadlessGolden,
    BehaviorOnly,
    TokenOnly,
    SupportingApi,
}

#[derive(Debug, Deserialize)]
struct Material3TokenVisualSuiteV1 {
    cases: Vec<Material3TokenVisualCaseV1>,
}

#[derive(Debug, Deserialize)]
struct Material3TokenVisualCaseV1 {
    component: String,
}

#[test]
fn material3_recipe_proof_manifest_tracks_public_recipe_coverage_v1() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = repo_root_for_crate_dir(&crate_dir);
    let manifest: Material3RecipeProofManifestV1 =
        serde_json::from_str(MATERIAL3_RECIPE_PROOF_MANIFEST_V1)
            .expect("material3 recipe proof manifest must parse");

    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.suite, "material3-recipe-proof-manifest-v1");
    assert!(
        manifest.notes.contains("headless_golden_suites"),
        "manifest notes should define proof vocabulary"
    );

    let manifest_sources: BTreeSet<_> = manifest
        .entries
        .iter()
        .map(|entry| entry.source.clone())
        .collect();
    assert_eq!(
        manifest_sources,
        public_root_recipe_sources(&crate_dir),
        "every public root recipe source must appear in the proof manifest"
    );

    assert_sorted_unique_manifest_entries(&manifest.entries);
    assert_token_visual_components_match_manifest(&manifest.entries);

    for entry in &manifest.entries {
        assert_source_matches_entry_id(entry);
        assert!(
            crate_dir.join(&entry.source).exists(),
            "{} must exist",
            entry.source
        );

        for suite in &entry.headless_golden_suites {
            let runner = crate_dir
                .join("tests/support/headless_golden_runners")
                .join(format!("{suite}.rs"));
            assert!(
                runner.exists(),
                "{} references missing runner {suite}",
                entry.id
            );

            let test_marker = format!("material3_headless_{suite}_suite_goldens_v1");
            assert!(
                MATERIAL3_HEADLESS_GOLDEN_TESTS.contains(&test_marker),
                "{} references runner {suite}, but material3_headless_goldens.rs does not call {test_marker}",
                entry.id
            );
            assert_headless_golden_suite_files_exist(&repo_root, &entry.id, suite);
        }

        for test in &entry.behavior_tests {
            assert!(
                crate_dir.join("tests").join(test).exists(),
                "{} references missing behavior test {test}",
                entry.id
            );
        }

        assert_status_contract(entry);
    }
}

fn public_root_recipe_sources(crate_dir: &Path) -> BTreeSet<String> {
    fs::read_dir(crate_dir.join("src"))
        .expect("material3 src directory must be readable")
        .map(|entry| entry.expect("material3 src entry must be readable").path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .filter_map(|path| {
            let file = path.file_name()?.to_str()?;
            (file != "lib.rs").then(|| format!("src/{file}"))
        })
        .collect()
}

fn repo_root_for_crate_dir(crate_dir: &Path) -> PathBuf {
    crate_dir
        .parent()
        .and_then(|path| path.parent())
        .map(Path::to_path_buf)
        .expect("material3 crate must live under ecosystem/<crate>")
}

fn assert_sorted_unique_manifest_entries(entries: &[Material3RecipeProofEntryV1]) {
    for pair in entries.windows(2) {
        assert!(
            pair[0].id.as_str() < pair[1].id.as_str(),
            "recipe proof manifest entries must be sorted by id and unique; found {} then {}",
            pair[0].id,
            pair[1].id
        );
    }
}

fn assert_token_visual_components_match_manifest(entries: &[Material3RecipeProofEntryV1]) {
    let suite: Material3TokenVisualSuiteV1 = serde_json::from_str(MATERIAL3_TOKEN_VISUAL_CASES_V1)
        .expect("material3 token visual suite must parse");
    let actual: BTreeSet<_> = suite.cases.into_iter().map(|case| case.component).collect();
    let manifest: BTreeSet<_> = entries
        .iter()
        .filter_map(|entry| entry.token_visual_component.clone())
        .collect();

    assert_eq!(
        manifest, actual,
        "recipe proof manifest token_visual_component set must match material3_token_visual_cases_v1 components"
    );
}

fn assert_source_matches_entry_id(entry: &Material3RecipeProofEntryV1) {
    let stem = Path::new(&entry.source)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .expect("recipe source must have a utf-8 file stem");
    assert_eq!(
        entry.id.as_str(),
        stem,
        "{} should use the source file stem as its stable manifest id",
        entry.source
    );
}

fn assert_status_contract(entry: &Material3RecipeProofEntryV1) {
    match entry.proof_status {
        Material3RecipeProofStatusV1::HeadlessGolden => {
            assert!(
                !entry.headless_golden_suites.is_empty(),
                "{} is marked headless_golden but has no headless_golden_suites",
                entry.id
            );
            assert!(
                entry.known_gap.is_none(),
                "{} is marked headless_golden but still carries a known_gap",
                entry.id
            );
        }
        Material3RecipeProofStatusV1::BehaviorOnly => {
            panic!(
                "{} is behavior_only; public Material3 recipe proof entries must now use headless_golden or supporting_api",
                entry.id
            );
        }
        Material3RecipeProofStatusV1::TokenOnly => {
            panic!(
                "{} is token_only; public Material3 recipe proof entries must now use headless_golden or supporting_api",
                entry.id
            );
        }
        Material3RecipeProofStatusV1::SupportingApi => {
            assert!(
                SUPPORTING_API_STATUS_ALLOWLIST.contains(&entry.id.as_str()),
                "{} is supporting_api but is not in the explicit allowlist {:?}",
                entry.id,
                SUPPORTING_API_STATUS_ALLOWLIST
            );
            assert!(
                entry.token_visual_component.is_none(),
                "{} is supporting_api but lists a token_visual_component",
                entry.id
            );
            assert_known_gap(entry);
        }
    }
}

fn assert_headless_golden_suite_files_exist(repo_root: &Path, entry_id: &str, suite: &str) {
    let golden_stem = suite.replace('_', "-");
    let golden_dir = repo_root
        .join("goldens")
        .join("material3-headless")
        .join("v1");

    for scale in MATERIAL3_HEADLESS_GOLDEN_SCALE_SEGMENTS {
        for label in MATERIAL3_HEADLESS_GOLDEN_SCHEME_LABELS {
            let file_name = format!("material3-{golden_stem}.{scale}.{label}.json");
            let path = golden_dir.join(&file_name);
            assert!(
                path.exists(),
                "{entry_id} references headless golden suite {suite}, but golden file is missing: {}",
                path.display()
            );
        }
    }
}

fn assert_known_gap(entry: &Material3RecipeProofEntryV1) {
    assert!(
        entry
            .known_gap
            .as_deref()
            .is_some_and(|gap| !gap.trim().is_empty()),
        "{} must explain its known proof gap",
        entry.id
    );
}
