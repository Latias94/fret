use fret_core::Color;
use fret_ui::Theme;

pub(crate) fn syntax_color(theme: &Theme, highlight: &str) -> Option<Color> {
    let mut key = String::with_capacity("color.syntax.".len() + highlight.len());
    key.push_str("color.syntax.");
    key.push_str(highlight);
    if let Some(c) = theme.color_by_key(key.as_str()) {
        return Some(c);
    }

    let fallback = highlight.split('.').next().unwrap_or(highlight);
    if fallback != highlight {
        let mut key = String::with_capacity("color.syntax.".len() + fallback.len());
        key.push_str("color.syntax.");
        key.push_str(fallback);
        if let Some(c) = theme.color_by_key(key.as_str()) {
            return Some(c);
        }
    }

    match fallback {
        "comment" => Some(theme.color_required("muted-foreground")),
        "keyword" | "operator" => Some(theme.color_required("primary")),
        "property" | "variable" => Some(theme.color_required("foreground")),
        "punctuation" => Some(theme.color_required("muted-foreground")),

        "string" => Some(theme.color_required("foreground")),
        "number" | "boolean" | "constant" => Some(theme.color_required("primary")),
        "type" | "constructor" | "function" => Some(theme.color_required("foreground")),
        _ => None,
    }
}
