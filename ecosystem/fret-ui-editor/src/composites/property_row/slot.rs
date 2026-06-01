use fret_core::{Axis, Edges, Px};
use fret_ui::element::{
    AnyElement, FlexItemStyle, FlexProps, LayoutStyle, Length, Overflow, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

pub(super) fn property_row_trailing_slot<H, Children>(
    cx: &mut ElementContext<'_, H>,
    width: Px,
    row_height: Px,
    children: Children,
) -> AnyElement
where
    H: UiHost,
    Children: FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement> + 'static,
{
    cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(width),
                    height: Length::Auto,
                    min_height: Some(Length::Px(row_height)),
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    order: 0,
                    grow: 0.0,
                    shrink: 0.0,
                    basis: Length::Px(width),
                    align_self: None,
                },
                overflow: Overflow::Clip,
                ..Default::default()
            },
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: fret_ui::element::MainAlign::End,
            align: fret_ui::element::CrossAlign::Center,
            wrap: false,
        },
        children,
    )
}
