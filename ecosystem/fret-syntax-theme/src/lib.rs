//! Theme helpers for syntax highlighting.
//!
//! This crate centralizes how `fret-syntax` / tree-sitter highlight tags map to Fret theme tokens.
//! Consumers (code view/editor, markdown, AI elements) should use these helpers to avoid drift.

use fret_core::Color;
use fret_ui::Theme;

/// Resolve a tree-sitter highlight tag (e.g. `keyword.operator`) into a theme color.
///
/// Lookup order:
///
/// 1) `color.syntax.<highlight>`
/// 2) prefix fallback: `color.syntax.keyword.operator` -> `color.syntax.keyword`
/// 3) built-in semantic fallbacks for common tags
pub fn syntax_color(theme: &Theme, highlight: &str) -> Option<Color> {
    let mut cur = Some(highlight);
    while let Some(name) = cur {
        let mut key = String::with_capacity("color.syntax.".len() + name.len());
        key.push_str("color.syntax.");
        key.push_str(name);
        if let Some(c) = theme.color_by_key(key.as_str()) {
            return Some(c);
        }
        cur = name.rsplit_once('.').map(|(prefix, _)| prefix);
    }

    let fallback = highlight.split('.').next().unwrap_or(highlight);
    match fallback {
        "comment" => Some(theme.color_token("muted-foreground")),
        "keyword" | "operator" => Some(theme.color_token("primary")),
        "property" | "variable" => Some(theme.color_token("foreground")),
        "punctuation" => Some(theme.color_token("muted-foreground")),

        "string" => Some(theme.color_token("foreground")),
        "number" | "boolean" | "constant" => Some(theme.color_token("primary")),
        "type" | "constructor" | "function" => Some(theme.color_token("foreground")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use fret_app::App;
    use fret_ui::theme::ThemeConfig;

    use super::*;

    #[test]
    fn resolves_prefix_fallback_tokens() {
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            let mut colors = HashMap::<String, String>::new();
            colors.insert("color.syntax.keyword".to_string(), "#ff0000".to_string());
            theme.apply_config(&ThemeConfig {
                name: "test".to_string(),
                colors,
                ..Default::default()
            });

            let exact = syntax_color(theme, "keyword").expect("exact token should resolve");
            let prefixed = syntax_color(theme, "keyword.operator")
                .expect("prefixed tag should resolve via prefix fallback");
            assert_eq!(exact, prefixed);
        });
    }
}
