use fret_core::Color;
use fret_ui::Theme;

pub(crate) fn syntax_color(theme: &Theme, highlight: &str) -> Option<Color> {
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
