use fret_core::{Color, Edges, Px};
use fret_ui::Theme;

use super::super::spec::{DisclosureKind, DisclosureSpec};

pub(in crate::imui::disclosure_controls) fn disclosure_content_padding(
    spec: &DisclosureSpec,
) -> Edges {
    match spec.kind {
        DisclosureKind::CollapsingHeader => Edges {
            top: Px(4.0),
            right: Px(0.0),
            bottom: Px(0.0),
            left: Px(0.0),
        },
        DisclosureKind::TreeNode => Edges {
            top: Px(0.0),
            right: Px(0.0),
            bottom: Px(0.0),
            left: Px(0.0),
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::imui::disclosure_controls) struct DisclosurePalette {
    pub(in crate::imui::disclosure_controls) background: Option<Color>,
    pub(in crate::imui::disclosure_controls) foreground: Color,
}

pub(in crate::imui::disclosure_controls) fn resolve_disclosure_palette(
    theme: &Theme,
    spec: &DisclosureSpec,
    state: fret_ui::element::PressableState,
) -> DisclosurePalette {
    let selected_bg = theme
        .color_by_key("list.active.background")
        .or_else(|| theme.color_by_key("selection.background"))
        .unwrap_or_else(|| theme.color_token("selection.background"));
    let hover_bg = theme
        .color_by_key("list.hover.background")
        .or_else(|| theme.color_by_key("accent"))
        .unwrap_or_else(|| theme.color_token("accent"));
    let idle_bg = theme
        .color_by_key("card")
        .or_else(|| theme.color_by_key("popover"))
        .unwrap_or_else(|| theme.color_token("popover"));
    let foreground = theme
        .color_by_key("foreground")
        .unwrap_or_else(|| theme.color_token("foreground"));
    let hover_foreground = theme
        .color_by_key("accent-foreground")
        .or_else(|| theme.color_by_key("foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"));
    let interactive = state.pressed || state.hovered || state.focused;

    match spec.kind {
        DisclosureKind::CollapsingHeader => {
            if interactive {
                DisclosurePalette {
                    background: Some(if state.pressed { selected_bg } else { hover_bg }),
                    foreground: hover_foreground,
                }
            } else {
                DisclosurePalette {
                    background: Some(idle_bg),
                    foreground,
                }
            }
        }
        DisclosureKind::TreeNode => {
            if spec.selected {
                DisclosurePalette {
                    background: Some(selected_bg),
                    foreground: hover_foreground,
                }
            } else if interactive {
                DisclosurePalette {
                    background: Some(if state.pressed { selected_bg } else { hover_bg }),
                    foreground: hover_foreground,
                }
            } else {
                DisclosurePalette {
                    background: None,
                    foreground,
                }
            }
        }
    }
}
