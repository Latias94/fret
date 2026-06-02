use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px, SemanticsRole};
use fret_ui::element::{
    ContainerProps, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle, SemanticsProps,
    SizeStyle, StackProps,
};

use super::types::{PopupModalPalette, PopupModalPanelLayout};

pub(in crate::imui::popup_overlay::modal) fn modal_layer_stack_props() -> StackProps {
    let mut stack = StackProps::default();
    stack.layout.position = PositionStyle::Absolute;
    stack.layout.inset = full_inset();
    stack.layout.size.width = Length::Fill;
    stack.layout.size.height = Length::Fill;
    stack.layout.overflow = Overflow::Visible;
    stack
}

pub(in crate::imui::popup_overlay::modal) fn modal_backdrop_props(dim: Color) -> ContainerProps {
    let mut props = ContainerProps::default();
    props.layout.position = PositionStyle::Absolute;
    props.layout.inset = full_inset();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Fill;
    props.background = Some(dim);
    props
}

pub(in crate::imui::popup_overlay::modal) fn modal_panel_semantics(
    id: &str,
    layout: PopupModalPanelLayout,
) -> SemanticsProps {
    let mut semantics = SemanticsProps::default();
    semantics.role = SemanticsRole::Dialog;
    semantics.test_id = Some(Arc::from(format!("imui-popup-modal-{id}")));
    semantics.layout = LayoutStyle {
        position: PositionStyle::Absolute,
        inset: InsetStyle {
            left: Some(layout.left).into(),
            top: Some(layout.top).into(),
            ..Default::default()
        },
        size: SizeStyle {
            width: Length::Px(layout.size.width),
            height: Length::Px(layout.size.height),
            ..Default::default()
        },
        ..Default::default()
    };
    semantics
}

pub(in crate::imui::popup_overlay::modal) fn modal_panel_props(
    palette: &PopupModalPalette,
) -> ContainerProps {
    let mut panel_props = ContainerProps::default();
    panel_props.background = Some(palette.popover);
    panel_props.border = Edges::all(Px(1.0));
    panel_props.border_color = Some(palette.border);
    panel_props.corner_radii = Corners::all(crate::imui::control_chrome::PANEL_RADIUS);
    panel_props.padding = Edges::all(Px(8.0)).into();
    panel_props.layout.size.width = Length::Fill;
    panel_props.layout.size.height = Length::Fill;
    panel_props
}

fn full_inset() -> InsetStyle {
    InsetStyle {
        left: Some(Px(0.0)).into(),
        right: Some(Px(0.0)).into(),
        top: Some(Px(0.0)).into(),
        bottom: Some(Px(0.0)).into(),
    }
}
