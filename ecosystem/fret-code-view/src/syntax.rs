use fret_core::Color;
use fret_ui::Theme;

pub(crate) fn syntax_color(theme: &Theme, highlight: &str) -> Option<Color> {
    let key = format!("color.syntax.{highlight}");
    if let Some(c) = theme.color_by_key(&key) {
        return Some(c);
    }

    let fallback = highlight.split('.').next().unwrap_or(highlight);
    match fallback {
        "comment" => Some(theme.color_required("muted-foreground")),
        "keyword" | "operator" => Some(theme.color_required("primary")),
        "property" | "variable" => Some(theme.color_required("foreground")),
        "punctuation" => Some(theme.color_required("muted-foreground")),

        // These are still treated as editor-ish baseline tokens until a dedicated SyntaxTheme
        // subsystem lands.
        "string" => Some(theme.color_required("color.viewport.gizmo.y")),
        "number" | "boolean" | "constant" => {
            Some(theme.color_required("color.viewport.rotate_gizmo"))
        }
        "type" | "constructor" => Some(theme.color_required("color.viewport.marker")),
        "function" => Some(theme.color_required("color.viewport.drag_line.orbit")),
        _ => None,
    }
}
