//! Internal helpers for subtree-scoped description typography in shadcn recipes.

use fret_core::{Color, TextLineHeightPolicy, TextStyleRefinement};
use fret_ui::ThemeSnapshot;
use fret_ui::element::AnyElement;

pub(crate) fn muted_foreground(theme: &ThemeSnapshot) -> Color {
    theme
        .color_by_key("muted.foreground")
        .or_else(|| theme.color_by_key("muted-foreground"))
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_token("muted-foreground"))
}

pub(crate) fn description_refinement(
    theme: &ThemeSnapshot,
    metric_prefix: &str,
) -> TextStyleRefinement {
    description_refinement_with_fallbacks(theme, metric_prefix, None, None)
}

pub(crate) fn description_refinement_with_fallbacks(
    theme: &ThemeSnapshot,
    metric_prefix: &str,
    fallback_size_key: Option<&str>,
    fallback_line_height_key: Option<&str>,
) -> TextStyleRefinement {
    let size_key = format!("{metric_prefix}_px");
    let line_height_key = format!("{metric_prefix}_line_height");

    let size = theme
        .metric_by_key(&size_key)
        .or_else(|| fallback_size_key.and_then(|key| theme.metric_by_key(key)))
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let line_height = theme
        .metric_by_key(&line_height_key)
        .or_else(|| fallback_line_height_key.and_then(|key| theme.metric_by_key(key)))
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_token("font.line_height"));

    TextStyleRefinement {
        size: Some(size),
        line_height: Some(line_height),
        line_height_policy: Some(TextLineHeightPolicy::FixedFromStyle),
        ..Default::default()
    }
}

pub(crate) fn scope_description(
    element: AnyElement,
    theme: &ThemeSnapshot,
    metric_prefix: &str,
) -> AnyElement {
    element
        .inherit_foreground(muted_foreground(theme))
        .inherit_text_style(description_refinement(theme, metric_prefix))
}

pub(crate) fn scope_description_with_fallbacks(
    element: AnyElement,
    theme: &ThemeSnapshot,
    metric_prefix: &str,
    fallback_size_key: Option<&str>,
    fallback_line_height_key: Option<&str>,
) -> AnyElement {
    element
        .inherit_foreground(muted_foreground(theme))
        .inherit_text_style(description_refinement_with_fallbacks(
            theme,
            metric_prefix,
            fallback_size_key,
            fallback_line_height_key,
        ))
}
