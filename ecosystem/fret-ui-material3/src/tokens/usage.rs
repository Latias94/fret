//! Shared Material token usage discovery for conformance tests and maintainer tools.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use serde::Deserialize;

const ROOT_SOURCE_EXCLUDES: &[&str] = &["lib.rs"];
const FOUNDATION_SOURCE_EXCLUDES: &[&str] =
    &["field_overlay.rs", "motion_roles.rs", "style_overrides.rs"];
const TOKEN_SOURCE_EXCLUDES: &[&str] = &[
    "coverage.rs",
    "material_web_v30.rs",
    "usage.rs",
    "v30.rs",
    "v30_overlay.rs",
    "v30_overlay_metadata.rs",
    "visual_fixture_model.rs",
    "visual_fixtures.rs",
];
pub const MATERIAL_TOKEN_USAGE_MANIFEST_SUITE: &str = "material3-token-usage-manifest-v1";
pub const MATERIAL_TOKEN_USAGE_MANIFEST_NOTES: &str = "Structured manifest for literal Material token uses in Material3 recipes, foundations, and token modules. It is checked against source drift and v30 theme resolution by tokens::coverage; crate-internal *test* token namespaces are intentionally excluded.";

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum MaterialTokenSourceLayer {
    Foundation,
    Recipe,
    TokenModule,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MaterialTokenSource {
    pub id: String,
    pub layer: MaterialTokenSourceLayer,
    pub path: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MaterialTokenLiteralScan {
    pub exact: BTreeSet<String>,
    pub templates: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialTokenSourceScan {
    pub source: MaterialTokenSource,
    pub literals: MaterialTokenLiteralScan,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MaterialTokenUsageScan {
    pub sources: Vec<MaterialTokenSourceScan>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MaterialTokenTemplateExpansion {
    pub expanded: BTreeSet<String>,
    pub unexpanded: BTreeSet<String>,
}

impl MaterialTokenSourceLayer {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Foundation => "foundation",
            Self::Recipe => "recipe",
            Self::TokenModule => "token_module",
        }
    }
}

impl MaterialTokenUsageScan {
    pub fn exact_keys(&self) -> BTreeSet<String> {
        self.sources
            .iter()
            .flat_map(|source| source.literals.exact.iter().cloned())
            .collect()
    }

    pub fn template_keys(&self) -> BTreeSet<String> {
        self.sources
            .iter()
            .flat_map(|source| source.literals.templates.iter().cloned())
            .collect()
    }
}

pub fn discover_audited_sources(crate_dir: &Path) -> Result<Vec<MaterialTokenSource>, String> {
    let mut sources = Vec::new();
    for path in collect_rs_files(crate_dir, "src", ROOT_SOURCE_EXCLUDES)? {
        sources.push(MaterialTokenSource {
            id: source_id_from_path(&path),
            layer: expected_source_layer(&path),
            path,
        });
    }
    for path in collect_rs_files(crate_dir, "src/foundation", FOUNDATION_SOURCE_EXCLUDES)? {
        sources.push(MaterialTokenSource {
            id: source_id_from_path(&path),
            layer: expected_source_layer(&path),
            path,
        });
    }
    for path in collect_rs_files(crate_dir, "src/interaction", &[])? {
        sources.push(MaterialTokenSource {
            id: source_id_from_path(&path),
            layer: expected_source_layer(&path),
            path,
        });
    }
    for path in collect_rs_files(crate_dir, "src/tokens", TOKEN_SOURCE_EXCLUDES)? {
        sources.push(MaterialTokenSource {
            id: source_id_from_path(&path),
            layer: expected_source_layer(&path),
            path,
        });
    }
    Ok(sources)
}

pub fn scan_audited_sources(crate_dir: &Path) -> Result<MaterialTokenUsageScan, String> {
    let sources = discover_audited_sources(crate_dir)?;
    let mut scans = Vec::with_capacity(sources.len());
    for source in sources {
        let text = read_source_text(crate_dir, &source.path)?;
        scans.push(MaterialTokenSourceScan {
            source,
            literals: scan_md_string_literals(&text),
        });
    }
    Ok(MaterialTokenUsageScan { sources: scans })
}

pub fn read_source_text(crate_dir: &Path, path: &str) -> Result<String, String> {
    let full_path = crate_dir.join(path);
    fs::read_to_string(&full_path)
        .map_err(|err| format!("{}: failed to read source file: {err}", full_path.display()))
}

pub fn expected_source_layer(path: &str) -> MaterialTokenSourceLayer {
    if path.starts_with("src/foundation/") || path.starts_with("src/interaction/") {
        MaterialTokenSourceLayer::Foundation
    } else if path.starts_with("src/tokens/") {
        MaterialTokenSourceLayer::TokenModule
    } else {
        MaterialTokenSourceLayer::Recipe
    }
}

pub fn source_id_from_path(path: &str) -> String {
    let id = path.strip_prefix("src/").unwrap_or(path);
    id.strip_suffix(".rs")
        .unwrap_or(id)
        .replace(['/', '\\'], ".")
}

pub fn scan_md_string_literals(source: &str) -> MaterialTokenLiteralScan {
    let mut scan = MaterialTokenLiteralScan::default();
    let mut cursor: usize = 0;
    while let Some(idx) = source[cursor..].find("\"md.") {
        let start = cursor + idx + 1;
        let rest = &source[start..];
        let Some(end) = rest.find('"') else {
            break;
        };
        let key = &source[start..start + end];
        cursor = start + end + 1;

        if should_skip_md_literal(key) {
            continue;
        }

        if key.contains('{') || key.contains('}') {
            scan.templates.insert(key.to_string());
        } else {
            scan.exact.insert(key.to_string());
        }
    }
    scan
}

pub fn expand_key_templates(
    crate_dir: &Path,
    templates: &BTreeSet<String>,
) -> MaterialTokenTemplateExpansion {
    let mut expansion = MaterialTokenTemplateExpansion::default();

    for template in templates {
        if let Some(keys) = expand_key_template(crate_dir, template) {
            expansion.expanded.extend(keys);
        } else {
            expansion.unexpanded.insert(template.clone());
        }
    }

    expansion
}

pub fn usage_manifest_json(scan: &MaterialTokenUsageScan, suite: &str, notes: &str) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"schema_version\": 1,\n");
    out.push_str("  \"suite\": ");
    push_json_string(&mut out, suite);
    out.push_str(",\n");
    out.push_str("  \"notes\": ");
    push_json_string(&mut out, notes);
    out.push_str(",\n");
    out.push_str("  \"sources\": [\n");

    for (source_idx, source) in scan.sources.iter().enumerate() {
        out.push_str("    {\n");
        out.push_str("      \"id\": ");
        push_json_string(&mut out, &source.source.id);
        out.push_str(",\n");
        out.push_str("      \"layer\": ");
        push_json_string(&mut out, source.source.layer.as_str());
        out.push_str(",\n");
        out.push_str("      \"path\": ");
        push_json_string(&mut out, &source.source.path);
        out.push_str(",\n");
        out.push_str("      \"tokens\": ");
        push_json_string_array(&mut out, &source.literals.exact, 6);
        out.push('\n');
        out.push_str("    }");
        if source_idx + 1 != scan.sources.len() {
            out.push(',');
        }
        out.push('\n');
    }

    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

pub fn default_usage_manifest_json(scan: &MaterialTokenUsageScan) -> String {
    usage_manifest_json(
        scan,
        MATERIAL_TOKEN_USAGE_MANIFEST_SUITE,
        MATERIAL_TOKEN_USAGE_MANIFEST_NOTES,
    )
}

fn collect_rs_files(
    root: &Path,
    relative_dir: &str,
    excluded_names: &[&str],
) -> Result<Vec<String>, String> {
    let dir = root.join(relative_dir);
    let mut paths = BTreeSet::new();
    let entries = fs::read_dir(&dir).map_err(|err| {
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
    Ok(paths.into_iter().collect())
}

fn should_skip_md_literal(key: &str) -> bool {
    if !key.starts_with("md.")
        || key.contains(' ')
        || key.contains('\n')
        || is_internal_test_token(key)
    {
        return true;
    }

    // Skip namespace/prefix strings like `md.comp.button` / `md.comp.checkbox.selected`
    // that are used to build other keys.
    // - `md.sys.*` tokens can be as short as `md.sys.color.primary` (3 dots).
    // - `md.comp.*` tokens are always deeper (at least 4 dots).
    let dot_count = key.matches('.').count();
    if key.starts_with("md.comp.") {
        dot_count < 4
    } else {
        dot_count < 3
    }
}

fn is_internal_test_token(key: &str) -> bool {
    key.contains(".test-")
        || key.contains("-test-")
        || key.starts_with("md.comp.test.")
        || key.starts_with("md.sys.test.")
}

fn push_json_string_array(out: &mut String, values: &BTreeSet<String>, indent: usize) {
    if values.is_empty() {
        out.push_str("[]");
        return;
    }

    out.push_str("[\n");
    let pad = " ".repeat(indent + 2);
    for (idx, value) in values.iter().enumerate() {
        out.push_str(&pad);
        push_json_string(out, value);
        if idx + 1 != values.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str(&" ".repeat(indent));
    out.push(']');
}

fn push_json_string(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => {
                write!(out, "\\u{:04x}", ch as u32).ok();
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
}

fn expand_key_template(crate_dir: &Path, template: &str) -> Option<BTreeSet<String>> {
    if template.starts_with("md.comp.button.") {
        return expand_button_template(template);
    }
    if template.starts_with("md.comp.icon-button.") {
        return expand_icon_button_template(template);
    }
    if template.starts_with("md.comp.radio-button.") {
        return expand_radio_button_template(template);
    }
    if template.starts_with("md.comp.outlined-segmented-button.") {
        return expand_outlined_segmented_button_template(template);
    }
    if template.starts_with("md.comp.switch.") {
        return expand_switch_template(template);
    }
    if template.starts_with("md.comp.date-picker.docked.")
        || template.starts_with("md.comp.date-picker.modal.")
    {
        return expand_date_picker_template(crate_dir, template);
    }
    if template.starts_with("md.sys.typescale.") {
        return expand_typescale_template(template);
    }

    None
}

fn ensure_no_template_braces(keys: &BTreeSet<String>) -> Option<BTreeSet<String>> {
    keys.iter()
        .all(|key| !key.contains('{') && !key.contains('}'))
        .then_some(keys.clone())
}

fn expand_placeholder(
    keys: &BTreeSet<String>,
    placeholder: &str,
    values: &[&'static str],
) -> BTreeSet<String> {
    if !keys.iter().any(|key| key.contains(placeholder)) {
        return keys.clone();
    }

    let mut out = BTreeSet::new();
    for key in keys {
        if key.contains(placeholder) {
            for value in values {
                out.insert(key.replace(placeholder, value));
            }
        } else {
            out.insert(key.clone());
        }
    }
    out
}

fn expand_placeholder_dynamic(
    keys: &BTreeSet<String>,
    placeholder: &str,
    values: &[String],
) -> BTreeSet<String> {
    if !keys.iter().any(|key| key.contains(placeholder)) {
        return keys.clone();
    }

    let mut out = BTreeSet::new();
    for key in keys {
        if key.contains(placeholder) {
            for value in values {
                out.insert(key.replace(placeholder, value));
            }
        } else {
            out.insert(key.clone());
        }
    }
    out
}

fn expand_button_template(template: &str) -> Option<BTreeSet<String>> {
    const VARIANTS: &[&str] = &["filled", "tonal", "elevated", "outlined", "text"];
    const SUFFIXES: &[&str] = &[
        "hovered.state-layer.color",
        "focused.state-layer.color",
        "pressed.state-layer.color",
    ];

    let mut keys = BTreeSet::from([template.to_string()]);
    keys = expand_placeholder(&keys, "{variant_key}", VARIANTS);
    keys = expand_placeholder(&keys, "{}", VARIANTS);
    keys = expand_placeholder(&keys, "{suffix}", SUFFIXES);

    ensure_no_template_braces(&keys)
}

fn expand_outlined_segmented_button_template(template: &str) -> Option<BTreeSet<String>> {
    const BASES: &[&str] = &["selected", "unselected"];

    let mut keys = BTreeSet::from([template.to_string()]);
    keys = expand_placeholder(&keys, "{base}", BASES);

    ensure_no_template_braces(&keys)
}

fn expand_date_picker_template(crate_dir: &Path, template: &str) -> Option<BTreeSet<String>> {
    let suffixes = date_picker_suffixes_from_source(crate_dir)?;

    let mut keys = BTreeSet::from([template.to_string()]);
    keys = expand_placeholder_dynamic(&keys, "{suffix}", &suffixes);

    ensure_no_template_braces(&keys)
}

fn date_picker_suffixes_from_source(crate_dir: &Path) -> Option<Vec<String>> {
    let path = crate_dir.join("src").join("tokens").join("date_picker.rs");
    let content = fs::read_to_string(path).ok()?;

    let mut suffixes: BTreeSet<String> = BTreeSet::new();
    let mut remaining = content.as_str();
    while let Some(pos) = remaining.find("token_key(variant,") {
        remaining = &remaining[pos..];
        let Some(start_quote) = remaining.find('"') else {
            break;
        };
        let after_quote = &remaining[start_quote + 1..];
        let Some(end_quote) = after_quote.find('"') else {
            break;
        };
        let literal = &after_quote[..end_quote];
        suffixes.insert(literal.to_string());
        remaining = &after_quote[end_quote + 1..];
    }

    // `token_key(variant, "...")` always uses a string literal suffix in this module.
    // If that ever changes, prefer making the suffixes explicit rather than weakening `--check`.
    (!suffixes.is_empty()).then_some(suffixes.into_iter().collect())
}

fn expand_icon_button_template(template: &str) -> Option<BTreeSet<String>> {
    const VARIANTS: &[&str] = &["standard", "filled", "tonal", "outlined"];
    const STATE_DOTTED: &[&str] = &["hovered.", "focused.", "pressed."];
    const STATE_TRIMMED: &[&str] = &["hovered", "focused", "pressed"];

    let mut keys = BTreeSet::from([template.to_string()]);

    keys = expand_placeholder(&keys, "{variant_key}", VARIANTS);
    keys = expand_icon_button_variant_slot(&keys, VARIANTS);
    keys = expand_icon_button_select_prefix(&keys);
    keys = expand_placeholder(&keys, "{prefix}", &["", "selected."]);
    keys = expand_placeholder(&keys, "{state}", STATE_DOTTED);
    keys = expand_placeholder(&keys, "{}", STATE_TRIMMED);

    ensure_no_template_braces(&keys)
}

fn expand_icon_button_variant_slot(
    keys: &BTreeSet<String>,
    variants: &[&'static str],
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for key in keys {
        if key.starts_with("md.comp.icon-button.{}.") {
            for variant in variants {
                out.insert(key.replacen(
                    "md.comp.icon-button.{}.",
                    &format!("md.comp.icon-button.{variant}."),
                    1,
                ));
            }
        } else {
            out.insert(key.clone());
        }
    }
    out
}

fn expand_icon_button_select_prefix(keys: &BTreeSet<String>) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for key in keys {
        if !key.contains("{select_prefix}") {
            out.insert(key.clone());
            continue;
        }

        let Some(variant) = icon_button_variant_from_key(key) else {
            out.insert(key.clone());
            continue;
        };

        for prefix in icon_button_select_prefixes(variant) {
            out.insert(key.replace("{select_prefix}", prefix));
        }
    }
    out
}

fn icon_button_variant_from_key(key: &str) -> Option<&str> {
    let rest = key.strip_prefix("md.comp.icon-button.")?;
    Some(rest.split('.').next().unwrap_or_default())
}

fn icon_button_select_prefixes(variant: &str) -> &'static [&'static str] {
    match variant {
        // Standard: base tokens are unselected; selected uses a distinct prefix.
        "standard" => &["", "selected."],
        // Filled: base tokens are the selected look; unselected uses a distinct prefix.
        "filled" => &["", "unselected."],
        // Tonal: base tokens are unselected; selected uses a distinct prefix.
        "tonal" => &["", "selected."],
        // Outlined: base tokens are unselected; selected uses a distinct prefix.
        "outlined" => &["", "selected."],
        _ => &[""],
    }
}

fn expand_radio_button_template(template: &str) -> Option<BTreeSet<String>> {
    const GROUPS: &[&str] = &["selected", "unselected"];

    let mut keys = BTreeSet::from([template.to_string()]);
    keys = expand_placeholder(&keys, "{group}", GROUPS);

    ensure_no_template_braces(&keys)
}

fn expand_switch_template(template: &str) -> Option<BTreeSet<String>> {
    const GROUPS: &[&str] = &["selected", "unselected"];
    const STATES: &[&str] = &["hover", "focus", "pressed"];

    let mut keys = BTreeSet::from([template.to_string()]);
    keys = expand_placeholder(&keys, "{group}", GROUPS);
    keys = expand_placeholder(&keys, "{state}", STATES);

    ensure_no_template_braces(&keys)
}

fn expand_typescale_template(template: &str) -> Option<BTreeSet<String>> {
    const NAMES: &[&str] = &[
        "display-large",
        "display-medium",
        "display-small",
        "headline-large",
        "headline-medium",
        "headline-small",
        "title-large",
        "title-medium",
        "title-small",
        "body-large",
        "body-medium",
        "body-small",
        "label-large",
        "label-medium",
        "label-small",
    ];

    let mut keys = BTreeSet::from([template.to_string()]);
    keys = expand_placeholder(&keys, "{name}", NAMES);

    ensure_no_template_braces(&keys)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_ids_match_manifest_shape() {
        assert_eq!(source_id_from_path("src/button.rs"), "button");
        assert_eq!(
            source_id_from_path("src/foundation/context.rs"),
            "foundation.context"
        );
        assert_eq!(
            source_id_from_path("src/tokens/navigation_common.rs"),
            "tokens.navigation_common"
        );
    }

    #[test]
    fn md_literal_scan_filters_prefixes_and_internal_test_tokens() {
        let scan = scan_md_string_literals(
            r#"
            let prefix = "md.comp.button";
            let sys_prefix = "md.sys.color";
            let exact = "md.comp.button.filled.container.color";
            let sys = "md.sys.color.primary";
            let template = "md.comp.button.{variant_key}.{suffix}";
            let internal = "md.comp.outlined-test-field.container.height";
            let internal_namespace = "md.comp.test.focus.state-layer.opacity";
            "#,
        );

        assert_eq!(
            scan.exact,
            BTreeSet::from([
                "md.comp.button.filled.container.color".to_string(),
                "md.sys.color.primary".to_string(),
            ])
        );
        assert_eq!(
            scan.templates,
            BTreeSet::from(["md.comp.button.{variant_key}.{suffix}".to_string()])
        );
    }

    #[test]
    fn generated_manifest_matches_checked_fixture() {
        let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let scan = scan_audited_sources(crate_dir).expect("material token usage scan must succeed");
        let generated = default_usage_manifest_json(&scan);
        let checked = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/material3_token_usage_manifest_v1.json"
        ));

        assert_eq!(generated, checked.replace("\r\n", "\n"));
    }
}
