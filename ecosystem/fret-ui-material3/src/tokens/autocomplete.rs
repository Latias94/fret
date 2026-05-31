//! Typed token access for Material 3 autocomplete.
//!
//! This module centralizes token key mapping and fallback chains so autocomplete visuals remain
//! stable and drift-resistant during refactors.

use fret_core::{Color, Corners, Edges, Px, TextStyle};
use fret_ui::{TextInputStyle, Theme};
use fret_ui_kit::typography::{self, TextIntent};

use crate::foundation::token_resolver::{MaterialTokenResolver, alpha_mul, blend_over};
use crate::text_field::TextFieldVariant;
use crate::tokens::{selectable_menu_item as selectable_item_tokens, shape};

pub(crate) fn text_field_container_height(theme: &Theme, variant: TextFieldVariant) -> Px {
    let key = match variant {
        TextFieldVariant::Outlined => "md.comp.outlined-autocomplete.text-field.container.height",
        TextFieldVariant::Filled => "md.comp.filled-autocomplete.text-field.container.height",
    };
    theme.metric_by_key(key).unwrap_or(Px(56.0))
}

pub(crate) fn text_field_container_shape(theme: &Theme, variant: TextFieldVariant) -> Corners {
    let key = match variant {
        TextFieldVariant::Outlined => "md.comp.outlined-autocomplete.text-field.container.shape",
        TextFieldVariant::Filled => "md.comp.filled-autocomplete.text-field.container.shape",
    };
    shape::corners_or_metric(theme, key)
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.extra-small"))
        .unwrap_or_else(|| Corners::all(Px(4.0)))
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

pub(crate) fn trailing_icon_size(theme: &Theme, variant: TextFieldVariant) -> Px {
    let key = match variant {
        TextFieldVariant::Outlined => "md.comp.outlined-autocomplete.text-field.trailing-icon.size",
        TextFieldVariant::Filled => "md.comp.filled-autocomplete.text-field.trailing-icon.size",
    };
    theme.metric_by_key(key).unwrap_or(Px(24.0))
}

pub(crate) fn leading_icon_size(theme: &Theme, variant: TextFieldVariant) -> Px {
    let key = match variant {
        TextFieldVariant::Outlined => "md.comp.outlined-autocomplete.text-field.leading-icon.size",
        TextFieldVariant::Filled => "md.comp.filled-autocomplete.text-field.leading-icon.size",
    };
    theme.metric_by_key(key).unwrap_or(Px(24.0))
}

pub(crate) fn leading_icon_color(
    theme: &Theme,
    variant: TextFieldVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> (Color, f32) {
    let (color_key, opacity_key) = if disabled {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.disabled.leading-icon.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.disabled.leading-icon.color"
                }
            },
            Some(match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.disabled.leading-icon.opacity"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.disabled.leading-icon.opacity"
                }
            }),
        )
    } else if error && focused {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.error.focus.leading-icon.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.error.focus.leading-icon.color"
                }
            },
            None,
        )
    } else if error && hovered {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.error.hover.leading-icon.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.error.hover.leading-icon.color"
                }
            },
            None,
        )
    } else if error {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.error.leading-icon.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.error.leading-icon.color"
                }
            },
            None,
        )
    } else if focused {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.focus.leading-icon.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.focus.leading-icon.color"
                }
            },
            None,
        )
    } else if hovered {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.hover.leading-icon.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.hover.leading-icon.color"
                }
            },
            None,
        )
    } else {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.leading-icon.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.leading-icon.color"
                }
            },
            None,
        )
    };

    MaterialTokenResolver::new(theme).color_comp_or_sys_with_opacity(
        color_key,
        "md.sys.color.on-surface-variant",
        opacity_key,
        1.0,
    )
}

pub(crate) fn trailing_icon_color(
    theme: &Theme,
    variant: TextFieldVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> (Color, f32) {
    let (color_key, opacity_key) = if disabled {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.disabled.trailing-icon.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.disabled.trailing-icon.color"
                }
            },
            Some(match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.disabled.trailing-icon.opacity"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.disabled.trailing-icon.opacity"
                }
            }),
        )
    } else if error && focused {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.error.focus.trailing-icon.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.error.focus.trailing-icon.color"
                }
            },
            None,
        )
    } else if error && hovered {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.error.hover.trailing-icon.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.error.hover.trailing-icon.color"
                }
            },
            None,
        )
    } else if error {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.error.trailing-icon.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.error.trailing-icon.color"
                }
            },
            None,
        )
    } else if focused {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.focus.trailing-icon.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.focus.trailing-icon.color"
                }
            },
            None,
        )
    } else if hovered {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.hover.trailing-icon.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.hover.trailing-icon.color"
                }
            },
            None,
        )
    } else {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.trailing-icon.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.trailing-icon.color"
                }
            },
            None,
        )
    };

    MaterialTokenResolver::new(theme).color_comp_or_sys_with_opacity(
        color_key,
        "md.sys.color.on-surface-variant",
        opacity_key,
        1.0,
    )
}

pub(crate) fn hover_state_layer(
    theme: &Theme,
    variant: TextFieldVariant,
    error: bool,
) -> Option<(Color, f32)> {
    let (color_key, opacity_key) = match variant {
        TextFieldVariant::Outlined => (
            "md.comp.outlined-autocomplete.text-field.hover.state-layer.color",
            if error {
                "md.comp.outlined-autocomplete.text-field.error.hover.state-layer.opacity"
            } else {
                "md.comp.outlined-autocomplete.text-field.hover.state-layer.opacity"
            },
        ),
        TextFieldVariant::Filled => (
            "md.comp.filled-autocomplete.text-field.hover.state-layer.color",
            if error {
                "md.comp.filled-autocomplete.text-field.error.hover.state-layer.opacity"
            } else {
                "md.comp.filled-autocomplete.text-field.hover.state-layer.opacity"
            },
        ),
    };

    let (color, opacity) = MaterialTokenResolver::new(theme).color_comp_or_sys_with_opacity(
        color_key,
        "md.sys.color.on-surface",
        Some(opacity_key),
        0.08,
    );
    Some((color, opacity))
}

pub(crate) fn label_color(
    theme: &Theme,
    variant: TextFieldVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    let (color_key, opacity_key) = if disabled {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.disabled.label-text.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.disabled.label-text.color"
                }
            },
            Some(match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.disabled.label-text.opacity"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.disabled.label-text.opacity"
                }
            }),
        )
    } else if error && focused {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.error.focus.label-text.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.error.focus.label-text.color"
                }
            },
            None,
        )
    } else if error && hovered {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.error.hover.label-text.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.error.hover.label-text.color"
                }
            },
            None,
        )
    } else if error {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.error.label-text.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.error.label-text.color"
                }
            },
            None,
        )
    } else if focused {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.focus.label-text.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.focus.label-text.color"
                }
            },
            None,
        )
    } else if hovered {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.hover.label-text.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.hover.label-text.color"
                }
            },
            None,
        )
    } else {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.label-text.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.label-text.color"
                }
            },
            None,
        )
    };

    let (color, opacity) = MaterialTokenResolver::new(theme).color_comp_or_sys_with_opacity(
        color_key,
        "md.sys.color.on-surface-variant",
        opacity_key,
        1.0,
    );
    alpha_mul(color, opacity)
}

pub(crate) fn supporting_text_color(
    theme: &Theme,
    variant: TextFieldVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    let (color_key, opacity_key) = if disabled {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.disabled.supporting-text.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.disabled.supporting-text.color"
                }
            },
            Some(match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.disabled.supporting-text.opacity"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.disabled.supporting-text.opacity"
                }
            }),
        )
    } else if error && focused {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.error.focus.supporting-text.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.error.focus.supporting-text.color"
                }
            },
            None,
        )
    } else if error && hovered {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.error.hover.supporting-text.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.error.hover.supporting-text.color"
                }
            },
            None,
        )
    } else if error {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.error.supporting-text.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.error.supporting-text.color"
                }
            },
            None,
        )
    } else if focused {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.focus.supporting-text.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.focus.supporting-text.color"
                }
            },
            None,
        )
    } else if hovered {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.hover.supporting-text.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.hover.supporting-text.color"
                }
            },
            None,
        )
    } else {
        (
            match variant {
                TextFieldVariant::Outlined => {
                    "md.comp.outlined-autocomplete.text-field.supporting-text.color"
                }
                TextFieldVariant::Filled => {
                    "md.comp.filled-autocomplete.text-field.supporting-text.color"
                }
            },
            None,
        )
    };

    let (color, opacity) = MaterialTokenResolver::new(theme).color_comp_or_sys_with_opacity(
        color_key,
        "md.sys.color.on-surface-variant",
        opacity_key,
        1.0,
    );
    alpha_mul(color, opacity)
}

fn outlined_text_input_style(
    theme: &Theme,
    focused: bool,
    hovered: bool,
    disabled: bool,
    error: bool,
) -> TextInputStyle {
    let corner = text_field_container_shape(theme, TextFieldVariant::Outlined);

    let outline_width = theme
        .metric_by_key("md.comp.outlined-autocomplete.text-field.outline.width")
        .unwrap_or(Px(1.0));
    let hover_width = theme
        .metric_by_key("md.comp.outlined-autocomplete.text-field.hover.outline.width")
        .unwrap_or(outline_width);
    let focus_width = theme
        .metric_by_key("md.comp.outlined-autocomplete.text-field.focus.outline.width")
        .unwrap_or(Px(2.0));
    let disabled_width = theme
        .metric_by_key("md.comp.outlined-autocomplete.text-field.disabled.outline.width")
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

    style.background = MaterialTokenResolver::new(theme).color_comp_or_sys(
        "md.comp.outlined-autocomplete.text-field.container.color",
        "md.sys.color.surface",
    );

    let (border_width, border_color, border_color_focused) = outlined_outline(
        theme,
        hovered,
        disabled,
        error,
        focused,
        outline_width,
        hover_width,
        focus_width,
        disabled_width,
    );
    style.border = Edges::all(border_width);
    style.border_color = border_color;
    style.border_color_focused = border_color_focused;

    style.text_color = outlined_input_text_color(theme, hovered, disabled, error, focused);
    style.placeholder_color = theme
        .color_by_key("md.sys.color.on-surface-variant")
        .unwrap_or(style.placeholder_color);
    style.selection_color = theme
        .color_by_key("md.sys.color.primary")
        .map(|c| alpha_mul(c, 0.35))
        .unwrap_or(style.selection_color);
    style.caret_color = outlined_caret_color(theme, disabled, error, focused);
    style.preedit_color = theme
        .color_by_key("md.sys.color.primary")
        .unwrap_or(style.preedit_color);
    style.preedit_underline_color = style.preedit_color;

    if disabled {
        let opacity = MaterialTokenResolver::new(theme).number_optional(
            Some("md.comp.outlined-autocomplete.text-field.disabled.input-text.opacity"),
            0.38,
        );
        style.text_color = alpha_mul(style.text_color, opacity);
        style.placeholder_color = alpha_mul(style.placeholder_color, opacity);

        let outline_opacity = MaterialTokenResolver::new(theme).number_optional(
            Some("md.comp.outlined-autocomplete.text-field.disabled.outline.opacity"),
            0.12,
        );
        style.border_color = alpha_mul(style.border_color, outline_opacity);
        style.border_color_focused = alpha_mul(style.border_color_focused, outline_opacity);
    }

    style
}

fn outlined_outline(
    theme: &Theme,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
    outline_width: Px,
    hover_width: Px,
    focus_width: Px,
    disabled_width: Px,
) -> (Px, Color, Color) {
    let border_width = if disabled {
        disabled_width
    } else if focused {
        focus_width
    } else if hovered {
        hover_width
    } else {
        outline_width
    };

    let color_key = if error && focused {
        "md.comp.outlined-autocomplete.text-field.error.focus.outline.color"
    } else if error && hovered {
        "md.comp.outlined-autocomplete.text-field.error.hover.outline.color"
    } else if error {
        "md.comp.outlined-autocomplete.text-field.error.outline.color"
    } else if focused {
        "md.comp.outlined-autocomplete.text-field.focus.outline.color"
    } else if hovered {
        "md.comp.outlined-autocomplete.text-field.hover.outline.color"
    } else {
        "md.comp.outlined-autocomplete.text-field.outline.color"
    };

    let focused_color_key = if error {
        "md.comp.outlined-autocomplete.text-field.error.focus.outline.color"
    } else {
        "md.comp.outlined-autocomplete.text-field.focus.outline.color"
    };

    let tokens = MaterialTokenResolver::new(theme);
    let border_color = tokens.color_comp_or_sys(color_key, "md.sys.color.outline");
    let border_color_focused = tokens.color_comp_or_sys(focused_color_key, "md.sys.color.outline");
    (border_width, border_color, border_color_focused)
}

fn outlined_input_text_color(
    theme: &Theme,
    _hovered: bool,
    _disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    let key = if error && focused {
        "md.comp.outlined-autocomplete.text-field.error.focus.input-text.color"
    } else if error {
        "md.comp.outlined-autocomplete.text-field.error.input-text.color"
    } else if focused {
        "md.comp.outlined-autocomplete.text-field.focus.input-text.color"
    } else {
        "md.comp.outlined-autocomplete.text-field.input-text.color"
    };
    MaterialTokenResolver::new(theme).color_comp_or_sys(key, "md.sys.color.on-surface")
}

fn outlined_caret_color(theme: &Theme, _disabled: bool, error: bool, focused: bool) -> Color {
    let key = if error && focused {
        "md.comp.outlined-autocomplete.text-field.error.focus.caret.color"
    } else {
        "md.comp.outlined-autocomplete.text-field.caret.color"
    };
    MaterialTokenResolver::new(theme).color_comp_or_sys(key, "md.sys.color.primary")
}

fn filled_text_input_style(
    theme: &Theme,
    focused: bool,
    hovered: bool,
    disabled: bool,
    error: bool,
) -> TextInputStyle {
    let corner = text_field_container_shape(theme, TextFieldVariant::Filled);

    let active_height = theme
        .metric_by_key("md.comp.filled-autocomplete.text-field.active-indicator.height")
        .unwrap_or(Px(1.0));
    let hover_height = theme
        .metric_by_key("md.comp.filled-autocomplete.text-field.hover.active-indicator.height")
        .unwrap_or(active_height);
    let focus_height = theme
        .metric_by_key("md.comp.filled-autocomplete.text-field.focus.active-indicator.height")
        .unwrap_or(Px(2.0));
    let disabled_height = theme
        .metric_by_key("md.comp.filled-autocomplete.text-field.disabled.active-indicator.height")
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

    let tokens = MaterialTokenResolver::new(theme);
    let mut background = tokens.color_comp_or_sys_chain(
        "md.comp.filled-autocomplete.text-field.container.color",
        &[
            "md.sys.color.surface-container-highest",
            "md.sys.color.surface",
        ],
    );

    if disabled {
        let (overlay, opacity) = tokens.color_comp_or_sys_with_opacity(
            "md.comp.filled-autocomplete.text-field.disabled.container.color",
            "md.sys.color.on-surface",
            Some("md.comp.filled-autocomplete.text-field.disabled.container.opacity"),
            0.04,
        );
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
        .color_by_key("md.sys.color.on-surface-variant")
        .unwrap_or(style.placeholder_color);
    style.selection_color = theme
        .color_by_key("md.sys.color.primary")
        .map(|c| alpha_mul(c, 0.35))
        .unwrap_or(style.selection_color);
    style.caret_color = filled_caret_color(theme, disabled, error, focused);
    style.preedit_color = theme
        .color_by_key("md.sys.color.primary")
        .unwrap_or(style.preedit_color);
    style.preedit_underline_color = style.preedit_color;

    if disabled {
        let opacity = MaterialTokenResolver::new(theme).number_optional(
            Some("md.comp.filled-autocomplete.text-field.disabled.input-text.opacity"),
            0.38,
        );
        style.text_color = alpha_mul(style.text_color, opacity);
        style.placeholder_color = alpha_mul(style.placeholder_color, opacity);
    }

    style
}

fn filled_active_indicator_color(
    theme: &Theme,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    let key = if disabled {
        "md.comp.filled-autocomplete.text-field.disabled.active-indicator.color"
    } else if error && focused {
        "md.comp.filled-autocomplete.text-field.error.focus.active-indicator.color"
    } else if error && hovered {
        "md.comp.filled-autocomplete.text-field.error.hover.active-indicator.color"
    } else if error {
        "md.comp.filled-autocomplete.text-field.error.active-indicator.color"
    } else if focused {
        "md.comp.filled-autocomplete.text-field.focus.active-indicator.color"
    } else if hovered {
        "md.comp.filled-autocomplete.text-field.hover.active-indicator.color"
    } else {
        "md.comp.filled-autocomplete.text-field.active-indicator.color"
    };
    let tokens = MaterialTokenResolver::new(theme);
    let mut color = tokens.color_comp_or_sys(key, "md.sys.color.on-surface");

    if disabled {
        let opacity = tokens.number_optional(
            Some("md.comp.filled-autocomplete.text-field.disabled.active-indicator.opacity"),
            0.38,
        );
        color = alpha_mul(color, opacity);
    }
    color
}

fn filled_input_text_color(
    theme: &Theme,
    _hovered: bool,
    _disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    let key = if error && focused {
        "md.comp.filled-autocomplete.text-field.error.focus.input-text.color"
    } else if error {
        "md.comp.filled-autocomplete.text-field.error.input-text.color"
    } else if focused {
        "md.comp.filled-autocomplete.text-field.focus.input-text.color"
    } else {
        "md.comp.filled-autocomplete.text-field.input-text.color"
    };
    MaterialTokenResolver::new(theme).color_comp_or_sys(key, "md.sys.color.on-surface")
}

fn filled_caret_color(theme: &Theme, _disabled: bool, error: bool, focused: bool) -> Color {
    let key = if error && focused {
        "md.comp.filled-autocomplete.text-field.error.focus.caret.color"
    } else {
        "md.comp.filled-autocomplete.text-field.caret.color"
    };
    MaterialTokenResolver::new(theme).color_comp_or_sys(key, "md.sys.color.primary")
}

pub(crate) fn menu_container_background(theme: &Theme, variant: TextFieldVariant) -> Color {
    let key = match variant {
        TextFieldVariant::Outlined => "md.comp.outlined-autocomplete.menu.container.color",
        TextFieldVariant::Filled => "md.comp.filled-autocomplete.menu.container.color",
    };
    MaterialTokenResolver::new(theme).color_comp_or_sys(key, "md.sys.color.surface-container")
}

pub(crate) fn menu_container_elevation(theme: &Theme, variant: TextFieldVariant) -> Px {
    let key = match variant {
        TextFieldVariant::Outlined => "md.comp.outlined-autocomplete.menu.container.elevation",
        TextFieldVariant::Filled => "md.comp.filled-autocomplete.menu.container.elevation",
    };
    theme.metric_by_key(key).unwrap_or(Px(3.0))
}

pub(crate) fn menu_container_shadow_color(theme: &Theme, variant: TextFieldVariant) -> Color {
    let key = match variant {
        TextFieldVariant::Outlined => "md.comp.outlined-autocomplete.menu.container.shadow-color",
        TextFieldVariant::Filled => "md.comp.filled-autocomplete.menu.container.shadow-color",
    };
    MaterialTokenResolver::new(theme).color_comp_or_sys(key, "md.sys.color.shadow")
}

pub(crate) fn menu_container_shape(theme: &Theme, variant: TextFieldVariant) -> Corners {
    let key = match variant {
        TextFieldVariant::Outlined => "md.comp.outlined-autocomplete.menu.container.shape",
        TextFieldVariant::Filled => "md.comp.filled-autocomplete.menu.container.shape",
    };
    shape::corners_or_metric(theme, key)
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.extra-small"))
        .unwrap_or_else(|| Corners::all(Px(4.0)))
}

pub(crate) fn menu_list_item_height(theme: &Theme, variant: TextFieldVariant) -> Px {
    let key = match variant {
        TextFieldVariant::Outlined => {
            "md.comp.outlined-autocomplete.menu.list-item.container.height"
        }
        TextFieldVariant::Filled => "md.comp.filled-autocomplete.menu.list-item.container.height",
    };
    theme.metric_by_key(key).unwrap_or(Px(48.0))
}

pub(crate) fn menu_selectable_item_outer_horizontal_padding(
    theme: &Theme,
    _variant: TextFieldVariant,
) -> Px {
    selectable_item_tokens::outer_horizontal_padding(theme)
}

pub(crate) fn menu_selectable_item_outer_vertical_padding(
    theme: &Theme,
    _variant: TextFieldVariant,
    has_secondary_text: bool,
) -> Px {
    selectable_item_tokens::outer_vertical_padding(theme, has_secondary_text)
}

pub(crate) fn menu_list_item_content_horizontal_padding(
    theme: &Theme,
    _variant: TextFieldVariant,
) -> Px {
    selectable_item_tokens::content_horizontal_padding(theme)
}

pub(crate) fn menu_list_item_container_shape(
    theme: &Theme,
    _variant: TextFieldVariant,
    selected: bool,
) -> Corners {
    selectable_item_tokens::container_shape(theme, selected)
}

pub(crate) fn menu_list_item_label_text_style(
    theme: &Theme,
    _variant: TextFieldVariant,
) -> Option<TextStyle> {
    theme
        .text_style_by_key("md.sys.typescale.label-large")
        .map(|style| typography::with_intent(style, TextIntent::Control))
}

pub(crate) fn menu_list_item_label_text_color(
    theme: &Theme,
    variant: TextFieldVariant,
    enabled: bool,
    selected: bool,
) -> Color {
    if let Some(label) =
        selectable_item_tokens::selected_or_disabled_label_color(theme, selected, enabled)
    {
        return label;
    }

    let key = match variant {
        TextFieldVariant::Outlined => {
            "md.comp.outlined-autocomplete.menu.list-item.label-text.color"
        }
        TextFieldVariant::Filled => "md.comp.filled-autocomplete.menu.list-item.label-text.color",
    };
    MaterialTokenResolver::new(theme).color_comp_or_sys(key, "md.sys.color.on-surface")
}

pub(crate) fn menu_list_item_selected_container_color(
    theme: &Theme,
    variant: TextFieldVariant,
) -> Color {
    let key = match variant {
        TextFieldVariant::Outlined => {
            "md.comp.outlined-autocomplete.menu.list-item.selected.container.color"
        }
        TextFieldVariant::Filled => {
            "md.comp.filled-autocomplete.menu.list-item.selected.container.color"
        }
    };
    MaterialTokenResolver::new(theme)
        .color_comp_or_sys(key, "md.sys.color.surface-container-highest")
}
