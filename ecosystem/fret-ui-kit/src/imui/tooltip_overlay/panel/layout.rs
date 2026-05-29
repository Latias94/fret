use fret_core::{Corners, Edges, Point, Px};
use fret_ui::element::{
    ColumnProps, ContainerProps, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle,
    SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

pub(super) fn tooltip_panel_props<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    origin: Point,
) -> ContainerProps {
    let theme = fret_ui::Theme::global(&*cx.app);
    ContainerProps {
        layout: LayoutStyle {
            position: PositionStyle::Absolute,
            inset: InsetStyle {
                left: Some(origin.x).into(),
                top: Some(origin.y).into(),
                ..Default::default()
            },
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Auto,
                ..Default::default()
            },
            overflow: Overflow::Visible,
            ..Default::default()
        },
        padding: Edges::all(Px(4.0)).into(),
        background: Some(theme.color_token("popover")),
        border: Edges::all(Px(1.0)),
        border_color: Some(theme.color_token("border")),
        corner_radii: Corners::all(crate::imui::control_chrome::PANEL_RADIUS),
        ..Default::default()
    }
}

pub(super) fn tooltip_panel_column_props() -> ColumnProps {
    let mut column = ColumnProps::default();
    column.layout.size.width = Length::Auto;
    column.layout.size.height = Length::Auto;
    column.gap = SpacingLength::Px(Px(4.0));
    column
}
