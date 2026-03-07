use fret_core::{
    Color, FontId, Px, TextLineHeightPolicy, TextStrutStyle, TextStyle, TextStyleRefinement,
    TextVerticalPlacement,
};
use fret_ui::element::AnyElement;

use crate::style::ThemeTokenRead;
use crate::theme_tokens;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTextSize {
    Xs,
    Sm,
    Base,
    Prose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiTextFamily {
    Ui,
    Monospace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextIntent {
    Control,
    Content,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypographyPreset {
    pub intent: TextIntent,
    pub family: UiTextFamily,
    pub size: UiTextSize,
}

impl TypographyPreset {
    pub const fn new(intent: TextIntent, family: UiTextFamily, size: UiTextSize) -> Self {
        Self {
            intent,
            family,
            size,
        }
    }

    pub const fn control_ui(size: UiTextSize) -> Self {
        Self::new(TextIntent::Control, UiTextFamily::Ui, size)
    }

    pub const fn content_ui(size: UiTextSize) -> Self {
        Self::new(TextIntent::Content, UiTextFamily::Ui, size)
    }

    pub const fn control_monospace(size: UiTextSize) -> Self {
        Self::new(TextIntent::Control, UiTextFamily::Monospace, size)
    }

    pub const fn content_monospace(size: UiTextSize) -> Self {
        Self::new(TextIntent::Content, UiTextFamily::Monospace, size)
    }

    pub fn resolve(self, theme: &impl ThemeTokenRead) -> TextStyle {
        match self.intent {
            TextIntent::Control => control_text_style_with_family(theme, self.size, self.family),
            TextIntent::Content => content_text_style_with_family(theme, self.size, self.family),
        }
    }
}

fn font_size(theme: &impl ThemeTokenRead) -> Px {
    theme
        .metric_by_key("font.size")
        .unwrap_or_else(|| theme.metric_token("font.size"))
}

fn font_line_height(theme: &impl ThemeTokenRead) -> Px {
    theme
        .metric_by_key("font.line_height")
        .unwrap_or_else(|| theme.metric_token("font.line_height"))
}

fn base_line_height_ratio(theme: &impl ThemeTokenRead) -> f32 {
    let base_size_px = font_size(theme).0;
    let base_line_height_px = font_line_height(theme).0;
    if base_size_px.is_finite()
        && base_line_height_px.is_finite()
        && base_size_px > 0.0
        && base_line_height_px > 0.0
    {
        base_line_height_px / base_size_px
    } else {
        1.25
    }
}

/// Creates a `TextStyle` with an explicit fixed line box.
///
/// This is intended for UI control text where layout stability is preferred over accommodating
/// taller fallback glyphs.
pub fn fixed_line_box_style(font: FontId, size: Px, line_height: Px) -> TextStyle {
    TextStyle {
        font,
        size,
        line_height: Some(line_height),
        line_height_policy: TextLineHeightPolicy::FixedFromStyle,
        vertical_placement: TextVerticalPlacement::BoundsAsLineBox,
        ..Default::default()
    }
}

/// Applies a high-level intent to an existing `TextStyle`.
///
/// Notes:
/// - `TextIntent::Control` only produces stable fixed line boxes when the style has an explicit
///   `line_height` or `line_height_em` (see `TextLineHeightPolicy` contract).
pub fn with_intent(mut style: TextStyle, intent: TextIntent) -> TextStyle {
    match intent {
        TextIntent::Control => {
            style.line_height_policy = TextLineHeightPolicy::FixedFromStyle;
            style.vertical_placement = TextVerticalPlacement::BoundsAsLineBox;
        }
        TextIntent::Content => {
            style.line_height_policy = TextLineHeightPolicy::ExpandToFit;
            style.vertical_placement = TextVerticalPlacement::CenterMetricsBox;
        }
    }
    style
}

pub fn as_control_text(style: TextStyle) -> TextStyle {
    with_intent(style, TextIntent::Control)
}

pub fn as_content_text(style: TextStyle) -> TextStyle {
    with_intent(style, TextIntent::Content)
}

fn force_strut_from_style(style: &TextStyle) -> Option<TextStrutStyle> {
    if style.line_height.is_none() && style.line_height_em.is_none() {
        return None;
    }

    Some(TextStrutStyle {
        line_height: style.line_height,
        line_height_em: style.line_height_em,
        force: true,
        ..Default::default()
    })
}

/// Returns a theme-based text style intended for content-like multiline text areas.
///
/// This leaves `TextLineHeightPolicy` as `ExpandToFit` to avoid clipping.
pub fn text_area_content_text_style(theme: &impl ThemeTokenRead) -> TextStyle {
    TextStyle {
        font: FontId::ui(),
        size: font_size(theme),
        line_height: Some(font_line_height(theme)),
        ..Default::default()
    }
}

/// Returns a theme-based text style intended for content-like multiline text areas, scaled to an
/// explicit size.
pub fn text_area_content_text_style_scaled(
    theme: &impl ThemeTokenRead,
    font: FontId,
    size: Px,
) -> TextStyle {
    let ratio = base_line_height_ratio(theme);
    let line_height = Px((size.0 * ratio).max(size.0));

    let mut style = TextStyle {
        font,
        size,
        line_height: Some(line_height),
        ..Default::default()
    };
    style.line_height_policy = TextLineHeightPolicy::ExpandToFit;
    style.vertical_placement = TextVerticalPlacement::CenterMetricsBox;
    style
}

/// Returns an opt-in text style intended for UI/form multiline text areas.
///
/// This favors stable per-line metrics via:
/// - fixed line height policy (stable line boxes),
/// - and a forced strut derived from the chosen style line height (stable baseline across mixed
///   scripts / emoji fallback runs).
pub fn text_area_control_text_style(theme: &impl ThemeTokenRead) -> TextStyle {
    let mut style = text_area_content_text_style(theme);
    style.line_height_policy = TextLineHeightPolicy::FixedFromStyle;
    style.strut_style = force_strut_from_style(&style);
    style
}

/// Returns an opt-in text style intended for UI/form multiline text areas, scaled to an explicit
/// size.
pub fn text_area_control_text_style_scaled(
    theme: &impl ThemeTokenRead,
    font: FontId,
    size: Px,
) -> TextStyle {
    let ratio = base_line_height_ratio(theme);
    let line_height = Px((size.0 * ratio).max(size.0));

    let mut style = TextStyle {
        font,
        size,
        line_height: Some(line_height),
        ..Default::default()
    };
    style.line_height_policy = TextLineHeightPolicy::FixedFromStyle;
    style.strut_style = force_strut_from_style(&style);
    style
}

/// Returns a control-text style intended for UI components (stable line box).
///
/// This is a policy helper for ecosystem components. It is intentionally not a `fret-ui` runtime
/// commitment (see ADR 0066).
pub fn control_text_style(theme: &impl ThemeTokenRead, size: UiTextSize) -> TextStyle {
    control_text_style_with_family(theme, size, UiTextFamily::Ui)
}

/// Returns a control-text style intended for UI components (stable line box).
pub fn control_text_style_with_family(
    theme: &impl ThemeTokenRead,
    size: UiTextSize,
    family: UiTextFamily,
) -> TextStyle {
    let font = match family {
        UiTextFamily::Ui => FontId::ui(),
        UiTextFamily::Monospace => FontId::monospace(),
    };

    match size {
        UiTextSize::Xs => {
            let px = theme
                .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_XS_PX)
                .unwrap_or(Px(12.0));
            let line_height = theme
                .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_XS_LINE_HEIGHT)
                .unwrap_or(Px(16.0));
            fixed_line_box_style(font, px, line_height)
        }
        UiTextSize::Sm => {
            let px = theme
                .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_PX)
                .unwrap_or_else(|| font_size(theme));
            let line_height = theme
                .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT)
                .unwrap_or_else(|| font_line_height(theme));
            fixed_line_box_style(font, px, line_height)
        }
        UiTextSize::Base => {
            let px = theme
                .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_BASE_PX)
                .unwrap_or_else(|| Px(font_size(theme).0 + 1.0));

            let line_height = theme
                .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_BASE_LINE_HEIGHT)
                .unwrap_or_else(|| Px((px.0 * base_line_height_ratio(theme)).max(px.0)));

            fixed_line_box_style(font, px, line_height)
        }
        UiTextSize::Prose => {
            let px = theme
                .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_PROSE_PX)
                .unwrap_or_else(|| Px(font_size(theme).0 + 2.0));
            let line_height = theme
                .metric_by_key(theme_tokens::metric::COMPONENT_TEXT_PROSE_LINE_HEIGHT)
                .unwrap_or_else(|| Px(font_line_height(theme).0 + 4.0));
            fixed_line_box_style(font, px, line_height)
        }
    }
}

/// Returns a content-text style intended for prose surfaces (avoid clipping).
pub fn content_text_style(theme: &impl ThemeTokenRead, size: UiTextSize) -> TextStyle {
    let mut style = control_text_style(theme, size);
    style.line_height_policy = TextLineHeightPolicy::ExpandToFit;
    style.vertical_placement = TextVerticalPlacement::CenterMetricsBox;
    style
}

/// Returns a content-text style intended for prose surfaces (avoid clipping).
pub fn content_text_style_with_family(
    theme: &impl ThemeTokenRead,
    size: UiTextSize,
    family: UiTextFamily,
) -> TextStyle {
    let mut style = control_text_style_with_family(theme, size, family);
    style.line_height_policy = TextLineHeightPolicy::ExpandToFit;
    style.vertical_placement = TextVerticalPlacement::CenterMetricsBox;
    style
}

/// Returns a control-text style scaled to an explicit font size, using the theme's baseline
/// `font.line_height / font.size` ratio.
///
/// This is intended for widget surfaces that take `TextStyle` directly (e.g. text inputs) where the
/// component decides the font size but still wants stable line box behavior.
pub fn control_text_style_scaled(theme: &impl ThemeTokenRead, font: FontId, size: Px) -> TextStyle {
    let ratio = base_line_height_ratio(theme);
    let line_height = Px((size.0 * ratio).max(size.0));
    fixed_line_box_style(font, size, line_height)
}

/// Returns a control-text style for a caller-chosen font size using the theme's `font.line_height`
/// metric directly (no scaling).
///
/// This matches common “UI control” usage where size and line height are independently tokenized.
pub fn control_text_style_for_font_size(
    theme: &impl ThemeTokenRead,
    font: FontId,
    size: Px,
) -> TextStyle {
    fixed_line_box_style(font, size, font_line_height(theme))
}

fn color_by_aliases(theme: &impl ThemeTokenRead, aliases: &[&str], fallback_token: &str) -> Color {
    aliases
        .iter()
        .find_map(|key| theme.color_by_key(key))
        .unwrap_or_else(|| theme.color_token(fallback_token))
}

pub fn muted_foreground_color(theme: &impl ThemeTokenRead) -> Color {
    color_by_aliases(
        theme,
        &["muted.foreground", "muted-foreground", "muted_foreground"],
        "muted-foreground",
    )
}

pub fn scope_text_style(element: AnyElement, refinement: TextStyleRefinement) -> AnyElement {
    element.inherit_text_style(refinement)
}

pub fn scope_text_style_with_color(
    element: AnyElement,
    refinement: TextStyleRefinement,
    foreground: Color,
) -> AnyElement {
    element
        .inherit_foreground(foreground)
        .inherit_text_style(refinement)
}

pub fn description_text_refinement(
    theme: &impl ThemeTokenRead,
    metric_prefix: &str,
) -> TextStyleRefinement {
    description_text_refinement_with_fallbacks(theme, metric_prefix, None, None)
}

pub fn description_text_refinement_with_fallbacks(
    theme: &impl ThemeTokenRead,
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

pub fn scope_description_text(
    element: AnyElement,
    theme: &impl ThemeTokenRead,
    metric_prefix: &str,
) -> AnyElement {
    scope_description_text_with_fallbacks(element, theme, metric_prefix, None, None)
}

pub fn scope_description_text_with_fallbacks(
    element: AnyElement,
    theme: &impl ThemeTokenRead,
    metric_prefix: &str,
    fallback_size_key: Option<&str>,
    fallback_line_height_key: Option<&str>,
) -> AnyElement {
    scope_text_style_with_color(
        element,
        description_text_refinement_with_fallbacks(
            theme,
            metric_prefix,
            fallback_size_key,
            fallback_line_height_key,
        ),
        muted_foreground_color(theme),
    )
}

pub fn refinement_from_style(style: &TextStyle) -> TextStyleRefinement {
    TextStyleRefinement {
        font: Some(style.font.clone()),
        size: Some(style.size),
        weight: Some(style.weight),
        slant: Some(style.slant),
        line_height: style.line_height,
        line_height_em: style.line_height_em,
        line_height_policy: Some(style.line_height_policy),
        letter_spacing_em: style.letter_spacing_em,
        vertical_placement: Some(style.vertical_placement),
        leading_distribution: Some(style.leading_distribution),
    }
}

pub fn preset_text_refinement(
    theme: &impl ThemeTokenRead,
    preset: TypographyPreset,
) -> TextStyleRefinement {
    refinement_from_style(&preset.resolve(theme))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_ui::element::{ContainerProps, ElementKind};
    use fret_ui::elements::GlobalElementId;
    use fret_ui::{Theme, ThemeConfig};

    #[test]
    fn description_text_refinement_uses_component_metrics_and_fixed_policy() {
        let mut app = fret_app::App::default();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("font.size".to_string(), 14.0),
                    ("font.line_height".to_string(), 20.0),
                    ("component.dialog.description_px".to_string(), 13.0),
                    ("component.dialog.description_line_height".to_string(), 18.0),
                ]),
                ..ThemeConfig::default()
            });
        });
        let theme = Theme::global(&app).snapshot();

        let refinement = description_text_refinement(&theme, "component.dialog.description");
        assert_eq!(refinement.size, Some(Px(13.0)));
        assert_eq!(refinement.line_height, Some(Px(18.0)));
        assert_eq!(
            refinement.line_height_policy,
            Some(TextLineHeightPolicy::FixedFromStyle)
        );
    }

    #[test]
    fn scope_description_text_attaches_color_and_inherited_refinement() {
        let mut app = fret_app::App::default();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("font.size".to_string(), 14.0),
                    ("font.line_height".to_string(), 20.0),
                    ("component.card.description_px".to_string(), 12.0),
                    ("component.card.description_line_height".to_string(), 17.0),
                ]),
                colors: std::collections::HashMap::from([(
                    "muted-foreground".to_string(),
                    "#778899".to_string(),
                )]),
                ..ThemeConfig::default()
            });
        });
        let theme = Theme::global(&app).snapshot();
        let element = scope_description_text(
            AnyElement::new(
                GlobalElementId(1),
                ElementKind::Container(ContainerProps::default()),
                Vec::new(),
            ),
            &theme,
            "component.card.description",
        );

        assert_eq!(
            element.inherited_foreground,
            Some(muted_foreground_color(&theme))
        );
        assert_eq!(
            element.inherited_text_style,
            Some(description_text_refinement(
                &theme,
                "component.card.description"
            ))
        );
    }

    #[test]
    fn preset_text_refinement_matches_resolved_preset() {
        let mut app = fret_app::App::default();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("font.size".to_string(), 14.0),
                    ("font.line_height".to_string(), 20.0),
                    (
                        crate::theme_tokens::metric::COMPONENT_TEXT_SM_PX.to_string(),
                        13.0,
                    ),
                    (
                        crate::theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT.to_string(),
                        18.0,
                    ),
                ]),
                ..ThemeConfig::default()
            });
        });
        let theme = Theme::global(&app).snapshot();
        let preset = TypographyPreset::control_ui(UiTextSize::Sm);
        let style = preset.resolve(&theme);

        assert_eq!(
            preset_text_refinement(&theme, preset),
            refinement_from_style(&style)
        );
    }

    #[test]
    fn with_intent_updates_line_height_policy() {
        let base = TextStyle {
            font: FontId::ui(),
            size: Px(12.0),
            line_height: Some(Px(16.0)),
            ..Default::default()
        };

        let control = with_intent(base.clone(), TextIntent::Control);
        assert_eq!(
            control.line_height_policy,
            TextLineHeightPolicy::FixedFromStyle
        );
        assert_eq!(
            control.vertical_placement,
            TextVerticalPlacement::BoundsAsLineBox
        );

        let content = with_intent(base, TextIntent::Content);
        assert_eq!(
            content.line_height_policy,
            TextLineHeightPolicy::ExpandToFit
        );
        assert_eq!(
            content.vertical_placement,
            TextVerticalPlacement::CenterMetricsBox
        );
    }

    #[test]
    fn typography_preset_resolves_to_intended_policy() {
        let mut app = fret_app::App::default();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("font.size".to_string(), 10.0),
                    ("font.line_height".to_string(), 15.0),
                ]),
                ..ThemeConfig::default()
            });
        });
        let theme = Theme::global(&app).clone();

        let control = TypographyPreset::control_ui(UiTextSize::Sm).resolve(&theme);
        assert_eq!(
            control.line_height_policy,
            TextLineHeightPolicy::FixedFromStyle
        );
        assert_eq!(
            control.vertical_placement,
            TextVerticalPlacement::BoundsAsLineBox
        );

        let content = TypographyPreset::content_ui(UiTextSize::Sm).resolve(&theme);
        assert_eq!(
            content.line_height_policy,
            TextLineHeightPolicy::ExpandToFit
        );
        assert_eq!(
            content.vertical_placement,
            TextVerticalPlacement::CenterMetricsBox
        );
    }

    #[test]
    fn control_text_styles_use_fixed_line_boxes() {
        let mut app = fret_app::App::default();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("font.size".to_string(), 10.0),
                    ("font.line_height".to_string(), 15.0),
                ]),
                ..ThemeConfig::default()
            });
        });
        let theme = Theme::global(&app).clone();

        for size in [
            UiTextSize::Xs,
            UiTextSize::Sm,
            UiTextSize::Base,
            UiTextSize::Prose,
        ] {
            let style = control_text_style(&theme, size);
            assert_eq!(
                style.line_height_policy,
                TextLineHeightPolicy::FixedFromStyle,
                "expected control text styles to use fixed line boxes: size={size:?}, style={style:?}"
            );
            assert!(
                style.line_height.is_some(),
                "expected control text styles to set an explicit line height: size={size:?}, style={style:?}"
            );
        }
    }

    #[test]
    fn content_text_styles_expand_to_fit() {
        let mut app = fret_app::App::default();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("font.size".to_string(), 10.0),
                    ("font.line_height".to_string(), 15.0),
                ]),
                ..ThemeConfig::default()
            });
        });
        let theme = Theme::global(&app).clone();

        for size in [
            UiTextSize::Xs,
            UiTextSize::Sm,
            UiTextSize::Base,
            UiTextSize::Prose,
        ] {
            let style = content_text_style(&theme, size);
            assert_eq!(
                style.line_height_policy,
                TextLineHeightPolicy::ExpandToFit,
                "expected content text styles to expand to fit: size={size:?}, style={style:?}"
            );
            assert!(
                style.line_height.is_some(),
                "expected content text styles to keep an explicit line height: size={size:?}, style={style:?}"
            );
        }
    }

    #[test]
    fn control_text_style_scaled_uses_theme_ratio_and_fixed_line_box() {
        let mut app = fret_app::App::default();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("font.size".to_string(), 10.0),
                    ("font.line_height".to_string(), 15.0),
                ]),
                ..ThemeConfig::default()
            });
        });
        let theme = Theme::global(&app).clone();

        let style = control_text_style_scaled(&theme, FontId::ui(), Px(20.0));
        assert_eq!(style.size, Px(20.0));
        assert_eq!(style.line_height, Some(Px(30.0)));
        assert_eq!(
            style.line_height_policy,
            TextLineHeightPolicy::FixedFromStyle
        );
    }
}
