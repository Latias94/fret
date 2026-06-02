use fret_core::{Axis, Corners, Edges, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle,
    SpacingLength,
};
use fret_ui::{ElementContext, UiHost};
use fret_ui_headless::checked_state::CheckedState;
use fret_ui_kit::ColorRef;

use crate::primitives::visuals::EditorFrameVisuals;

pub(super) fn checkbox_indicator_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    checked_state: CheckedState,
    visuals: EditorFrameVisuals,
    checkbox_size: Px,
    checkbox_radius: Px,
    border_width: Px,
) -> AnyElement {
    let icon_id = match checked_state {
        CheckedState::Checked => Some(fret_icons::ids::ui::CHECK),
        CheckedState::Indeterminate => Some(fret_icons::ids::ui::MINUS),
        CheckedState::Unchecked => None,
    };
    let icon_px = Px((checkbox_size.0 - 4.0).max(8.0));

    let box_el = cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(checkbox_size),
                    height: Length::Px(checkbox_size),
                    ..Default::default()
                },
                ..Default::default()
            },
            background: Some(visuals.bg),
            border: Edges::all(border_width),
            border_color: Some(visuals.border),
            corner_radii: Corners::all(checkbox_radius),
            ..Default::default()
        },
        move |cx| {
            let Some(icon) = icon_id else {
                return vec![];
            };

            vec![cx.flex(
                FlexProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    direction: Axis::Horizontal,
                    gap: SpacingLength::Px(Px(0.0)),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |cx| {
                    vec![fret_ui_kit::declarative::icon::icon_with(
                        cx,
                        icon,
                        Some(icon_px),
                        Some(ColorRef::Color(visuals.icon)),
                    )]
                },
            )]
        },
    );

    cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Center,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |_cx| vec![box_el],
    )
}
