use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    PressableA11y, SizeStyle, SpacerProps, SpacingLength,
};
use fret_ui::{ElementContext, Theme, UiHost};

use super::{DisclosureKind, DisclosureSpec};

pub(super) fn disclosure_a11y(spec: &DisclosureSpec, open_now: bool) -> PressableA11y {
    match spec.kind {
        DisclosureKind::CollapsingHeader => PressableA11y {
            role: Some(SemanticsRole::Button),
            label: Some(spec.label.clone()),
            expanded: spec.has_children().then_some(open_now),
            ..Default::default()
        },
        DisclosureKind::TreeNode => PressableA11y {
            role: Some(SemanticsRole::TreeItem),
            label: Some(spec.label.clone()),
            level: Some(spec.level),
            selected: spec.selected,
            expanded: spec.has_children().then_some(open_now),
            pos_in_set: spec.pos_in_set,
            set_size: spec.set_size,
            ..Default::default()
        },
    }
}

pub(super) fn disclosure_content_padding(spec: &DisclosureSpec) -> Edges {
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

pub(super) fn header_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    spec: &DisclosureSpec,
    label: Arc<str>,
    open_now: bool,
    state: fret_ui::element::PressableState,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let palette = resolve_disclosure_palette(theme, spec, state);
    let border = theme.color_token("border");
    let indicator: Option<Arc<str>> = if spec.leaf {
        None
    } else if open_now {
        Some(Arc::from("v"))
    } else {
        Some(Arc::from(">"))
    };
    let row_padding = match spec.kind {
        DisclosureKind::CollapsingHeader => Edges {
            top: Px(4.0),
            right: Px(6.0),
            bottom: Px(4.0),
            left: Px(6.0),
        },
        DisclosureKind::TreeNode => {
            let indent = Px(16.0 * (spec.level.saturating_sub(1) as f32));
            Edges {
                top: Px(2.0),
                right: Px(6.0),
                bottom: Px(2.0),
                left: Px(6.0 + indent.0),
            }
        }
    };
    let border_edges = match spec.kind {
        DisclosureKind::CollapsingHeader => Edges::all(Px(1.0)),
        DisclosureKind::TreeNode => Edges::all(Px(0.0)),
    };

    let mut row_props = ContainerProps::default();
    row_props.layout = LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Auto,
            ..Default::default()
        },
        overflow: Overflow::Visible,
        ..Default::default()
    };
    row_props.padding = row_padding.into();
    row_props.background = palette.background;
    row_props.border = border_edges;
    row_props.border_color = (spec.kind == DisclosureKind::CollapsingHeader).then_some(border);
    row_props.corner_radii = Corners::all(super::super::control_chrome::CONTROL_RADIUS);

    cx.container(row_props, move |cx| {
        vec![cx.flex(
            FlexProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                direction: Axis::Horizontal,
                gap: SpacingLength::Px(Px(4.0)),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
                ..Default::default()
            },
            move |cx| {
                let mut out = Vec::new();
                out.push(cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Px(Px(12.0)),
                                height: Length::Auto,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    move |cx| {
                        indicator
                            .as_ref()
                            .map(|indicator| {
                                vec![
                                    crate::declarative::text::text_chrome_glyph(
                                        cx,
                                        indicator.clone(),
                                    )
                                    .inherit_foreground(palette.foreground),
                                ]
                            })
                            .unwrap_or_default()
                    },
                ));

                out.push(disclosure_label_text(cx, label, palette.foreground));
                out.push(cx.spacer(SpacerProps::default()));
                out
            },
        )]
    })
}

fn disclosure_label_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    foreground: Color,
) -> AnyElement {
    crate::declarative::text::text_list_row_label(cx, label).inherit_foreground(foreground)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct DisclosurePalette {
    pub(super) background: Option<Color>,
    pub(super) foreground: Color,
}

pub(super) fn resolve_disclosure_palette(
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
