use std::collections::HashMap;

use fret_syntax::HIGHLIGHT_NAMES;
use fret_ui::theme::ThemeConfig;

#[derive(Debug, thiserror::Error)]
pub enum VscodeThemeError {
    #[error("invalid VS Code theme json: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(default)]
pub struct VscodeImportMapping {
    pub version: u32,
    pub tokens: HashMap<String, MappingEntry>,
    pub highlights: HashMap<String, MappingEntry>,
}

impl Default for VscodeImportMapping {
    fn default() -> Self {
        Self {
            version: 1,
            tokens: HashMap::new(),
            highlights: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(default)]
pub struct MappingEntry {
    pub scopes: Vec<String>,
    pub foreground: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct VscodeTheme {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    colors: HashMap<String, String>,
    #[serde(default, rename = "tokenColors")]
    token_colors: Vec<serde_json::Value>,
}

#[derive(Debug, Default)]
struct TokenRule {
    scopes: Vec<String>,
    foreground: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct VscodeSyntaxImportOptions {
    /// Generate `color.syntax.<tag>` for every `fret-syntax` highlight tag.
    ///
    /// This is best-effort: tags without a matching VS Code scope remain unset and will fall back
    /// to Fret's built-in defaults (or whatever your base theme provides).
    pub generate_all_fret_syntax_tokens: bool,
}

impl Default for VscodeSyntaxImportOptions {
    fn default() -> Self {
        Self {
            generate_all_fret_syntax_tokens: false,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportMatch {
    pub rule_index: usize,
    pub selector: String,
    pub matched_scope: String,
    pub selector_specificity: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportDecision {
    pub token_key: String,
    pub highlight: Option<String>,
    pub candidate_scopes: Vec<String>,
    pub foreground: Option<String>,
    pub matched: Option<ImportMatch>,
    pub source: ImportDecisionSource,
}

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportDecisionSource {
    ThemeRule,
    MappingScopes,
    MappingForeground,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportReport {
    pub theme_name: Option<String>,
    pub applied_tokens: usize,
    pub matched_tokens: usize,
    pub decisions: Vec<ImportDecision>,
}

/// Build a `ThemeConfig` patch that populates `color.syntax.*` from a VS Code theme JSON.
///
/// By default this is intentionally conservative: it only tries to extract a small set of stable
/// syntax colors (comment/keyword/function/type/string/constant/number/operator/punctuation/variable).
pub fn syntax_theme_patch_from_vscode_json(bytes: &[u8]) -> Result<ThemeConfig, VscodeThemeError> {
    syntax_theme_patch_from_vscode_json_with_options(bytes, VscodeSyntaxImportOptions::default())
}

pub fn syntax_theme_patch_from_vscode_json_with_options(
    bytes: &[u8],
    options: VscodeSyntaxImportOptions,
) -> Result<ThemeConfig, VscodeThemeError> {
    Ok(syntax_theme_patch_and_report_from_vscode_json_with_options(bytes, options)?.0)
}

pub fn syntax_theme_patch_and_report_from_vscode_json_with_options(
    bytes: &[u8],
    options: VscodeSyntaxImportOptions,
) -> Result<(ThemeConfig, ImportReport), VscodeThemeError> {
    syntax_theme_patch_and_report_from_vscode_json_with_options_and_mapping(bytes, options, None)
}

pub fn syntax_theme_patch_and_report_from_vscode_json_with_options_and_mapping(
    bytes: &[u8],
    options: VscodeSyntaxImportOptions,
    mapping: Option<&VscodeImportMapping>,
) -> Result<(ThemeConfig, ImportReport), VscodeThemeError> {
    let theme: VscodeTheme = serde_json::from_slice(bytes)?;

    let rules = theme
        .token_colors
        .iter()
        .filter_map(parse_token_rule)
        .collect::<Vec<_>>();

    let mut colors = HashMap::<String, String>::new();
    let mut report = ImportReport {
        theme_name: theme.name.clone(),
        applied_tokens: 0,
        matched_tokens: 0,
        decisions: Vec::new(),
    };

    for (token_key, candidate_scopes) in SYNTAX_TOKEN_SCOPE_MAP {
        let default_scopes = candidate_scopes
            .iter()
            .map(|s| (*s).to_string())
            .collect::<Vec<_>>();
        let override_entry = mapping.and_then(|m| m.tokens.get(*token_key));
        let candidate_scopes_vec = override_entry
            .map(|e| {
                if e.scopes.is_empty() {
                    default_scopes.clone()
                } else {
                    e.scopes.clone()
                }
            })
            .unwrap_or_else(|| default_scopes.clone());

        let (foreground, matched, source) = if let Some(e) = override_entry
            && e.foreground.is_some()
        {
            (
                e.foreground.as_deref().and_then(normalize_color_value),
                None,
                ImportDecisionSource::MappingForeground,
            )
        } else {
            let scope_refs = candidate_scopes_vec
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>();
            let (foreground, matched) = pick_foreground_with_match(&rules, &scope_refs);
            let source = if override_entry.is_some() {
                ImportDecisionSource::MappingScopes
            } else {
                ImportDecisionSource::ThemeRule
            };
            (foreground, matched, source)
        };

        report.decisions.push(ImportDecision {
            token_key: (*token_key).to_string(),
            highlight: None,
            candidate_scopes: candidate_scopes_vec,
            foreground: foreground.clone(),
            matched,
            source,
        });

        report.applied_tokens += 1;
        if foreground.is_some() {
            report.matched_tokens += 1;
        }

        if let Some(fg) = foreground {
            colors.insert((*token_key).to_string(), fg);
        }
    }

    if options.generate_all_fret_syntax_tokens {
        for &highlight in HIGHLIGHT_NAMES {
            if highlight == "none" {
                continue;
            }

            let key = format!("color.syntax.{highlight}");
            if colors.contains_key(&key) {
                continue;
            }

            let override_entry = mapping.and_then(|m| m.highlights.get(highlight));
            let scopes = override_entry
                .map(|e| {
                    if e.scopes.is_empty() {
                        candidate_scopes_for_fret_highlight(highlight)
                    } else {
                        e.scopes.clone()
                    }
                })
                .unwrap_or_else(|| candidate_scopes_for_fret_highlight(highlight));
            let scope_refs = scopes.iter().map(|s| s.as_str()).collect::<Vec<_>>();
            let (foreground, matched, source) = if let Some(e) = override_entry
                && e.foreground.is_some()
            {
                (
                    e.foreground.as_deref().and_then(normalize_color_value),
                    None,
                    ImportDecisionSource::MappingForeground,
                )
            } else {
                let (foreground, matched) = pick_foreground_with_match(&rules, &scope_refs);
                let source = if override_entry.is_some() {
                    ImportDecisionSource::MappingScopes
                } else {
                    ImportDecisionSource::ThemeRule
                };
                (foreground, matched, source)
            };
            report.decisions.push(ImportDecision {
                token_key: key.clone(),
                highlight: Some(highlight.to_string()),
                candidate_scopes: scopes,
                foreground: foreground.clone(),
                matched,
                source,
            });

            report.applied_tokens += 1;
            if foreground.is_some() {
                report.matched_tokens += 1;
            }

            if let Some(fg) = foreground {
                colors.insert(key, fg);
            }
        }
    }

    Ok((
        ThemeConfig {
            name: format!(
                "vscode/{}",
                theme
                    .name
                    .as_deref()
                    .unwrap_or("theme")
                    .trim()
                    .replace(' ', "-")
                    .to_lowercase()
            ),
            author: Some("VS Code theme".to_string()),
            url: Some("https://code.visualstudio.com".to_string()),
            colors,
            ..ThemeConfig::default()
        },
        report,
    ))
}

const SYNTAX_TOKEN_SCOPE_MAP: &[(&str, &[&str])] = &[
    ("color.syntax.comment", &["comment"]),
    (
        "color.syntax.keyword",
        &["keyword", "storage", "storage.type", "storage.modifier"],
    ),
    (
        "color.syntax.function",
        &[
            "entity.name.function",
            "support.function",
            "variable.function",
        ],
    ),
    (
        "color.syntax.type",
        &[
            "entity.name.type",
            "entity.name.class",
            "entity.name.struct",
            "entity.name.enum",
            "support.type",
        ],
    ),
    ("color.syntax.string", &["string"]),
    (
        "color.syntax.constant",
        &["constant", "constant.language", "support.constant"],
    ),
    ("color.syntax.number", &["constant.numeric"]),
    // Some themes (e.g. HardHacker) don't define `keyword.operator` but do theme `punctuation`.
    (
        "color.syntax.operator",
        &["keyword.operator", "punctuation"],
    ),
    ("color.syntax.punctuation", &["punctuation"]),
    ("color.syntax.variable", &["variable", "variable.parameter"]),
];

fn parse_token_rule(v: &serde_json::Value) -> Option<TokenRule> {
    let obj = v.as_object()?;
    if obj.contains_key("include") {
        return None;
    }

    let scopes = match obj.get("scope") {
        None => Vec::new(),
        Some(serde_json::Value::String(s)) => split_scopes(s),
        Some(serde_json::Value::Array(a)) => a
            .iter()
            .filter_map(|v| v.as_str())
            .flat_map(split_scopes)
            .collect(),
        _ => Vec::new(),
    };

    let settings = obj.get("settings")?.as_object()?;
    let foreground = settings
        .get("foreground")
        .and_then(|v| v.as_str())
        .and_then(normalize_color_value);

    Some(TokenRule { scopes, foreground })
}

fn split_scopes(s: &str) -> Vec<String> {
    s.split(',')
        .flat_map(|piece| piece.split_whitespace())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

fn scope_matches(scope_selector: &str, want: &str) -> bool {
    scope_selector == want
        || (scope_selector.starts_with(want) && scope_selector[want.len()..].starts_with('.'))
}

fn pick_foreground_with_match(
    rules: &[TokenRule],
    candidate_scopes: &[&str],
) -> (Option<String>, Option<ImportMatch>) {
    let mut best: Option<(usize, usize, String, String, String)> = None;
    for (rule_index, rule) in rules.iter().enumerate() {
        let Some(fg) = rule.foreground.as_ref() else {
            continue;
        };

        for selector in &rule.scopes {
            for want in candidate_scopes {
                if !scope_matches(selector, want) {
                    continue;
                }
                let specificity = selector.split('.').count();
                match &best {
                    None => {
                        best = Some((
                            specificity,
                            rule_index,
                            fg.clone(),
                            selector.clone(),
                            (*want).to_string(),
                        ));
                    }
                    Some((best_spec, best_idx, _, _, _)) => {
                        if specificity > *best_spec
                            || (specificity == *best_spec && rule_index >= *best_idx)
                        {
                            best = Some((
                                specificity,
                                rule_index,
                                fg.clone(),
                                selector.clone(),
                                (*want).to_string(),
                            ));
                        }
                    }
                }
            }
        }
    }

    match best {
        Some((selector_specificity, rule_index, fg, selector, matched_scope)) => (
            Some(fg),
            Some(ImportMatch {
                rule_index,
                selector,
                matched_scope,
                selector_specificity,
            }),
        ),
        None => (None, None),
    }
}

fn normalize_color_value(raw: &str) -> Option<String> {
    let s = raw.trim();
    if s.is_empty() {
        return None;
    }
    if s.eq_ignore_ascii_case("transparent") {
        return Some("#00000000".to_string());
    }
    if let Some(hex) = normalize_hex_color(s) {
        return Some(hex);
    }
    if let Some(hex) = normalize_rgb_function(s) {
        return Some(hex);
    }
    Some(s.to_string())
}

fn normalize_hex_color(s: &str) -> Option<String> {
    let s = s.trim();
    let hex = s.strip_prefix('#')?;

    fn is_hex(s: &str) -> bool {
        s.bytes().all(|b| b.is_ascii_hexdigit())
    }

    match hex.len() {
        3 => {
            if !is_hex(hex) {
                return None;
            }
            let mut out = String::with_capacity(7);
            out.push('#');
            for ch in hex.chars() {
                out.push(ch);
                out.push(ch);
            }
            Some(out)
        }
        4 => {
            if !is_hex(hex) {
                return None;
            }
            let mut out = String::with_capacity(9);
            out.push('#');
            for ch in hex.chars() {
                out.push(ch);
                out.push(ch);
            }
            Some(out)
        }
        6 | 8 => {
            if !is_hex(hex) {
                return None;
            }
            Some(format!("#{hex}"))
        }
        _ => None,
    }
}

fn normalize_rgb_function(s: &str) -> Option<String> {
    let s = s.trim();
    let inner = if let Some(rest) = s.strip_prefix("rgb(") {
        rest.strip_suffix(')')?
    } else if let Some(rest) = s.strip_prefix("rgba(") {
        rest.strip_suffix(')')?
    } else {
        return None;
    };

    let parts = inner
        .split(',')
        .map(|p| p.trim())
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>();

    if parts.len() != 3 && parts.len() != 4 {
        return None;
    }

    fn parse_u8(s: &str) -> Option<u8> {
        let n: i32 = s.parse().ok()?;
        if !(0..=255).contains(&n) {
            return None;
        }
        Some(n as u8)
    }

    fn parse_alpha_u8(s: &str) -> Option<u8> {
        let s = s.trim();
        if let Some(pct) = s.strip_suffix('%') {
            let pct: f32 = pct.trim().parse().ok()?;
            let a = (pct / 100.0).clamp(0.0, 1.0);
            return Some(((a * 255.0).round() as i32).clamp(0, 255) as u8);
        }
        let a: f32 = s.parse().ok()?;
        let a = a.clamp(0.0, 1.0);
        Some(((a * 255.0).round() as i32).clamp(0, 255) as u8)
    }

    let r = parse_u8(parts[0])?;
    let g = parse_u8(parts[1])?;
    let b = parse_u8(parts[2])?;
    if parts.len() == 3 {
        return Some(format!("#{:02x}{:02x}{:02x}", r, g, b));
    }
    let a = parse_alpha_u8(parts[3])?;
    Some(format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a))
}

fn candidate_scopes_for_fret_highlight(highlight: &str) -> Vec<String> {
    fn v(scopes: &[&str]) -> Vec<String> {
        scopes.iter().map(|s| (*s).to_string()).collect()
    }

    let head = highlight.split('.').next().unwrap_or(highlight);
    match head {
        "attribute" => v(&["entity.other.attribute-name", "entity.other.attribute"]),
        "boolean" => v(&[
            "constant.language.boolean",
            "constant.boolean",
            "constant.language",
        ]),
        "character" => v(&["constant.character", "string"]),
        "comment" => {
            if highlight.contains("doc") || highlight.contains("documentation") {
                v(&[
                    "comment.block.documentation",
                    "comment.line.documentation",
                    "comment.documentation",
                    "comment",
                ])
            } else if highlight.contains("unused") {
                v(&["comment.unused", "comment"])
            } else {
                v(&["comment"])
            }
        }
        "constant" => {
            if highlight.contains("builtin") {
                v(&["constant.language", "support.constant", "constant"])
            } else if highlight.contains("macro") {
                v(&[
                    "entity.name.function.macro",
                    "support.function.macro",
                    "meta.macro",
                    "constant",
                ])
            } else {
                v(&["constant", "constant.language", "support.constant"])
            }
        }
        "constructor" => v(&[
            "entity.name.function.constructor",
            "support.function.constructor",
            "entity.name.type",
            "variable.function.constructor",
        ]),
        "diff" => match highlight {
            "diff.plus" => v(&["markup.inserted", "diff.inserted", "diff.plus"]),
            "diff.minus" => v(&["markup.deleted", "diff.deleted", "diff.minus"]),
            _ => v(&["diff"]),
        },
        "embedded" => v(&["meta.embedded", "string"]),
        "emphasis" => v(&["markup.italic", "emphasis"]),
        "enum" => v(&["entity.name.type.enum", "entity.name.type", "support.type"]),
        "error" => v(&["invalid.illegal", "invalid"]),
        "field" => v(&[
            "variable.other.member",
            "variable.other.property",
            "variable.other",
            "field",
        ]),
        "parameter" => v(&[
            "variable.parameter",
            "variable.parameter.builtin",
            "variable.other.readwrite.parameter",
            "parameter",
        ]),
        "float" => v(&["constant.numeric.float", "constant.numeric"]),
        "function" | "method" => {
            if highlight.contains("builtin") {
                v(&["support.function", "entity.name.function", "function"])
            } else if highlight.contains("macro") {
                v(&[
                    "entity.name.function.macro",
                    "support.function.macro",
                    "meta.macro",
                    "entity.name.function",
                ])
            } else if highlight.contains("method") {
                v(&[
                    "entity.name.function.method",
                    "entity.name.function",
                    "support.function",
                ])
            } else {
                v(&["entity.name.function", "support.function", "function"])
            }
        }
        "hint" => v(&["comment", "hint"]),
        "import" | "include" | "namespace" | "module" => v(&[
            "keyword.control.import",
            "keyword.control.include",
            "entity.name.namespace",
            "entity.name.module",
        ]),
        "keyword" => {
            if highlight.contains("operator") {
                v(&["keyword.operator", "keyword"])
            } else if highlight.contains("conditional") {
                v(&["keyword.control.conditional", "keyword.control", "keyword"])
            } else if highlight.contains("exception") {
                v(&["keyword.control.exception", "keyword.control", "keyword"])
            } else if highlight.contains("repeat") || highlight.contains("coroutine") {
                v(&["keyword.control.loop", "keyword.control", "keyword"])
            } else if highlight.contains("return") {
                v(&["keyword.control.return", "keyword.control", "keyword"])
            } else if highlight.contains("type") {
                v(&["storage.type", "keyword", "storage"])
            } else if highlight.contains("modifier") {
                v(&["storage.modifier", "keyword", "storage"])
            } else if highlight.contains("directive") {
                v(&[
                    "keyword.other.directive",
                    "keyword.control.directive",
                    "keyword",
                ])
            } else if highlight.contains("import") {
                v(&["keyword.control.import", "keyword.control", "keyword"])
            } else {
                v(&["keyword", "storage", "storage.type", "storage.modifier"])
            }
        }
        "label" => v(&["entity.name.label", "label"]),
        "link_text" => v(&["markup.underline.link", "string.other.link", "link"]),
        "link_uri" => v(&["markup.underline.link", "string.other.link", "link"]),
        "number" => v(&["constant.numeric", "number"]),
        "operator" => v(&["keyword.operator", "punctuation", "operator", "keyword"]),
        "preproc" => v(&[
            "keyword.control.directive",
            "meta.preprocessor",
            "entity.name.function.preprocessor",
        ]),
        "primary" => v(&["variable.language", "constant.language", "primary"]),
        "property" => v(&[
            "variable.other.property",
            "support.type.property-name",
            "property",
        ]),
        "punctuation" => {
            if highlight.contains("bracket") {
                v(&["punctuation.section", "punctuation"])
            } else if highlight.contains("delimiter") {
                v(&["punctuation.separator", "punctuation"])
            } else if highlight.contains("list_marker") {
                v(&["punctuation.definition.list", "punctuation"])
            } else if highlight.contains("special") {
                v(&["punctuation.definition", "punctuation"])
            } else {
                v(&["punctuation"])
            }
        }
        "predictive" => v(&["comment", "predictive"]),
        "storageclass" => v(&["storage.type", "storage.modifier", "storage"]),
        "string" => {
            if highlight.contains("escape") {
                v(&["constant.character.escape", "string"])
            } else if highlight.contains("regex") || highlight.contains("regexp") {
                v(&["string.regexp", "string"])
            } else if highlight.contains("documentation") {
                v(&["string.quoted.docstring", "string"])
            } else {
                v(&["string"])
            }
        }
        "tag" => {
            if highlight.contains("doctype") {
                v(&["meta.tag.doctype", "entity.name.tag", "tag"])
            } else if highlight.contains("error") {
                v(&["invalid", "entity.name.tag", "tag"])
            } else {
                v(&["entity.name.tag", "tag"])
            }
        }
        "text" => match highlight {
            "text.literal" => v(&["markup.raw", "string", "text"]),
            "text.uri" => v(&["markup.underline.link", "string.other.link", "text"]),
            "text.note" => v(&["comment", "markup.quote", "text"]),
            "text.warning" => v(&["invalid.deprecated", "invalid", "text"]),
            "text.danger" => v(&["invalid.illegal", "invalid", "text"]),
            "text.reference" => v(&["variable", "text"]),
            _ => v(&["text"]),
        },
        "title" => v(&["markup.heading", "entity.name.section", "title"]),
        "type" => {
            if highlight.contains("builtin") {
                v(&["support.type", "entity.name.type", "type"])
            } else if highlight.contains("qualifier") {
                v(&["storage.type", "entity.name.type", "type"])
            } else {
                v(&["entity.name.type", "support.type", "type"])
            }
        }
        "variable" => {
            if highlight.contains("builtin") {
                v(&["variable.language", "variable.builtin", "variable"])
            } else if highlight.contains("parameter") {
                v(&["variable.parameter", "variable"])
            } else if highlight.contains("member") {
                v(&[
                    "variable.other.member",
                    "variable.other.property",
                    "variable",
                ])
            } else if highlight.contains("special") {
                v(&["variable.other.constant", "variable"])
            } else {
                v(&["variable", "variable.parameter"])
            }
        }
        "variant" => v(&["constant.language", "variant"]),
        _ => vec![head.to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_basic_syntax_colors() {
        let json = br##"{
          "name": "Test Theme",
          "tokenColors": [
            { "scope": "comment", "settings": { "foreground": "#111111" } },
            { "scope": ["keyword", "storage.type"], "settings": { "foreground": "#222222" } },
            { "scope": "entity.name.function", "settings": { "foreground": "#333333" } },
            { "scope": "string", "settings": { "foreground": "#444444" } }
          ]
        }"##;

        let cfg = syntax_theme_patch_from_vscode_json(json).expect("valid json");
        assert_eq!(
            cfg.colors.get("color.syntax.comment").map(String::as_str),
            Some("#111111")
        );
        assert_eq!(
            cfg.colors.get("color.syntax.keyword").map(String::as_str),
            Some("#222222")
        );
        assert_eq!(
            cfg.colors.get("color.syntax.function").map(String::as_str),
            Some("#333333")
        );
        assert_eq!(
            cfg.colors.get("color.syntax.string").map(String::as_str),
            Some("#444444")
        );
    }

    #[test]
    fn later_rules_win_for_equal_specificity() {
        let json = br##"{
          "tokenColors": [
            { "scope": "comment", "settings": { "foreground": "#111111" } },
            { "scope": "comment", "settings": { "foreground": "#222222" } }
          ]
        }"##;
        let cfg = syntax_theme_patch_from_vscode_json(json).expect("valid json");
        assert_eq!(
            cfg.colors.get("color.syntax.comment").map(String::as_str),
            Some("#222222")
        );
    }

    #[test]
    fn more_specific_scope_wins() {
        let json = br##"{
          "tokenColors": [
            { "scope": "comment", "settings": { "foreground": "#111111" } },
            { "scope": "comment.block", "settings": { "foreground": "#222222" } }
          ]
        }"##;
        let cfg = syntax_theme_patch_from_vscode_json(json).expect("valid json");
        assert_eq!(
            cfg.colors.get("color.syntax.comment").map(String::as_str),
            Some("#222222")
        );
    }

    #[test]
    fn normalizes_hex_short_forms() {
        assert_eq!(
            normalize_color_value("#abc").as_deref(),
            Some("#aabbcc"),
            "#RGB should expand"
        );
        assert_eq!(
            normalize_color_value("#abcd").as_deref(),
            Some("#aabbccdd"),
            "#RGBA should expand"
        );
    }

    #[test]
    fn normalizes_rgb_functions() {
        assert_eq!(
            normalize_color_value("rgb(255, 0, 16)").as_deref(),
            Some("#ff0010")
        );
        assert_eq!(
            normalize_color_value("rgba(255, 0, 16, 0.5)").as_deref(),
            Some("#ff001080")
        );
        assert_eq!(
            normalize_color_value("rgba(255, 0, 16, 50%)").as_deref(),
            Some("#ff001080")
        );
    }

    #[test]
    fn can_generate_all_fret_syntax_tokens() {
        let json = br##"{
          "tokenColors": [
            { "scope": "keyword.operator", "settings": { "foreground": "#010203" } },
            { "scope": "string.regexp", "settings": { "foreground": "#040506" } }
          ]
        }"##;

        let cfg = syntax_theme_patch_from_vscode_json_with_options(
            json,
            VscodeSyntaxImportOptions {
                generate_all_fret_syntax_tokens: true,
            },
        )
        .expect("valid json");

        // These are `fret-syntax` highlight tags, not the minimal 10-token palette.
        assert_eq!(
            cfg.colors
                .get("color.syntax.keyword.operator")
                .map(String::as_str),
            Some("#010203")
        );
        assert_eq!(
            cfg.colors
                .get("color.syntax.string.regex")
                .map(String::as_str),
            Some("#040506"),
            "fret-syntax uses `string.regex`"
        );
    }

    #[test]
    fn mapping_can_force_foreground() {
        let json = br##"{
          "tokenColors": [
            { "scope": "comment", "settings": { "foreground": "#111111" } }
          ]
        }"##;

        let mapping = VscodeImportMapping {
            version: 1,
            tokens: HashMap::from([(
                "color.syntax.comment".to_string(),
                MappingEntry {
                    scopes: Vec::new(),
                    foreground: Some("#222222".to_string()),
                },
            )]),
            highlights: HashMap::new(),
        };

        let (cfg, report) =
            syntax_theme_patch_and_report_from_vscode_json_with_options_and_mapping(
                json,
                VscodeSyntaxImportOptions::default(),
                Some(&mapping),
            )
            .expect("valid json");

        assert_eq!(
            cfg.colors.get("color.syntax.comment").map(String::as_str),
            Some("#222222")
        );

        let comment = report
            .decisions
            .iter()
            .find(|d| d.token_key == "color.syntax.comment")
            .expect("comment decision");
        assert!(matches!(
            comment.source,
            ImportDecisionSource::MappingForeground
        ));
    }

    #[test]
    fn mapping_can_override_scopes() {
        let json = br##"{
          "tokenColors": [
            { "scope": "punctuation", "settings": { "foreground": "#aaaaaa" } },
            { "scope": "comment", "settings": { "foreground": "#111111" } }
          ]
        }"##;

        let mapping = VscodeImportMapping {
            version: 1,
            tokens: HashMap::from([(
                "color.syntax.comment".to_string(),
                MappingEntry {
                    scopes: vec!["punctuation".to_string()],
                    foreground: None,
                },
            )]),
            highlights: HashMap::new(),
        };

        let (cfg, report) =
            syntax_theme_patch_and_report_from_vscode_json_with_options_and_mapping(
                json,
                VscodeSyntaxImportOptions::default(),
                Some(&mapping),
            )
            .expect("valid json");

        assert_eq!(
            cfg.colors.get("color.syntax.comment").map(String::as_str),
            Some("#aaaaaa")
        );

        let comment = report
            .decisions
            .iter()
            .find(|d| d.token_key == "color.syntax.comment")
            .expect("comment decision");
        assert!(matches!(
            comment.source,
            ImportDecisionSource::MappingScopes
        ));
    }
}
