//! Material token coverage audit helpers used by crate-level conformance tests.

use std::collections::BTreeSet;
use std::path::Path;

use fret_ui::Theme;
use serde::Deserialize;

const TOKEN_USAGE_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_token_usage_manifest_v1.json"
));

const ROOT_SOURCE_EXCLUDES: &[&str] = &["lib.rs"];
const TOKEN_SOURCE_EXCLUDES: &[&str] = &[
    "coverage.rs",
    "material_web_v30.rs",
    "v30.rs",
    "visual_fixture_model.rs",
    "visual_fixtures.rs",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MaterialTokenUse {
    pub(crate) source: String,
    pub(crate) key: String,
}

pub(crate) fn literal_md_token_uses() -> Vec<MaterialTokenUse> {
    let mut uses = Vec::new();
    for source in load_manifest().sources {
        uses.extend(source.tokens.into_iter().map(|key| MaterialTokenUse {
            source: source.path.clone(),
            key,
        }));
    }
    uses.sort_unstable_by(|a, b| a.key.cmp(&b.key).then_with(|| a.source.cmp(&b.source)));
    uses.dedup_by(|a, b| a.key == b.key);
    uses
}

pub(crate) fn token_resolves(theme: &Theme, key: &str) -> bool {
    token_resolution_kind(theme, key).is_some()
}

pub(crate) fn validate_manifest_against_sources() -> Result<(), String> {
    let manifest = load_manifest();
    let mut errors = Vec::new();

    if manifest.schema_version != 1 {
        errors.push(format!(
            "unsupported material token usage manifest schema_version {}",
            manifest.schema_version
        ));
    }

    match discover_audited_source_paths() {
        Ok(discovered_paths) => {
            let manifest_paths: BTreeSet<_> = manifest
                .sources
                .iter()
                .map(|source| source.path.clone())
                .collect();
            for path in discovered_paths.difference(&manifest_paths) {
                errors.push(format!("manifest missing audited source file {path}"));
            }
            for path in manifest_paths.difference(&discovered_paths) {
                errors.push(format!(
                    "manifest references non-audited source file {path}"
                ));
            }
        }
        Err(err) => errors.push(err),
    }

    let mut ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for source in &manifest.sources {
        if !ids.insert(source.id.as_str()) {
            errors.push(format!("duplicate material token source id {}", source.id));
        }
        if !paths.insert(source.path.as_str()) {
            errors.push(format!(
                "duplicate material token source path {}",
                source.path
            ));
        }
        if source.layer != expected_source_layer(&source.path) {
            errors.push(format!(
                "{}: layer {:?} does not match source path",
                source.path, source.layer
            ));
        }
        validate_sorted_unique_tokens(source, &mut errors);

        match read_manifest_source(&source.path) {
            Ok(text) => {
                let actual: BTreeSet<_> = extract_md_literal_keys(&text).into_iter().collect();
                let expected: BTreeSet<_> = source.tokens.iter().cloned().collect();

                for key in actual.difference(&expected) {
                    errors.push(format!(
                        "{}: manifest missing literal token {key}",
                        source.path
                    ));
                }
                for key in expected.difference(&actual) {
                    errors.push(format!("{}: stale manifest token {key}", source.path));
                }
            }
            Err(err) => errors.push(err),
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

#[derive(Debug, Deserialize)]
struct MaterialTokenManifest {
    schema_version: u32,
    sources: Vec<MaterialTokenSource>,
}

#[derive(Debug, Deserialize)]
struct MaterialTokenSource {
    id: String,
    layer: MaterialTokenSourceLayer,
    path: String,
    tokens: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum MaterialTokenSourceLayer {
    Foundation,
    Recipe,
    TokenModule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenResolutionKind {
    Color,
    Metric,
    Number,
    Duration,
    Easing,
    Corners,
    TextStyle,
}

fn load_manifest() -> MaterialTokenManifest {
    serde_json::from_str(TOKEN_USAGE_MANIFEST).expect("material token usage manifest must parse")
}

fn expected_source_layer(path: &str) -> MaterialTokenSourceLayer {
    if path.starts_with("src/foundation/") || path.starts_with("src/interaction/") {
        MaterialTokenSourceLayer::Foundation
    } else if path.starts_with("src/tokens/") {
        MaterialTokenSourceLayer::TokenModule
    } else {
        MaterialTokenSourceLayer::Recipe
    }
}

fn token_resolution_kind(theme: &Theme, key: &str) -> Option<TokenResolutionKind> {
    if theme.color_by_key(key).is_some() {
        Some(TokenResolutionKind::Color)
    } else if theme.metric_by_key(key).is_some() {
        Some(TokenResolutionKind::Metric)
    } else if theme.number_by_key(key).is_some() {
        Some(TokenResolutionKind::Number)
    } else if theme.duration_ms_by_key(key).is_some() {
        Some(TokenResolutionKind::Duration)
    } else if theme.easing_by_key(key).is_some() {
        Some(TokenResolutionKind::Easing)
    } else if theme.corners_by_key(key).is_some() {
        Some(TokenResolutionKind::Corners)
    } else if theme.text_style_by_key(key).is_some() {
        Some(TokenResolutionKind::TextStyle)
    } else {
        None
    }
}

fn validate_sorted_unique_tokens(source: &MaterialTokenSource, errors: &mut Vec<String>) {
    for pair in source.tokens.windows(2) {
        if pair[0] == pair[1] {
            errors.push(format!(
                "{}: duplicate manifest token {}",
                source.path, pair[0]
            ));
        } else if pair[0] > pair[1] {
            errors.push(format!(
                "{}: manifest tokens must be sorted; {} appears before {}",
                source.path, pair[0], pair[1]
            ));
        }
    }
}

fn discover_audited_source_paths() -> Result<BTreeSet<String>, String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut paths = BTreeSet::new();
    collect_rs_files(root, "src", ROOT_SOURCE_EXCLUDES, &mut paths)?;
    collect_rs_files(root, "src/foundation", &[], &mut paths)?;
    collect_rs_files(root, "src/interaction", &[], &mut paths)?;
    collect_rs_files(root, "src/tokens", TOKEN_SOURCE_EXCLUDES, &mut paths)?;
    Ok(paths)
}

fn collect_rs_files(
    root: &Path,
    relative_dir: &str,
    excluded_names: &[&str],
    paths: &mut BTreeSet<String>,
) -> Result<(), String> {
    let dir = root.join(relative_dir);
    let entries = std::fs::read_dir(&dir).map_err(|err| {
        format!(
            "{}: failed to read audited source dir: {err}",
            dir.display()
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|err| {
            format!(
                "{}: failed to read audited source entry: {err}",
                dir.display()
            )
        })?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if excluded_names.contains(&name) {
            continue;
        }
        paths.insert(format!("{relative_dir}/{name}"));
    }
    Ok(())
}

fn read_manifest_source(path: &str) -> Result<String, String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let full_path = root.join(path);
    std::fs::read_to_string(&full_path).map_err(|err| {
        format!(
            "{}: failed to read manifest source: {err}",
            full_path.display()
        )
    })
}

fn extract_md_literal_keys(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cursor: usize = 0;
    while let Some(idx) = source[cursor..].find("\"md.") {
        let start = cursor + idx + 1;
        let rest = &source[start..];
        let Some(end) = rest.find('"') else {
            break;
        };
        let key = &source[start..start + end];
        cursor = start + end + 1;
        if key.contains('{')
            || key.contains('}')
            || key.contains(' ')
            || key.contains('\n')
            || is_internal_test_token(key)
        {
            continue;
        }
        // Skip namespace/prefix strings like `md.comp.button` / `md.comp.checkbox.selected`
        // that are used to build other keys.
        // - `md.sys.*` tokens can be as short as `md.sys.color.primary` (3 dots).
        // - `md.comp.*` tokens are always deeper (at least 4 dots).
        let dot_count = key.matches('.').count();
        if key.starts_with("md.comp.") {
            if dot_count < 4 {
                continue;
            }
        } else if dot_count < 3 {
            continue;
        }
        out.push(key.to_string());
    }
    out.sort_unstable();
    out.dedup();
    out
}

fn is_internal_test_token(key: &str) -> bool {
    key.contains(".test-") || key.contains("-test-")
}
