use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    SemanticsDecoration, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::colors::{editor_foreground, editor_popup_background, editor_popup_border};
use crate::primitives::readout::editor_tooltip_readout_text_props;

use super::super::super::ColorEditAlphaPreview;
use super::super::preview::{color_preview_stack, preview_color_for_alpha_visibility};
use super::color_tooltip_lines;

pub(super) fn color_tooltip_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    color: Color,
    show_alpha: bool,
    alpha_preview: ColorEditAlphaPreview,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let lines = color_tooltip_lines(color, show_alpha);
    let preview_color = preview_color_for_alpha_visibility(color, show_alpha);
    let theme = Theme::global(&*cx.app);
    let bg = editor_popup_background(theme);
    let fg = editor_foreground(theme);
    let border = editor_popup_border(theme);
    let radius = Px(5.0);

    let panel = cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Auto,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            padding: Edges::all(Px(6.0)).into(),
            background: Some(bg),
            border: Edges::all(Px(1.0)),
            border_color: Some(border),
            corner_radii: Corners::all(radius),
            ..Default::default()
        },
        {
            let lines_for_text = lines.clone();
            move |cx| {
                vec![cx.flex(
                    FlexProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Auto,
                                height: Length::Auto,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        direction: Axis::Horizontal,
                        gap: SpacingLength::Px(Px(8.0)),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                    },
                    move |cx| {
                        vec![
                            cx.container(
                                ContainerProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Px(Px(44.0)),
                                            height: Length::Px(Px(44.0)),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    corner_radii: Corners::all(radius),
                                    ..Default::default()
                                },
                                move |cx| {
                                    vec![color_preview_stack(
                                        cx,
                                        preview_color,
                                        radius,
                                        alpha_preview,
                                    )]
                                },
                            ),
                            cx.flex(
                                FlexProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Auto,
                                            height: Length::Auto,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    direction: Axis::Vertical,
                                    gap: SpacingLength::Px(Px(2.0)),
                                    padding: Edges::all(Px(0.0)).into(),
                                    justify: MainAlign::Start,
                                    align: CrossAlign::Stretch,
                                    wrap: false,
                                },
                                move |cx| {
                                    lines_for_text
                                        .iter()
                                        .cloned()
                                        .map(|line| {
                                            cx.text_props(editor_tooltip_readout_text_props(
                                                line, fg,
                                            ))
                                        })
                                        .collect::<Vec<_>>()
                                },
                            ),
                        ]
                    },
                )]
            }
        },
    );

    let semantics_value = lines
        .iter()
        .map(|line| line.as_ref())
        .collect::<Vec<_>>()
        .join(" ");
    let mut semantics = SemanticsDecoration::default()
        .role(SemanticsRole::Tooltip)
        .value(Arc::from(semantics_value));
    if let Some(test_id) = test_id {
        semantics = semantics.test_id(test_id);
    }
    panel.attach_semantics(semantics)
}
