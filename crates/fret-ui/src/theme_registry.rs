#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeTokenKind {
    Color,
    Metric,
    Corners,
    Number,
    DurationMs,
    Easing,
    TextStyle,
}

pub fn canonicalize_token_key(kind: ThemeTokenKind, key: &str) -> &str {
    let key = key.trim();
    if key.is_empty() {
        return key;
    }

    match kind {
        ThemeTokenKind::Color => canonicalize_color_key(key),
        ThemeTokenKind::Metric => canonicalize_metric_key(key),
        ThemeTokenKind::Corners => key,
        ThemeTokenKind::Number => key,
        ThemeTokenKind::DurationMs => key,
        ThemeTokenKind::Easing => key,
        ThemeTokenKind::TextStyle => key,
    }
}

fn canonicalize_color_key(key: &str) -> &str {
    match key {
        // shadcn-style core semantic keys (aliases).
        "ring_offset_background" => "ring-offset-background",

        // shadcn surface sub-keys (aliases).
        "card.background" => "card",
        "card.foreground" => "card-foreground",
        "card-foreground" => "card-foreground",

        "popover.background" => "popover",
        "popover.foreground" => "popover-foreground",
        "popover-foreground" => "popover-foreground",

        // Keep historic dotted aliases accepted by some ports.
        "input.border" => "input",

        "primary.background" => "primary",
        "primary.foreground" => "primary-foreground",

        "secondary.background" => "secondary",
        "secondary.foreground" => "secondary-foreground",

        "destructive.background" => "destructive",
        "destructive.foreground" => "destructive-foreground",

        "muted.background" => "muted",
        "muted.foreground" => "muted-foreground",

        "accent.background" => "accent",
        "accent.foreground" => "accent-foreground",

        _ => key,
    }
}

fn canonicalize_metric_key(key: &str) -> &str {
    match key {
        // gpui-component historic naming.
        "mono_font.size" => "mono_font.size",
        "mono_font.line_height" => "mono_font.line_height",

        // Support both dotted and shadcn kebab-case for the first migration window.
        "radius.lg" => "radius.lg",

        _ => key,
    }
}
