//! Typed token access for Material 3 text fields.
//!
//! This module centralizes token key mapping and fallback chains so text field visuals remain
//! stable and drift-resistant during refactors.

use fret_core::{Color, Corners, Edges, Px};
use fret_ui::{TextInputStyle, Theme};

use crate::foundation::content::MaterialContentDefaults;
use crate::text_field::TextFieldVariant;

pub(crate) fn container_height(theme: &Theme, variant: TextFieldVariant) -> Px {
    match variant {
        TextFieldVariant::Outlined => theme
            .metric_by_key("md.comp.outlined-text-field.container.height")
            .unwrap_or(Px(56.0)),
        TextFieldVariant::Filled => theme
            .metric_by_key("md.comp.filled-text-field.container.height")
            .unwrap_or(Px(56.0)),
    }
}

fn outlined_container_corner(theme: &Theme) -> Corners {
    theme
        .corners_by_key("md.comp.outlined-text-field.container.shape")
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.extra-small"))
        .unwrap_or_else(|| Corners::all(Px(4.0)))
}

fn filled_container_corner(theme: &Theme) -> Corners {
    if let Some(corners) = theme.corners_by_key("md.comp.filled-text-field.container.shape") {
        return corners;
    }
    if let Some(corners) = theme.corners_by_key("md.sys.shape.corner.extra-small.top") {
        return corners;
    }
    let r = theme
        .metric_by_key("md.sys.shape.corner.extra-small")
        .unwrap_or(Px(4.0));
    Corners {
        top_left: r,
        top_right: r,
        bottom_right: Px(0.0),
        bottom_left: Px(0.0),
    }
}

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn blend_over(base: Color, overlay: Color, opacity: f32) -> Color {
    let a = (overlay.a * opacity).clamp(0.0, 1.0);
    if a <= 0.0 {
        return base;
    }

    let inv = 1.0 - a;
    Color {
        r: overlay.r * a + base.r * inv,
        g: overlay.g * a + base.g * inv,
        b: overlay.b * a + base.b * inv,
        a: a + base.a * inv,
    }
}

pub(crate) fn text_input_style(
    theme: &Theme,
    variant: TextFieldVariant,
    focused: bool,
    hovered: bool,
    disabled: bool,
    error: bool,
) -> TextInputStyle {
    match variant {
        TextFieldVariant::Outlined => {
            outlined_text_input_style(theme, focused, hovered, disabled, error)
        }
        TextFieldVariant::Filled => {
            filled_text_input_style(theme, focused, hovered, disabled, error)
        }
    }
}

pub(crate) fn label_color(
    theme: &Theme,
    variant: TextFieldVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    match variant {
        TextFieldVariant::Outlined => outlined_label_color(theme, disabled, error, focused),
        TextFieldVariant::Filled => filled_label_color(theme, hovered, disabled, error, focused),
    }
}

pub(crate) fn supporting_text_color(
    theme: &Theme,
    variant: TextFieldVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    match variant {
        TextFieldVariant::Outlined => {
            outlined_supporting_text_color(theme, disabled, error, focused)
        }
        TextFieldVariant::Filled => {
            filled_supporting_text_color(theme, hovered, disabled, error, focused)
        }
    }
}

pub(crate) fn hover_state_layer(
    theme: &Theme,
    variant: TextFieldVariant,
    error: bool,
) -> Option<(Color, f32)> {
    match variant {
        TextFieldVariant::Outlined => None,
        TextFieldVariant::Filled => {
            let (color_key, opacity_key) = if error {
                (
                    "md.comp.filled-text-field.error.hover.state-layer.color",
                    "md.comp.filled-text-field.error.hover.state-layer.opacity",
                )
            } else {
                (
                    "md.comp.filled-text-field.hover.state-layer.color",
                    "md.comp.filled-text-field.hover.state-layer.opacity",
                )
            };

            let color = theme
                .color_by_key(color_key)
                .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
                .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
            let opacity = theme.number_by_key(opacity_key).unwrap_or(0.08);
            Some((color, opacity))
        }
    }
}

fn outlined_text_input_style(
    theme: &Theme,
    focused: bool,
    hovered: bool,
    disabled: bool,
    error: bool,
) -> TextInputStyle {
    let corner = outlined_container_corner(theme);

    let outline_width = theme
        .metric_by_key("md.comp.outlined-text-field.outline.width")
        .unwrap_or(Px(1.0));
    let hover_width = theme
        .metric_by_key("md.comp.outlined-text-field.hover.outline.width")
        .unwrap_or(outline_width);
    let focus_width = theme
        .metric_by_key("md.comp.outlined-text-field.focus.outline.width")
        .unwrap_or(Px(2.0));
    let disabled_width = theme
        .metric_by_key("md.comp.outlined-text-field.disabled.outline.width")
        .unwrap_or(outline_width);

    let mut style = TextInputStyle::default();
    style.corner_radii = corner;
    style.focus_ring = None;

    style.padding = Edges {
        top: Px(18.0),
        right: Px(16.0),
        bottom: Px(14.0),
        left: Px(16.0),
    };

    let default_bg = theme
        .color_by_key("md.sys.color.surface")
        .unwrap_or_else(|| theme.color_required("md.sys.color.surface"));
    style.background = default_bg;

    let outline_color = outlined_outline_color(theme, hovered, disabled, error, focused);
    let focused_outline_color = outlined_outline_color(theme, false, disabled, error, true);

    let border_width = if disabled {
        disabled_width
    } else if focused {
        focus_width
    } else if hovered {
        hover_width
    } else {
        outline_width
    };
    style.border = Edges::all(border_width);
    style.border_color = outline_color;
    style.border_color_focused = focused_outline_color;

    style.text_color = outlined_input_text_color(theme, hovered, disabled, error, focused);
    style.placeholder_color = theme
        .color_by_key("md.comp.outlined-text-field.input-text.placeholder.color")
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or(style.placeholder_color);
    style.selection_color = theme
        .color_by_key("md.sys.color.primary")
        .map(|c| alpha_mul(c, 0.35))
        .unwrap_or(style.selection_color);
    style.caret_color = outlined_caret_color(theme, disabled, error, focused);
    style.preedit_color = theme
        .color_by_key("md.sys.color.primary")
        .unwrap_or(style.preedit_color);

    if disabled {
        let opacity = theme
            .number_by_key("md.comp.outlined-text-field.disabled.input-text.opacity")
            .unwrap_or(0.38);
        style.text_color = alpha_mul(style.text_color, opacity);
        style.placeholder_color = alpha_mul(style.placeholder_color, opacity);

        let outline_opacity = theme
            .number_by_key("md.comp.outlined-text-field.disabled.outline.opacity")
            .unwrap_or(0.12);
        style.border_color = alpha_mul(style.border_color, outline_opacity);
        style.border_color_focused = alpha_mul(style.border_color_focused, outline_opacity);
    }

    style
}

fn filled_text_input_style(
    theme: &Theme,
    focused: bool,
    hovered: bool,
    disabled: bool,
    error: bool,
) -> TextInputStyle {
    let corner = filled_container_corner(theme);

    let active_height = theme
        .metric_by_key("md.comp.filled-text-field.active-indicator.height")
        .unwrap_or(Px(1.0));
    let hover_height = theme
        .metric_by_key("md.comp.filled-text-field.hover.active-indicator.height")
        .unwrap_or(active_height);
    let focus_height = theme
        .metric_by_key("md.comp.filled-text-field.focus.active-indicator.height")
        .unwrap_or(Px(2.0));
    let disabled_height = theme
        .metric_by_key("md.comp.filled-text-field.disabled.active-indicator.height")
        .unwrap_or(active_height);

    let mut style = TextInputStyle::default();
    style.corner_radii = corner;
    style.focus_ring = None;

    style.padding = Edges {
        top: Px(18.0),
        right: Px(16.0),
        bottom: Px(14.0),
        left: Px(16.0),
    };

    let mut background = theme
        .color_by_key("md.comp.filled-text-field.container.color")
        .or_else(|| theme.color_by_key("md.sys.color.surface-container-highest"))
        .or_else(|| theme.color_by_key("md.sys.color.surface"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.surface"));

    if disabled {
        let overlay = theme
            .color_by_key("md.comp.filled-text-field.disabled.container.color")
            .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
            .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
        let opacity = theme
            .number_by_key("md.comp.filled-text-field.disabled.container.opacity")
            .unwrap_or(0.04);
        background = blend_over(background, overlay, opacity);
    }
    style.background = background;

    let indicator_color = filled_active_indicator_color(theme, hovered, disabled, error, focused);
    let focused_indicator_color =
        filled_active_indicator_color(theme, false, disabled, error, true);

    let bottom = if disabled {
        disabled_height
    } else if focused {
        focus_height
    } else if hovered {
        hover_height
    } else {
        active_height
    };
    style.border = Edges {
        top: Px(0.0),
        right: Px(0.0),
        bottom,
        left: Px(0.0),
    };
    style.border_color = indicator_color;
    style.border_color_focused = focused_indicator_color;

    style.text_color = filled_input_text_color(theme, hovered, disabled, error, focused);
    style.placeholder_color = theme
        .color_by_key("md.comp.filled-text-field.input-text.placeholder.color")
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or(style.placeholder_color);
    style.selection_color = theme
        .color_by_key("md.sys.color.primary")
        .map(|c| alpha_mul(c, 0.35))
        .unwrap_or(style.selection_color);
    style.caret_color = filled_caret_color(theme, disabled, error, focused);
    style.preedit_color = theme
        .color_by_key("md.sys.color.primary")
        .unwrap_or(style.preedit_color);

    if disabled {
        let opacity = theme
            .number_by_key("md.comp.filled-text-field.disabled.input-text.opacity")
            .unwrap_or(0.38);
        style.text_color = alpha_mul(style.text_color, opacity);
        style.placeholder_color = alpha_mul(style.placeholder_color, opacity);
    }

    style
}

fn outlined_caret_color(theme: &Theme, disabled: bool, error: bool, focused: bool) -> Color {
    let base = if error && focused {
        theme
            .color_by_key("md.comp.outlined-text-field.error.focus.caret.color")
            .or_else(|| theme.color_by_key("md.sys.color.error"))
    } else {
        theme
            .color_by_key("md.comp.outlined-text-field.caret.color")
            .or_else(|| theme.color_by_key("md.sys.color.primary"))
    }
    .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));

    if disabled {
        alpha_mul(base, 0.38)
    } else {
        base
    }
}

fn filled_caret_color(theme: &Theme, disabled: bool, error: bool, focused: bool) -> Color {
    let base = if error && focused {
        theme
            .color_by_key("md.comp.filled-text-field.error.focus.caret.color")
            .or_else(|| theme.color_by_key("md.sys.color.error"))
    } else {
        theme
            .color_by_key("md.comp.filled-text-field.caret.color")
            .or_else(|| theme.color_by_key("md.sys.color.primary"))
    }
    .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));

    if disabled {
        alpha_mul(base, 0.38)
    } else {
        base
    }
}

fn outlined_input_text_color(
    theme: &Theme,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    let key = if error && focused {
        "md.comp.outlined-text-field.error.focus.input-text.color"
    } else if error && hovered {
        "md.comp.outlined-text-field.error.hover.input-text.color"
    } else if error {
        "md.comp.outlined-text-field.error.input-text.color"
    } else if focused {
        "md.comp.outlined-text-field.focus.input-text.color"
    } else if hovered {
        "md.comp.outlined-text-field.hover.input-text.color"
    } else {
        "md.comp.outlined-text-field.input-text.color"
    };

    let mut c = theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));

    if disabled {
        let opacity = theme
            .number_by_key("md.comp.outlined-text-field.disabled.input-text.opacity")
            .unwrap_or(0.38);
        c = alpha_mul(c, opacity);
    }

    c
}

fn filled_input_text_color(
    theme: &Theme,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    let key = if error && focused {
        "md.comp.filled-text-field.error.focus.input-text.color"
    } else if error && hovered {
        "md.comp.filled-text-field.error.hover.input-text.color"
    } else if error {
        "md.comp.filled-text-field.error.input-text.color"
    } else if focused {
        "md.comp.filled-text-field.focus.input-text.color"
    } else if hovered {
        "md.comp.filled-text-field.hover.input-text.color"
    } else {
        "md.comp.filled-text-field.input-text.color"
    };

    let mut c = theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));

    if disabled {
        let opacity = theme
            .number_by_key("md.comp.filled-text-field.disabled.input-text.opacity")
            .unwrap_or(0.38);
        c = alpha_mul(c, opacity);
    }

    c
}

fn outlined_outline_color(
    theme: &Theme,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    let key = if error && focused {
        "md.comp.outlined-text-field.error.focus.outline.color"
    } else if error && hovered {
        "md.comp.outlined-text-field.error.hover.outline.color"
    } else if error {
        "md.comp.outlined-text-field.error.outline.color"
    } else if focused {
        "md.comp.outlined-text-field.focus.outline.color"
    } else if hovered {
        "md.comp.outlined-text-field.hover.outline.color"
    } else if disabled {
        "md.comp.outlined-text-field.disabled.outline.color"
    } else {
        "md.comp.outlined-text-field.outline.color"
    };

    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.outline"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.outline"))
}

fn filled_active_indicator_color(
    theme: &Theme,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    let key = if error && focused {
        "md.comp.filled-text-field.error.focus.active-indicator.color"
    } else if error && hovered {
        "md.comp.filled-text-field.error.hover.active-indicator.color"
    } else if error {
        "md.comp.filled-text-field.error.active-indicator.color"
    } else if focused {
        "md.comp.filled-text-field.focus.active-indicator.color"
    } else if hovered {
        "md.comp.filled-text-field.hover.active-indicator.color"
    } else if disabled {
        "md.comp.filled-text-field.disabled.active-indicator.color"
    } else {
        "md.comp.filled-text-field.active-indicator.color"
    };

    let mut c = theme
        .color_by_key(key)
        .unwrap_or(MaterialContentDefaults::on_surface_variant(theme).content_color);

    if disabled {
        let opacity = theme
            .number_by_key("md.comp.filled-text-field.disabled.active-indicator.opacity")
            .unwrap_or(0.38);
        c = alpha_mul(c, opacity);
    }

    c
}

fn outlined_label_color(theme: &Theme, disabled: bool, error: bool, focused: bool) -> Color {
    let key = if error && focused {
        "md.comp.outlined-text-field.error.focus.label-text.color"
    } else if error {
        "md.comp.outlined-text-field.error.label-text.color"
    } else if focused {
        "md.comp.outlined-text-field.focus.label-text.color"
    } else if disabled {
        "md.comp.outlined-text-field.disabled.label-text.color"
    } else {
        "md.comp.outlined-text-field.label-text.color"
    };

    let mut c = theme
        .color_by_key(key)
        .unwrap_or(MaterialContentDefaults::on_surface_variant(theme).content_color);

    if disabled {
        let opacity = theme
            .number_by_key("md.comp.outlined-text-field.disabled.label-text.opacity")
            .unwrap_or(0.38);
        c = alpha_mul(c, opacity);
    }

    c
}

fn filled_label_color(
    theme: &Theme,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    let key = if error && focused {
        "md.comp.filled-text-field.error.focus.label-text.color"
    } else if error && hovered {
        "md.comp.filled-text-field.error.hover.label-text.color"
    } else if error {
        "md.comp.filled-text-field.error.label-text.color"
    } else if focused {
        "md.comp.filled-text-field.focus.label-text.color"
    } else if hovered {
        "md.comp.filled-text-field.hover.label-text.color"
    } else if disabled {
        "md.comp.filled-text-field.disabled.label-text.color"
    } else {
        "md.comp.filled-text-field.label-text.color"
    };

    let mut c = theme
        .color_by_key(key)
        .unwrap_or(MaterialContentDefaults::on_surface_variant(theme).content_color);

    if disabled {
        let opacity = theme
            .number_by_key("md.comp.filled-text-field.disabled.label-text.opacity")
            .unwrap_or(0.38);
        c = alpha_mul(c, opacity);
    }

    c
}

fn outlined_supporting_text_color(
    theme: &Theme,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    let key = if error && focused {
        "md.comp.outlined-text-field.error.focus.supporting-text.color"
    } else if error {
        "md.comp.outlined-text-field.error.supporting-text.color"
    } else if focused {
        "md.comp.outlined-text-field.focus.supporting-text.color"
    } else if disabled {
        "md.comp.outlined-text-field.disabled.supporting-text.color"
    } else {
        "md.comp.outlined-text-field.supporting-text.color"
    };

    let mut c = theme
        .color_by_key(key)
        .unwrap_or(MaterialContentDefaults::on_surface_variant(theme).content_color);

    if disabled {
        let opacity = theme
            .number_by_key("md.comp.outlined-text-field.disabled.supporting-text.opacity")
            .unwrap_or(0.38);
        c = alpha_mul(c, opacity);
    }

    c
}

fn filled_supporting_text_color(
    theme: &Theme,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    let key = if error && focused {
        "md.comp.filled-text-field.error.focus.supporting-text.color"
    } else if error && hovered {
        "md.comp.filled-text-field.error.hover.supporting-text.color"
    } else if error {
        "md.comp.filled-text-field.error.supporting-text.color"
    } else if focused {
        "md.comp.filled-text-field.focus.supporting-text.color"
    } else if hovered {
        "md.comp.filled-text-field.hover.supporting-text.color"
    } else if disabled {
        "md.comp.filled-text-field.disabled.supporting-text.color"
    } else {
        "md.comp.filled-text-field.supporting-text.color"
    };

    let mut c = theme
        .color_by_key(key)
        .unwrap_or(MaterialContentDefaults::on_surface_variant(theme).content_color);

    if disabled {
        let opacity = theme
            .number_by_key("md.comp.filled-text-field.disabled.supporting-text.opacity")
            .unwrap_or(0.38);
        c = alpha_mul(c, opacity);
    }

    c
}
