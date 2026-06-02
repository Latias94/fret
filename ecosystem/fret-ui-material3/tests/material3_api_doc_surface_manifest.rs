//! API documentation surface checks for public Material3 recipe ergonomics.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

const MATERIAL3_API_DOC_SURFACE_MANIFEST_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_api_doc_surface_manifest_v1.json"
));
const MATERIAL3_RECIPE_PROOF_MANIFEST_V1: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_recipe_proof_manifest_v1.json"
));

#[derive(Debug, Deserialize)]
struct ApiDocSurfaceManifestV1 {
    schema_version: u32,
    suite: String,
    notes: String,
    required_crate_doc_terms: Vec<String>,
    required_readme_terms: Vec<String>,
    component_families: Vec<ComponentFamilyV1>,
    supporting_api_sources: Vec<String>,
    style_surfaces: Vec<StyleSurfaceV1>,
    copyable_state_sources: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ComponentFamilyV1 {
    id: String,
    heading: String,
    source_ids: Vec<String>,
    exports: Vec<String>,
    doc_terms: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct StyleSurfaceV1 {
    source_id: String,
    style_export: String,
}

#[derive(Debug, Deserialize)]
struct RecipeProofManifestV1 {
    entries: Vec<RecipeProofEntryV1>,
}

#[derive(Debug, Deserialize)]
struct RecipeProofEntryV1 {
    id: String,
    source: String,
}

#[test]
fn material3_api_doc_surface_manifest_tracks_public_authoring_surface_v1() {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest: ApiDocSurfaceManifestV1 =
        serde_json::from_str(MATERIAL3_API_DOC_SURFACE_MANIFEST_V1)
            .expect("material3 api doc surface manifest must parse");
    let recipe_manifest: RecipeProofManifestV1 =
        serde_json::from_str(MATERIAL3_RECIPE_PROOF_MANIFEST_V1)
            .expect("material3 recipe proof manifest must parse");

    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.suite, "material3-api-doc-surface-v1");
    assert!(
        manifest.notes.contains("style_surfaces"),
        "manifest notes should define the style surface contract"
    );
    assert_sorted_unique_by_id(&manifest.component_families);
    assert_sorted_unique_strings("supporting_api_sources", &manifest.supporting_api_sources);
    assert_sorted_unique_strings("copyable_state_sources", &manifest.copyable_state_sources);

    let lib_source = read_source(&crate_dir.join("src/lib.rs"));
    let crate_docs = leading_crate_docs(&lib_source);
    let readme = read_source(&crate_dir.join("README.md"));
    let reexports = public_reexport_block(&lib_source);
    let recipe_sources = recipe_sources_by_id(&recipe_manifest);

    for term in &manifest.required_crate_doc_terms {
        assert_contains(&crate_docs, term, "crate rustdoc");
    }
    for term in &manifest.required_readme_terms {
        assert_contains(&readme, term, "README");
    }

    assert_family_sources_cover_recipe_manifest(&manifest, &recipe_sources);

    for family in &manifest.component_families {
        assert_contains(&crate_docs, &family.heading, "crate rustdoc family heading");
        assert_contains(&readme, &family.heading, "README family heading");

        for term in &family.doc_terms {
            assert_contains(&crate_docs, term, "crate rustdoc family term");
            assert_contains(&readme, term, "README family term");
        }

        for export in &family.exports {
            assert_contains(reexports, export, "public re-export block");
        }

        for source_id in &family.source_ids {
            let source = recipe_source_path(&recipe_sources, source_id);
            let source_text = read_source(&crate_dir.join(source));
            assert!(
                source_text.trim_start().starts_with("//!"),
                "{source_id} should keep module-level rustdoc"
            );
        }
    }

    for source_id in rendered_recipe_sources(&manifest, &recipe_sources) {
        let source = recipe_source_path(&recipe_sources, &source_id);
        let source_text = read_source(&crate_dir.join(source));
        assert!(
            source_text.contains("pub fn test_id"),
            "{source_id} should expose a stable `.test_id(...)` authoring surface"
        );
    }

    for style in &manifest.style_surfaces {
        assert_contains(
            reexports,
            &style.style_export,
            "public style re-export block",
        );

        let source = recipe_source_path(&recipe_sources, &style.source_id);
        let source_text = read_source(&crate_dir.join(source));
        assert!(
            source_text.contains(&format!("pub struct {}", style.style_export)),
            "{} should declare public {}",
            style.source_id,
            style.style_export
        );
        let style_builder = format!(
            "pub fn style(mut self, style: {}) -> Self",
            style.style_export
        );
        assert!(
            normalize_ws(&source_text).contains(&normalize_ws(&style_builder)),
            "{} should expose `.style({})`",
            style.source_id,
            style.style_export
        );
    }

    for source_id in &manifest.copyable_state_sources {
        let source = recipe_source_path(&recipe_sources, source_id);
        let source_text = read_source(&crate_dir.join(source));
        assert!(
            source_text.contains("pub fn new_controllable"),
            "{source_id} should expose `new_controllable(cx, ...)` for copyable state"
        );
        assert!(
            source_text.contains("pub fn uncontrolled"),
            "{source_id} should expose `uncontrolled(cx)` for copyable state"
        );
        assert!(
            source_text.contains("_model(&self)"),
            "{source_id} should expose a resolved `*_model()` accessor"
        );
    }
}

fn read_source(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("{} must be readable: {err}", path.display()))
}

fn leading_crate_docs(lib_source: &str) -> String {
    let mut docs = Vec::new();
    for line in lib_source.lines() {
        if line.starts_with("//!") || line.trim().is_empty() {
            docs.push(line);
        } else {
            break;
        }
    }
    docs.join("\n")
}

fn public_reexport_block(lib_source: &str) -> &str {
    let start = lib_source
        .find("pub use autocomplete::")
        .expect("lib.rs should keep public recipe re-exports");
    let end = lib_source
        .find("pub mod context")
        .expect("lib.rs should declare context after public recipe re-exports");
    &lib_source[start..end]
}

fn recipe_sources_by_id(manifest: &RecipeProofManifestV1) -> BTreeMap<String, String> {
    manifest
        .entries
        .iter()
        .map(|entry| (entry.id.clone(), entry.source.clone()))
        .collect()
}

fn recipe_source_path<'a>(
    recipe_sources: &'a BTreeMap<String, String>,
    source_id: &str,
) -> &'a str {
    recipe_sources
        .get(source_id)
        .map(String::as_str)
        .unwrap_or_else(|| panic!("{source_id} must exist in material3 recipe proof manifest"))
}

fn rendered_recipe_sources(
    manifest: &ApiDocSurfaceManifestV1,
    recipe_sources: &BTreeMap<String, String>,
) -> BTreeSet<String> {
    recipe_sources
        .keys()
        .filter(|id| !manifest.supporting_api_sources.contains(id))
        .cloned()
        .collect()
}

fn assert_family_sources_cover_recipe_manifest(
    manifest: &ApiDocSurfaceManifestV1,
    recipe_sources: &BTreeMap<String, String>,
) {
    let covered_sources: BTreeSet<_> = manifest
        .component_families
        .iter()
        .flat_map(|family| family.source_ids.iter().cloned())
        .chain(manifest.supporting_api_sources.iter().cloned())
        .collect();
    let recipe_sources: BTreeSet<_> = recipe_sources.keys().cloned().collect();

    assert_eq!(
        covered_sources, recipe_sources,
        "Material3 API doc families plus supporting API sources must cover the public recipe proof manifest"
    );
}

fn assert_sorted_unique_by_id(families: &[ComponentFamilyV1]) {
    for pair in families.windows(2) {
        assert!(
            pair[0].id.as_str() < pair[1].id.as_str(),
            "component families must be sorted by id and unique; found {} then {}",
            pair[0].id,
            pair[1].id
        );
    }
}

fn assert_sorted_unique_strings(label: &str, values: &[String]) {
    for pair in values.windows(2) {
        assert!(
            pair[0].as_str() < pair[1].as_str(),
            "{label} must be sorted and unique; found {} then {}",
            pair[0],
            pair[1]
        );
    }
}

fn assert_contains(haystack: &str, needle: &str, label: &str) {
    assert!(
        haystack.contains(needle),
        "{label} should contain `{needle}`"
    );
}

fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}
