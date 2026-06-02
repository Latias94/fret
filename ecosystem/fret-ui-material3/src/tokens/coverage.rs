//! Material token coverage audit helpers used by crate-level conformance tests.

use std::collections::{BTreeMap, BTreeSet};

use fret_ui::Theme;
use serde::Deserialize;

use crate::tokens::usage::{self, MaterialTokenSourceLayer};

const TOKEN_USAGE_MANIFEST: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/material3_token_usage_manifest_v1.json"
));

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MaterialTokenUse {
    pub(crate) source: String,
    pub(crate) key: String,
}

pub(crate) fn literal_md_token_uses() -> Vec<MaterialTokenUse> {
    let mut uses = manifest_literal_md_token_uses();
    uses.extend(
        expanded_template_token_uses().expect("material token templates must expand for coverage"),
    );
    sort_and_dedup_token_uses(&mut uses);
    uses
}

fn manifest_literal_md_token_uses() -> Vec<MaterialTokenUse> {
    let mut uses = Vec::new();
    for source in load_manifest().sources {
        uses.extend(source.tokens.into_iter().map(|key| MaterialTokenUse {
            source: source.path.clone(),
            key,
        }));
    }
    uses
}

fn expanded_template_token_uses() -> Result<Vec<MaterialTokenUse>, String> {
    let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let scan = usage::scan_audited_sources(crate_dir)?;
    let mut uses = Vec::new();

    for source in scan.sources {
        for template in source.literals.templates {
            let expansion =
                usage::expand_key_templates(crate_dir, &BTreeSet::from([template.clone()]));
            if !expansion.unexpanded.is_empty() {
                return Err(format!(
                    "{}: unexpanded material token template {template}",
                    source.source.path
                ));
            }

            let source_label = format!("{} template {template}", source.source.path);
            uses.extend(expansion.expanded.into_iter().map(|key| MaterialTokenUse {
                source: source_label.clone(),
                key,
            }));
        }
    }

    Ok(uses)
}

fn sort_and_dedup_token_uses(uses: &mut Vec<MaterialTokenUse>) {
    uses.sort_unstable_by(|a, b| a.key.cmp(&b.key).then_with(|| a.source.cmp(&b.source)));
    uses.dedup_by(|a, b| a.key == b.key);
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
    if manifest.suite != usage::MATERIAL_TOKEN_USAGE_MANIFEST_SUITE {
        errors.push(format!(
            "material token usage manifest suite must be {}; found {}",
            usage::MATERIAL_TOKEN_USAGE_MANIFEST_SUITE,
            manifest.suite
        ));
    }
    if manifest.notes != usage::MATERIAL_TOKEN_USAGE_MANIFEST_NOTES {
        errors.push("material token usage manifest notes drifted from usage metadata".to_string());
    }

    let crate_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let audited_scan = match usage::scan_audited_sources(crate_dir) {
        Ok(scan) => Some(scan),
        Err(err) => {
            errors.push(err);
            None
        }
    };

    let actual_tokens_by_path: BTreeMap<_, _> = audited_scan
        .as_ref()
        .into_iter()
        .flat_map(|scan| {
            scan.sources
                .iter()
                .map(|source| (source.source.path.clone(), source.literals.exact.clone()))
        })
        .collect();

    if let Some(scan) = &audited_scan {
        let discovered_paths: BTreeSet<_> = scan
            .sources
            .iter()
            .map(|source| source.source.path.clone())
            .collect();
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
        if source.layer != usage::expected_source_layer(&source.path) {
            errors.push(format!(
                "{}: layer {:?} does not match source path",
                source.path, source.layer
            ));
        }
        validate_sorted_unique_tokens(source, &mut errors);

        if let Some(actual) = actual_tokens_by_path.get(&source.path) {
            let expected: BTreeSet<_> = source.tokens.iter().cloned().collect();

            for key in actual.difference(&expected) {
                errors.push(format!(
                    "{}: manifest missing literal token {key}",
                    source.path
                ));
            }
            for key in expected.difference(actual) {
                errors.push(format!("{}: stale manifest token {key}", source.path));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

pub(crate) fn validate_recipe_sources_are_token_free() -> Result<(), String> {
    let manifest = load_manifest();
    let errors: Vec<_> = manifest
        .sources
        .iter()
        .filter(|source| source.layer == MaterialTokenSourceLayer::Recipe)
        .filter(|source| !source.tokens.is_empty())
        .map(|source| {
            format!(
                "{}: recipe layer should delegate literal Material tokens to token modules; found {}",
                source.path,
                source.tokens.join(", ")
            )
        })
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("\n"))
    }
}

#[derive(Debug, Deserialize)]
struct MaterialTokenManifest {
    schema_version: u32,
    suite: String,
    notes: String,
    sources: Vec<MaterialTokenSource>,
}

#[derive(Debug, Deserialize)]
struct MaterialTokenSource {
    id: String,
    layer: MaterialTokenSourceLayer,
    path: String,
    tokens: Vec<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_md_token_uses_include_expanded_templates() {
        let uses = literal_md_token_uses();

        assert!(uses.iter().any(|token_use| {
            token_use.key == "md.comp.outlined-segmented-button.selected.hover.label-text.color"
                && token_use
                    .source
                    .contains("src/tokens/segmented_button.rs template")
        }));
    }
}
