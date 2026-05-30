use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Point, Px, Rect, SemanticsRole, Size};
use fret_ui::element::{
    ContainerProps, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle, SemanticsProps,
};
use fret_ui::{ElementContext, UiHost};

use crate::primitives::popper::{self, PopperContentPlacement};

#[derive(Debug, Clone, Copy)]
pub(super) struct PopupMenuPanelPalette {
    pub(super) popover: Color,
    pub(super) border: Color,
}

pub(super) fn popup_menu_panel_layout<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    anchor: Rect,
    desired: Size,
    placement: PopperContentPlacement,
) -> fret_ui::overlay_placement::AnchoredPanelLayout {
    popper::popper_content_layout_sized(
        cx.environment_viewport_bounds(fret_ui::Invalidation::Layout),
        anchor,
        desired,
        placement,
    )
}

pub(super) fn popup_menu_panel_palette(theme: &fret_ui::Theme) -> PopupMenuPanelPalette {
    PopupMenuPanelPalette {
        popover: theme.color_token("popover"),
        border: theme.color_token("border"),
    }
}

pub(super) fn popup_menu_panel_semantics(id: &str, origin: Point) -> SemanticsProps {
    let mut semantics = SemanticsProps::default();
    semantics.role = SemanticsRole::Menu;
    semantics.test_id = Some(Arc::from(format!("imui-popup-{id}")));
    semantics.layout = LayoutStyle {
        position: PositionStyle::Absolute,
        inset: InsetStyle {
            left: Some(origin.x).into(),
            top: Some(origin.y).into(),
            ..Default::default()
        },
        overflow: Overflow::Visible,
        ..Default::default()
    };
    semantics
}

pub(super) fn popup_menu_panel_props(palette: PopupMenuPanelPalette) -> ContainerProps {
    let mut panel_props = ContainerProps::default();
    panel_props.background = Some(palette.popover);
    panel_props.border = Edges::all(Px(1.0));
    panel_props.border_color = Some(palette.border);
    panel_props.corner_radii =
        Corners::all(super::super::super::super::control_chrome::PANEL_RADIUS);
    panel_props.padding = Edges::all(Px(4.0)).into();
    panel_props
}

pub(super) fn popup_menu_panel_column_props() -> fret_ui::element::ColumnProps {
    let mut col = fret_ui::element::ColumnProps::default();
    col.gap = fret_ui::element::SpacingLength::Px(Px(2.0));
    col.layout.size.width = Length::Auto;
    col.layout.size.height = Length::Auto;
    col
}
