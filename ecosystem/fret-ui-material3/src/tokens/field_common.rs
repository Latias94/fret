//! Shared token fallback helpers for Material 3 field-family chrome.

use fret_core::{Color, Corners, Edges, Px};
use fret_ui::{TextInputStyle, Theme};

use crate::foundation::token_resolver::{MaterialTokenResolver, alpha_mul, blend_over};
use crate::tokens::shape;

const DEFAULT_CONTAINER_HEIGHT: Px = Px(56.0);
const DEFAULT_CONTAINER_SHAPE: Px = Px(4.0);
const DEFAULT_ICON_SIZE: Px = Px(24.0);
const DEFAULT_OUTLINE_WIDTH: Px = Px(1.0);
const DEFAULT_FOCUS_OUTLINE_WIDTH: Px = Px(2.0);
const DEFAULT_ACTIVE_INDICATOR_HEIGHT: Px = Px(1.0);
const DEFAULT_FOCUS_ACTIVE_INDICATOR_HEIGHT: Px = Px(2.0);
const DEFAULT_DISABLED_CONTENT_OPACITY: f32 = 0.38;
const DEFAULT_DISABLED_CONTAINER_OPACITY: f32 = 0.04;
const DEFAULT_DISABLED_OUTLINE_OPACITY: f32 = 0.12;
const DEFAULT_HOVER_STATE_LAYER_OPACITY: f32 = 0.08;
const DEFAULT_SELECTION_ALPHA: f32 = 0.35;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldVariant {
    Outlined,
    Filled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FieldState {
    pub hovered: bool,
    pub disabled: bool,
    pub error: bool,
    pub focused: bool,
}

impl FieldState {
    pub(crate) const fn new(hovered: bool, disabled: bool, error: bool, focused: bool) -> Self {
        Self {
            hovered,
            disabled,
            error,
            focused,
        }
    }

    fn forced_focus(self) -> Self {
        Self {
            hovered: false,
            focused: true,
            ..self
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FieldTokenSet {
    pub outlined: &'static str,
    pub filled: &'static str,
}

impl FieldTokenSet {
    pub(crate) const fn new(outlined: &'static str, filled: &'static str) -> Self {
        Self { outlined, filled }
    }

    pub(crate) const fn prefix(self, variant: FieldVariant) -> &'static str {
        match variant {
            FieldVariant::Outlined => self.outlined,
            FieldVariant::Filled => self.filled,
        }
    }
}

pub(crate) fn container_height(theme: &Theme, prefix: &str) -> Px {
    theme
        .metric_by_key(&field_key(prefix, "container.height"))
        .unwrap_or(DEFAULT_CONTAINER_HEIGHT)
}

pub(crate) fn container_shape(theme: &Theme, prefix: &str, variant: FieldVariant) -> Corners {
    match variant {
        FieldVariant::Outlined => outlined_container_shape(theme, prefix),
        FieldVariant::Filled => filled_container_shape(theme, prefix),
    }
}

pub(crate) fn icon_size(theme: &Theme, prefix: &str, role: FieldIconRole) -> Px {
    theme
        .metric_by_key(&field_key(prefix, role.size_suffix()))
        .unwrap_or(DEFAULT_ICON_SIZE)
}

pub(crate) fn role_color_with_opacity(
    theme: &Theme,
    prefix: &str,
    role: &str,
    state: FieldState,
    sys_fallback: &str,
) -> (Color, f32) {
    let (color_key, opacity_key) = state_role_color_keys(prefix, role, state);
    MaterialTokenResolver::new(theme).color_comp_or_sys_with_opacity(
        &color_key,
        sys_fallback,
        opacity_key.as_deref(),
        state.default_role_opacity(),
    )
}

pub(crate) fn role_color(
    theme: &Theme,
    prefix: &str,
    role: &str,
    state: FieldState,
    sys_fallback: &str,
) -> Color {
    let (color, opacity) = role_color_with_opacity(theme, prefix, role, state, sys_fallback);
    alpha_mul(color, opacity.clamp(0.0, 1.0))
}

pub(crate) fn hover_state_layer(theme: &Theme, prefix: &str, error: bool) -> (Color, f32) {
    let color_key = field_key(prefix, "hover.state-layer.color");
    let opacity_key = if error {
        field_key(prefix, "error.hover.state-layer.opacity")
    } else {
        field_key(prefix, "hover.state-layer.opacity")
    };

    MaterialTokenResolver::new(theme).color_comp_or_sys_with_opacity(
        &color_key,
        "md.sys.color.on-surface",
        Some(&opacity_key),
        DEFAULT_HOVER_STATE_LAYER_OPACITY,
    )
}

pub(crate) fn outline(theme: &Theme, prefix: &str, state: FieldState) -> (Px, Color, f32) {
    let width = theme
        .metric_by_key(&field_key(prefix, outline_width_suffix(state)))
        .unwrap_or_else(|| outline_width_default(state));
    let (color_key, opacity_key) = state_role_color_keys(prefix, "outline", state);
    let (color, opacity) = MaterialTokenResolver::new(theme).color_comp_or_sys_with_opacity(
        &color_key,
        "md.sys.color.outline",
        opacity_key.as_deref(),
        outline_opacity_default(state),
    );
    (width, color, opacity)
}

pub(crate) fn active_indicator(
    theme: &Theme,
    prefix: &str,
    state: FieldState,
    sys_fallback: &str,
) -> (Px, Color, f32) {
    let height = theme
        .metric_by_key(&field_key(prefix, active_indicator_height_suffix(state)))
        .unwrap_or_else(|| active_indicator_height_default(state));
    let (color_key, opacity_key) = state_role_color_keys(prefix, "active-indicator", state);
    let (color, opacity) = MaterialTokenResolver::new(theme).color_comp_or_sys_with_opacity(
        &color_key,
        sys_fallback,
        opacity_key.as_deref(),
        state.default_role_opacity(),
    );
    (height, color, opacity)
}

pub(crate) fn outlined_text_input_style(
    theme: &Theme,
    prefix: &str,
    state: FieldState,
) -> TextInputStyle {
    let mut style = TextInputStyle::default();
    style.corner_radii = outlined_container_shape(theme, prefix);
    style.focus_ring = None;
    style.padding = field_content_padding();
    style.background = MaterialTokenResolver::new(theme).color_comp_or_sys(
        &field_key(prefix, "container.color"),
        "md.sys.color.surface",
    );

    let (border_width, border_color, border_opacity) = outline(theme, prefix, state);
    let (_, focused_border_color, focused_border_opacity) =
        outline(theme, prefix, state.forced_focus());
    style.border = Edges::all(border_width);
    style.border_color = alpha_mul(border_color, border_opacity.clamp(0.0, 1.0));
    style.border_color_focused =
        alpha_mul(focused_border_color, focused_border_opacity.clamp(0.0, 1.0));

    apply_text_input_ink(theme, prefix, state, &mut style);
    style
}

pub(crate) fn filled_text_input_style(
    theme: &Theme,
    prefix: &str,
    state: FieldState,
    active_indicator_sys_fallback: &str,
) -> TextInputStyle {
    let mut style = TextInputStyle::default();
    style.corner_radii = filled_container_shape(theme, prefix);
    style.focus_ring = None;
    style.padding = field_content_padding();
    style.background = filled_container_background(theme, prefix, state.disabled);

    let (height, color, opacity) =
        active_indicator(theme, prefix, state, active_indicator_sys_fallback);
    let (_, focused_color, focused_opacity) = active_indicator(
        theme,
        prefix,
        state.forced_focus(),
        active_indicator_sys_fallback,
    );
    style.border = Edges {
        top: Px(0.0),
        right: Px(0.0),
        bottom: height,
        left: Px(0.0),
    };
    style.border_color = alpha_mul(color, opacity.clamp(0.0, 1.0));
    style.border_color_focused = alpha_mul(focused_color, focused_opacity.clamp(0.0, 1.0));

    apply_text_input_ink(theme, prefix, state, &mut style);
    style
}

pub(crate) fn placeholder_color(theme: &Theme, prefix: &str, state: FieldState) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    let mut color = tokens.color_comp_or_sys_or(
        &field_key(prefix, "input-text.placeholder.color"),
        "md.sys.color.on-surface-variant",
        TextInputStyle::default().placeholder_color,
    );

    if state.disabled {
        color = alpha_mul(
            color,
            tokens.number_optional(
                Some(&field_key(prefix, "disabled.input-text.opacity")),
                DEFAULT_DISABLED_CONTENT_OPACITY,
            ),
        );
    }

    color
}

fn outlined_container_shape(theme: &Theme, prefix: &str) -> Corners {
    shape::corners_or_metric(theme, &field_key(prefix, "container.shape"))
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.extra-small"))
        .unwrap_or_else(|| Corners::all(DEFAULT_CONTAINER_SHAPE))
}

fn filled_container_shape(theme: &Theme, prefix: &str) -> Corners {
    if let Some(corners) = shape::corners_or_metric(theme, &field_key(prefix, "container.shape")) {
        return corners;
    }
    if let Some(corners) = theme.corners_by_key("md.sys.shape.corner.extra-small.top") {
        return corners;
    }

    let r = theme
        .metric_by_key("md.sys.shape.corner.extra-small")
        .unwrap_or(DEFAULT_CONTAINER_SHAPE);
    Corners {
        top_left: r,
        top_right: r,
        bottom_right: Px(0.0),
        bottom_left: Px(0.0),
    }
}

fn filled_container_background(theme: &Theme, prefix: &str, disabled: bool) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    let mut background = tokens.color_comp_or_sys_chain(
        &field_key(prefix, "container.color"),
        &[
            "md.sys.color.surface-container-highest",
            "md.sys.color.surface",
        ],
    );

    if disabled {
        let (overlay, opacity) = tokens.color_comp_or_sys_with_opacity(
            &field_key(prefix, "disabled.container.color"),
            "md.sys.color.on-surface",
            Some(&field_key(prefix, "disabled.container.opacity")),
            DEFAULT_DISABLED_CONTAINER_OPACITY,
        );
        background = blend_over(background, overlay, opacity);
    }

    background
}

fn apply_text_input_ink(
    theme: &Theme,
    prefix: &str,
    state: FieldState,
    style: &mut TextInputStyle,
) {
    style.text_color = role_color(
        theme,
        prefix,
        "input-text",
        state,
        "md.sys.color.on-surface",
    );
    style.placeholder_color = placeholder_color(theme, prefix, state);
    style.selection_color = theme
        .color_by_key("md.sys.color.primary")
        .map(|c| alpha_mul(c, DEFAULT_SELECTION_ALPHA))
        .unwrap_or(style.selection_color);
    style.caret_color = caret_color(theme, prefix, state);
    style.preedit_color = theme
        .color_by_key("md.sys.color.primary")
        .unwrap_or(style.preedit_color);
    style.preedit_underline_color = style.preedit_color;
}

fn caret_color(theme: &Theme, prefix: &str, state: FieldState) -> Color {
    let tokens = MaterialTokenResolver::new(theme);
    let mut color = if state.error && state.focused {
        tokens.color_comp_or_sys_chain(
            &field_key(prefix, "error.focus.caret.color"),
            &["md.sys.color.error", "md.sys.color.on-surface"],
        )
    } else {
        tokens.color_comp_or_sys_chain(
            &field_key(prefix, "caret.color"),
            &["md.sys.color.primary", "md.sys.color.on-surface"],
        )
    };

    if state.disabled {
        color = alpha_mul(color, DEFAULT_DISABLED_CONTENT_OPACITY);
    }

    color
}

fn field_content_padding() -> Edges {
    Edges {
        top: Px(18.0),
        right: Px(16.0),
        bottom: Px(14.0),
        left: Px(16.0),
    }
}

fn state_role_color_keys(prefix: &str, role: &str, state: FieldState) -> (String, Option<String>) {
    let color_suffix = state_role_color_suffix(role, state);
    let opacity_suffix = state
        .disabled
        .then(|| field_key(prefix, &format!("disabled.{role}.opacity")));
    (
        field_key(prefix, &format!("{color_suffix}.color")),
        opacity_suffix,
    )
}

fn state_role_color_suffix(role: &str, state: FieldState) -> String {
    if state.disabled {
        format!("disabled.{role}")
    } else if state.error && state.focused {
        format!("error.focus.{role}")
    } else if state.error && state.hovered {
        format!("error.hover.{role}")
    } else if state.error {
        format!("error.{role}")
    } else if state.focused {
        format!("focus.{role}")
    } else if state.hovered {
        format!("hover.{role}")
    } else {
        role.to_string()
    }
}

fn outline_width_suffix(state: FieldState) -> &'static str {
    if state.disabled {
        "disabled.outline.width"
    } else if state.focused {
        "focus.outline.width"
    } else if state.hovered {
        "hover.outline.width"
    } else {
        "outline.width"
    }
}

fn outline_width_default(state: FieldState) -> Px {
    if state.focused {
        DEFAULT_FOCUS_OUTLINE_WIDTH
    } else {
        DEFAULT_OUTLINE_WIDTH
    }
}

fn outline_opacity_default(state: FieldState) -> f32 {
    if state.disabled {
        DEFAULT_DISABLED_OUTLINE_OPACITY
    } else {
        1.0
    }
}

fn active_indicator_height_suffix(state: FieldState) -> &'static str {
    if state.disabled {
        "disabled.active-indicator.height"
    } else if state.focused {
        "focus.active-indicator.height"
    } else if state.hovered {
        "hover.active-indicator.height"
    } else {
        "active-indicator.height"
    }
}

fn active_indicator_height_default(state: FieldState) -> Px {
    if state.focused {
        DEFAULT_FOCUS_ACTIVE_INDICATOR_HEIGHT
    } else {
        DEFAULT_ACTIVE_INDICATOR_HEIGHT
    }
}

impl FieldState {
    fn default_role_opacity(self) -> f32 {
        if self.disabled {
            DEFAULT_DISABLED_CONTENT_OPACITY
        } else {
            1.0
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FieldIconRole {
    Leading,
    Trailing,
}

impl FieldIconRole {
    fn size_suffix(self) -> &'static str {
        match self {
            FieldIconRole::Leading => "leading-icon.size",
            FieldIconRole::Trailing => "trailing-icon.size",
        }
    }
}

fn field_key(prefix: &str, suffix: &str) -> String {
    format!("{prefix}.{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokens::v30::{TypographyOptions, theme_config};
    use fret_app::App;
    use fret_ui::{Theme, theme::ThemeConfig};

    fn theme_with_patch(patch: ThemeConfig) -> (App, Theme) {
        let mut app = App::new();
        let base = theme_config(TypographyOptions::default());
        Theme::with_global_mut(&mut app, |theme| theme.apply_config(&base));
        Theme::with_global_mut(&mut app, |theme| theme.apply_config_patch(&patch));
        let theme = Theme::global(&app).clone();
        (app, theme)
    }

    #[test]
    fn field_container_metrics_prefer_component_tokens() {
        let mut patch = ThemeConfig::default();
        patch.metrics.insert(
            "md.comp.outlined-test-field.container.height".to_string(),
            64.0,
        );
        patch.metrics.insert(
            "md.comp.outlined-test-field.leading-icon.size".to_string(),
            20.0,
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            container_height(&theme, "md.comp.outlined-test-field"),
            Px(64.0)
        );
        assert_eq!(
            icon_size(
                &theme,
                "md.comp.outlined-test-field",
                FieldIconRole::Leading
            ),
            Px(20.0)
        );
    }

    #[test]
    fn filled_field_shape_preserves_top_corner_fallback() {
        let mut patch = ThemeConfig::default();
        patch.corners.insert(
            "md.sys.shape.corner.extra-small.top".to_string(),
            Corners {
                top_left: Px(6.0),
                top_right: Px(6.0),
                bottom_right: Px(0.0),
                bottom_left: Px(0.0),
            },
        );
        let (_app, theme) = theme_with_patch(patch);

        assert_eq!(
            container_shape(&theme, "md.comp.filled-test-field", FieldVariant::Filled),
            Corners {
                top_left: Px(6.0),
                top_right: Px(6.0),
                bottom_right: Px(0.0),
                bottom_left: Px(0.0),
            }
        );
    }

    #[test]
    fn disabled_field_role_colors_default_to_disabled_content_opacity() {
        let mut patch = ThemeConfig::default();
        patch.colors.insert(
            "md.comp.outlined-test-field.disabled.label-text.color".to_string(),
            "#112233".to_string(),
        );
        let (_app, theme) = theme_with_patch(patch);
        let state = FieldState::new(false, true, false, false);

        let (_color, opacity) = role_color_with_opacity(
            &theme,
            "md.comp.outlined-test-field",
            "label-text",
            state,
            "md.sys.color.on-surface-variant",
        );
        assert_eq!(opacity, 0.38);
    }

    #[test]
    fn outlined_field_error_hover_prefers_error_hover_color() {
        let mut patch = ThemeConfig::default();
        patch.colors.insert(
            "md.comp.outlined-test-field.error.hover.outline.color".to_string(),
            "#112233".to_string(),
        );
        let (_app, theme) = theme_with_patch(patch);
        let state = FieldState::new(true, false, true, false);

        let (_width, color, opacity) = outline(&theme, "md.comp.outlined-test-field", state);
        assert_eq!(
            color,
            theme
                .color_by_key("md.comp.outlined-test-field.error.hover.outline.color")
                .expect("patched error hover outline color")
        );
        assert_eq!(opacity, 1.0);
    }
}
