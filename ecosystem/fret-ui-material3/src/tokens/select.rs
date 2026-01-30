//!
//! Centralized token key mapping and fallback chains for Material 3 Select.

use fret_core::{Color, Corners, Px, TextStyle};
use fret_ui::Theme;

use crate::select::SelectVariant;

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn container_height_key(variant: SelectVariant) -> &'static str {
    match variant {
        SelectVariant::Outlined => "md.comp.outlined-select.text-field.container.height",
        SelectVariant::Filled => "md.comp.filled-select.text-field.container.height",
    }
}

pub(crate) fn container_height(theme: &Theme, variant: SelectVariant) -> Px {
    theme
        .metric_by_key(container_height_key(variant))
        .unwrap_or(Px(56.0))
}

fn outlined_container_corner(theme: &Theme) -> Corners {
    theme
        .corners_by_key("md.comp.outlined-select.text-field.container.shape")
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.extra-small"))
        .unwrap_or_else(|| Corners::all(Px(4.0)))
}

fn filled_container_corner(theme: &Theme) -> Corners {
    if let Some(corners) = theme.corners_by_key("md.comp.filled-select.text-field.container.shape")
    {
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

pub(crate) fn container_corner(theme: &Theme, variant: SelectVariant) -> Corners {
    match variant {
        SelectVariant::Outlined => outlined_container_corner(theme),
        SelectVariant::Filled => filled_container_corner(theme),
    }
}

pub(crate) fn container_background(theme: &Theme, variant: SelectVariant, disabled: bool) -> Color {
    let key = match variant {
        SelectVariant::Outlined => "md.comp.outlined-select.text-field.container.color",
        SelectVariant::Filled => {
            if disabled {
                "md.comp.filled-select.text-field.disabled.container.color"
            } else {
                "md.comp.filled-select.text-field.container.color"
            }
        }
    };
    let mut color = theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.surface-container-highest"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.surface-container-highest"));

    if disabled {
        let opacity_key = match variant {
            SelectVariant::Outlined => None,
            SelectVariant::Filled => {
                Some("md.comp.filled-select.text-field.disabled.container.opacity")
            }
        };
        if let Some(opacity_key) = opacity_key {
            let opacity = theme.number_by_key(opacity_key).unwrap_or(0.04);
            color = alpha_mul(color, opacity);
        }
    }

    color
}

pub(crate) fn hover_state_layer(
    theme: &Theme,
    variant: SelectVariant,
    error: bool,
) -> (Color, f32) {
    let (color_key, opacity_key) = match variant {
        SelectVariant::Outlined => (
            "md.comp.outlined-select.text-field.hover.state-layer.color",
            if error {
                "md.comp.outlined-select.text-field.error.hover.state-layer.opacity"
            } else {
                "md.comp.outlined-select.text-field.hover.state-layer.opacity"
            },
        ),
        SelectVariant::Filled => (
            "md.comp.filled-select.text-field.hover.state-layer.color",
            if error {
                "md.comp.filled-select.text-field.error.hover.state-layer.opacity"
            } else {
                "md.comp.filled-select.text-field.hover.state-layer.opacity"
            },
        ),
    };

    let color = theme
        .color_by_key(color_key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
    let opacity = theme.number_by_key(opacity_key).unwrap_or(0.08);
    (color, opacity)
}

pub(crate) fn outline(
    theme: &Theme,
    variant: SelectVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Option<(Px, Color, f32)> {
    if variant != SelectVariant::Outlined {
        return None;
    }

    let (width_key, color_key, opacity_key) = if disabled {
        (
            "md.comp.outlined-select.text-field.disabled.outline.width",
            "md.comp.outlined-select.text-field.disabled.outline.color",
            Some("md.comp.outlined-select.text-field.disabled.outline.opacity"),
        )
    } else if error && focused {
        (
            "md.comp.outlined-select.text-field.focus.outline.width",
            "md.comp.outlined-select.text-field.error.focus.outline.color",
            None,
        )
    } else if error && hovered {
        (
            "md.comp.outlined-select.text-field.hover.outline.width",
            "md.comp.outlined-select.text-field.error.hover.outline.color",
            None,
        )
    } else if error {
        (
            "md.comp.outlined-select.text-field.outline.width",
            "md.comp.outlined-select.text-field.error.outline.color",
            None,
        )
    } else if focused {
        (
            "md.comp.outlined-select.text-field.focus.outline.width",
            "md.comp.outlined-select.text-field.focus.outline.color",
            None,
        )
    } else if hovered {
        (
            "md.comp.outlined-select.text-field.hover.outline.width",
            "md.comp.outlined-select.text-field.hover.outline.color",
            None,
        )
    } else {
        (
            "md.comp.outlined-select.text-field.outline.width",
            "md.comp.outlined-select.text-field.outline.color",
            None,
        )
    };

    let width = theme.metric_by_key(width_key).unwrap_or(Px(1.0));
    let color = theme
        .color_by_key(color_key)
        .or_else(|| theme.color_by_key("md.sys.color.outline"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.outline"));
    let opacity = opacity_key
        .and_then(|k| theme.number_by_key(k))
        .unwrap_or(1.0);
    Some((width, color, opacity))
}

pub(crate) fn active_indicator(
    theme: &Theme,
    variant: SelectVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Option<(Px, Color, f32)> {
    if variant != SelectVariant::Filled {
        return None;
    }

    let (height_key, color_key, opacity_key) = if disabled {
        (
            "md.comp.filled-select.text-field.disabled.active-indicator.height",
            "md.comp.filled-select.text-field.disabled.active-indicator.color",
            Some("md.comp.filled-select.text-field.disabled.active-indicator.opacity"),
        )
    } else if error && focused {
        (
            "md.comp.filled-select.text-field.focus.active-indicator.height",
            "md.comp.filled-select.text-field.error.focus.active-indicator.color",
            None,
        )
    } else if error && hovered {
        (
            "md.comp.filled-select.text-field.hover.active-indicator.height",
            "md.comp.filled-select.text-field.error.hover.active-indicator.color",
            None,
        )
    } else if error {
        (
            "md.comp.filled-select.text-field.active-indicator.height",
            "md.comp.filled-select.text-field.error.active-indicator.color",
            None,
        )
    } else if focused {
        (
            "md.comp.filled-select.text-field.focus.active-indicator.height",
            "md.comp.filled-select.text-field.focus.active-indicator.color",
            None,
        )
    } else if hovered {
        (
            "md.comp.filled-select.text-field.hover.active-indicator.height",
            "md.comp.filled-select.text-field.hover.active-indicator.color",
            None,
        )
    } else {
        (
            "md.comp.filled-select.text-field.active-indicator.height",
            "md.comp.filled-select.text-field.active-indicator.color",
            None,
        )
    };

    let height = theme.metric_by_key(height_key).unwrap_or(Px(1.0));
    let color = theme
        .color_by_key(color_key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface-variant"));
    let opacity = opacity_key
        .and_then(|k| theme.number_by_key(k))
        .unwrap_or(1.0);
    Some((height, color, opacity))
}

pub(crate) fn input_text_style(theme: &Theme, variant: SelectVariant) -> Option<TextStyle> {
    let _ = variant;
    // Material Web v30 models Select typography via `*.font/size/weight/tracking/line-height` and
    // a `*.type` mixin token (not a scalar key). For now, use the canonical sys typescale and keep
    // component-specific typography as a future import step.
    theme.text_style_by_key("md.sys.typescale.body-large")
}

pub(crate) fn input_text_color(
    theme: &Theme,
    variant: SelectVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> (Color, f32) {
    let (color_key, opacity_key) = if disabled {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.disabled.input-text.color"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.disabled.input-text.color"
                }
            },
            Some(match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.disabled.input-text.opacity"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.disabled.input-text.opacity"
                }
            }),
        )
    } else if error && focused {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.error.focus.input-text.color"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.error.focus.input-text.color"
                }
            },
            None,
        )
    } else if error && hovered {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.error.hover.input-text.color"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.error.hover.input-text.color"
                }
            },
            None,
        )
    } else if error {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.error.input-text.color"
                }
                SelectVariant::Filled => "md.comp.filled-select.text-field.error.input-text.color",
            },
            None,
        )
    } else if focused {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.focus.input-text.color"
                }
                SelectVariant::Filled => "md.comp.filled-select.text-field.focus.input-text.color",
            },
            None,
        )
    } else if hovered {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.hover.input-text.color"
                }
                SelectVariant::Filled => "md.comp.filled-select.text-field.hover.input-text.color",
            },
            None,
        )
    } else {
        (
            match variant {
                SelectVariant::Outlined => "md.comp.outlined-select.text-field.input-text.color",
                SelectVariant::Filled => "md.comp.filled-select.text-field.input-text.color",
            },
            None,
        )
    };

    let color = theme
        .color_by_key(color_key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"));
    let opacity = opacity_key
        .and_then(|k| theme.number_by_key(k))
        .unwrap_or(1.0);
    (color, opacity)
}

pub(crate) fn trailing_icon_size(theme: &Theme, variant: SelectVariant) -> Px {
    let key = match variant {
        SelectVariant::Outlined => "md.comp.outlined-select.text-field.trailing-icon.size",
        SelectVariant::Filled => "md.comp.filled-select.text-field.trailing-icon.size",
    };
    theme.metric_by_key(key).unwrap_or(Px(24.0))
}

pub(crate) fn trailing_icon_color(
    theme: &Theme,
    variant: SelectVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> (Color, f32) {
    let (color_key, opacity_key) = if disabled {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.disabled.trailing-icon.color"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.disabled.trailing-icon.color"
                }
            },
            Some(match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.disabled.trailing-icon.opacity"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.disabled.trailing-icon.opacity"
                }
            }),
        )
    } else if error && focused {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.error.focus.trailing-icon.color"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.error.focus.trailing-icon.color"
                }
            },
            None,
        )
    } else if error && hovered {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.error.hover.trailing-icon.color"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.error.hover.trailing-icon.color"
                }
            },
            None,
        )
    } else if error {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.error.trailing-icon.color"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.error.trailing-icon.color"
                }
            },
            None,
        )
    } else if focused {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.focus.trailing-icon.color"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.focus.trailing-icon.color"
                }
            },
            None,
        )
    } else if hovered {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.hover.trailing-icon.color"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.hover.trailing-icon.color"
                }
            },
            None,
        )
    } else {
        (
            match variant {
                SelectVariant::Outlined => "md.comp.outlined-select.text-field.trailing-icon.color",
                SelectVariant::Filled => "md.comp.filled-select.text-field.trailing-icon.color",
            },
            None,
        )
    };

    let color = theme
        .color_by_key(color_key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface-variant"));
    let opacity = opacity_key
        .and_then(|k| theme.number_by_key(k))
        .unwrap_or(1.0);
    (color, opacity)
}

pub(crate) fn placeholder_color(
    theme: &Theme,
    variant: SelectVariant,
    disabled: bool,
    _error: bool,
) -> Color {
    let base = theme
        .color_by_key("md.sys.color.on-surface-variant")
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface-variant"));

    if !disabled {
        return base;
    }

    let opacity_key = match variant {
        SelectVariant::Outlined => "md.comp.outlined-select.text-field.disabled.input-text.opacity",
        SelectVariant::Filled => "md.comp.filled-select.text-field.disabled.input-text.opacity",
    };

    alpha_mul(base, theme.number_by_key(opacity_key).unwrap_or(0.38))
}

pub(crate) fn label_color(
    theme: &Theme,
    variant: SelectVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    let (color_key, opacity_key) = if disabled {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.disabled.label-text.color"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.disabled.label-text.color"
                }
            },
            Some(match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.disabled.label-text.opacity"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.disabled.label-text.opacity"
                }
            }),
        )
    } else if error && focused {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.error.focus.label-text.color"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.error.focus.label-text.color"
                }
            },
            None,
        )
    } else if error && hovered {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.error.hover.label-text.color"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.error.hover.label-text.color"
                }
            },
            None,
        )
    } else if error {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.error.label-text.color"
                }
                SelectVariant::Filled => "md.comp.filled-select.text-field.error.label-text.color",
            },
            None,
        )
    } else if focused {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.focus.label-text.color"
                }
                SelectVariant::Filled => "md.comp.filled-select.text-field.focus.label-text.color",
            },
            None,
        )
    } else if hovered {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.hover.label-text.color"
                }
                SelectVariant::Filled => "md.comp.filled-select.text-field.hover.label-text.color",
            },
            None,
        )
    } else {
        (
            match variant {
                SelectVariant::Outlined => "md.comp.outlined-select.text-field.label-text.color",
                SelectVariant::Filled => "md.comp.filled-select.text-field.label-text.color",
            },
            None,
        )
    };

    let color = theme
        .color_by_key(color_key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface-variant"));

    let opacity = opacity_key
        .and_then(|k| theme.number_by_key(k))
        .unwrap_or(1.0);
    alpha_mul(color, opacity)
}

pub(crate) fn supporting_text_color(
    theme: &Theme,
    variant: SelectVariant,
    hovered: bool,
    disabled: bool,
    error: bool,
    focused: bool,
) -> Color {
    let (color_key, opacity_key) = if disabled {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.disabled.supporting-text.color"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.disabled.supporting-text.color"
                }
            },
            Some(match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.disabled.supporting-text.opacity"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.disabled.supporting-text.opacity"
                }
            }),
        )
    } else if error && focused {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.error.focus.supporting-text.color"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.error.focus.supporting-text.color"
                }
            },
            None,
        )
    } else if error && hovered {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.error.hover.supporting-text.color"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.error.hover.supporting-text.color"
                }
            },
            None,
        )
    } else if error {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.error.supporting-text.color"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.error.supporting-text.color"
                }
            },
            None,
        )
    } else if focused {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.focus.supporting-text.color"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.focus.supporting-text.color"
                }
            },
            None,
        )
    } else if hovered {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.hover.supporting-text.color"
                }
                SelectVariant::Filled => {
                    "md.comp.filled-select.text-field.hover.supporting-text.color"
                }
            },
            None,
        )
    } else {
        (
            match variant {
                SelectVariant::Outlined => {
                    "md.comp.outlined-select.text-field.supporting-text.color"
                }
                SelectVariant::Filled => "md.comp.filled-select.text-field.supporting-text.color",
            },
            None,
        )
    };

    let color = theme
        .color_by_key(color_key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface-variant"));

    let opacity = opacity_key
        .and_then(|k| theme.number_by_key(k))
        .unwrap_or(1.0);
    alpha_mul(color, opacity)
}

pub(crate) fn menu_container_background(theme: &Theme, variant: SelectVariant) -> Color {
    let key = match variant {
        SelectVariant::Outlined => "md.comp.outlined-select.menu.container.color",
        SelectVariant::Filled => "md.comp.filled-select.menu.container.color",
    };
    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.surface-container"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.surface-container"))
}

pub(crate) fn menu_container_elevation(theme: &Theme, variant: SelectVariant) -> Px {
    let key = match variant {
        SelectVariant::Outlined => "md.comp.outlined-select.menu.container.elevation",
        SelectVariant::Filled => "md.comp.filled-select.menu.container.elevation",
    };
    theme.metric_by_key(key).unwrap_or(Px(3.0))
}

pub(crate) fn menu_container_shadow_color(theme: &Theme, variant: SelectVariant) -> Color {
    let key = match variant {
        SelectVariant::Outlined => "md.comp.outlined-select.menu.container.shadow-color",
        SelectVariant::Filled => "md.comp.filled-select.menu.container.shadow-color",
    };
    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.shadow"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.shadow"))
}

pub(crate) fn menu_container_shape(theme: &Theme, variant: SelectVariant) -> Corners {
    let key = match variant {
        SelectVariant::Outlined => "md.comp.outlined-select.menu.container.shape",
        SelectVariant::Filled => "md.comp.filled-select.menu.container.shape",
    };
    theme
        .corners_by_key(key)
        .or_else(|| theme.corners_by_key("md.sys.shape.corner.extra-small"))
        .unwrap_or_else(|| Corners::all(Px(4.0)))
}

pub(crate) fn menu_list_item_height(theme: &Theme, variant: SelectVariant) -> Px {
    let key = match variant {
        SelectVariant::Outlined => "md.comp.outlined-select.menu.list-item.container.height",
        SelectVariant::Filled => "md.comp.filled-select.menu.list-item.container.height",
    };
    theme.metric_by_key(key).unwrap_or(Px(48.0))
}

pub(crate) fn menu_list_item_label_text_style(
    theme: &Theme,
    variant: SelectVariant,
) -> Option<TextStyle> {
    let _ = variant;
    // Material Web v30 `menu.list-item.label-text.type` is a mixin. The underlying scalars map to
    // sys `label-large`, so use that as the stable default in v1.
    theme.text_style_by_key("md.sys.typescale.label-large")
}

pub(crate) fn menu_list_item_label_text_color(theme: &Theme, variant: SelectVariant) -> Color {
    let key = match variant {
        SelectVariant::Outlined => "md.comp.outlined-select.menu.list-item.label-text.color",
        SelectVariant::Filled => "md.comp.filled-select.menu.list-item.label-text.color",
    };
    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface"))
}

pub(crate) fn menu_list_item_leading_icon_size(theme: &Theme, variant: SelectVariant) -> Px {
    let key = match variant {
        SelectVariant::Outlined => {
            "md.comp.outlined-select.menu.list-item.with-leading-icon.leading-icon.size"
        }
        SelectVariant::Filled => {
            "md.comp.filled-select.menu.list-item.with-leading-icon.leading-icon.size"
        }
    };
    theme.metric_by_key(key).unwrap_or(Px(24.0))
}

pub(crate) fn menu_list_item_leading_icon_color(theme: &Theme, variant: SelectVariant) -> Color {
    let key = match variant {
        SelectVariant::Outlined => {
            "md.comp.outlined-select.menu.list-item.with-leading-icon.leading-icon.color"
        }
        SelectVariant::Filled => {
            "md.comp.filled-select.menu.list-item.with-leading-icon.leading-icon.color"
        }
    };
    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface-variant"))
}

pub(crate) fn menu_list_item_trailing_icon_size(theme: &Theme, variant: SelectVariant) -> Px {
    let key = match variant {
        SelectVariant::Outlined => {
            "md.comp.outlined-select.menu.list-item.with-trailing-icon.trailing-icon.size"
        }
        SelectVariant::Filled => {
            "md.comp.filled-select.menu.list-item.with-trailing-icon.trailing-icon.size"
        }
    };
    theme.metric_by_key(key).unwrap_or(Px(24.0))
}

pub(crate) fn menu_list_item_trailing_icon_color(theme: &Theme, variant: SelectVariant) -> Color {
    let key = match variant {
        SelectVariant::Outlined => {
            "md.comp.outlined-select.menu.list-item.with-trailing-icon.trailing-icon.color"
        }
        SelectVariant::Filled => {
            "md.comp.filled-select.menu.list-item.with-trailing-icon.trailing-icon.color"
        }
    };
    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.on-surface-variant"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.on-surface-variant"))
}

pub(crate) fn menu_list_item_selected_container_color(
    theme: &Theme,
    variant: SelectVariant,
) -> Color {
    let key = match variant {
        SelectVariant::Outlined => {
            "md.comp.outlined-select.menu.list-item.selected.container.color"
        }
        SelectVariant::Filled => "md.comp.filled-select.menu.list-item.selected.container.color",
    };
    theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key("md.sys.color.surface-container-highest"))
        .unwrap_or_else(|| theme.color_required("md.sys.color.surface-container-highest"))
}
