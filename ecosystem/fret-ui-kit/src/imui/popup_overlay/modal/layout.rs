use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px, SemanticsRole, Size};
use fret_ui::element::{
    ContainerProps, InsetStyle, LayoutStyle, Length, Overflow, PositionStyle, SemanticsProps,
    SizeStyle, StackProps,
};

pub(super) struct PopupModalPalette {
    pub(super) popover: Color,
    pub(super) border: Color,
    pub(super) dim: Color,
}

#[derive(Clone, Copy)]
pub(super) struct PopupModalPanelLayout {
    pub(super) left: Px,
    pub(super) top: Px,
    pub(super) size: Size,
}

pub(super) fn popup_modal_palette(theme: &fret_ui::Theme) -> PopupModalPalette {
    PopupModalPalette {
        popover: theme.color_token("popover"),
        border: theme.color_token("border"),
        dim: Color {
            a: 0.4,
            ..Color::from_srgb_hex_rgb(0x00_00_00)
        },
    }
}

pub(super) fn centered_panel_layout(bounds: fret_core::Rect, size: Size) -> PopupModalPanelLayout {
    PopupModalPanelLayout {
        left: Px(bounds.origin.x.0 + (bounds.size.width.0 - size.width.0).max(0.0) * 0.5),
        top: Px(bounds.origin.y.0 + (bounds.size.height.0 - size.height.0).max(0.0) * 0.5),
        size,
    }
}

pub(super) fn modal_layer_stack_props() -> StackProps {
    let mut stack = StackProps::default();
    stack.layout.position = PositionStyle::Absolute;
    stack.layout.inset = full_inset();
    stack.layout.size.width = Length::Fill;
    stack.layout.size.height = Length::Fill;
    stack.layout.overflow = Overflow::Visible;
    stack
}

pub(super) fn modal_backdrop_props(dim: Color) -> ContainerProps {
    let mut props = ContainerProps::default();
    props.layout.position = PositionStyle::Absolute;
    props.layout.inset = full_inset();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Fill;
    props.background = Some(dim);
    props
}

pub(super) fn modal_panel_semantics(id: &str, layout: PopupModalPanelLayout) -> SemanticsProps {
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

pub(super) fn modal_panel_props(palette: &PopupModalPalette) -> ContainerProps {
    let mut panel_props = ContainerProps::default();
    panel_props.background = Some(palette.popover);
    panel_props.border = Edges::all(Px(1.0));
    panel_props.border_color = Some(palette.border);
    panel_props.corner_radii = Corners::all(super::super::super::control_chrome::PANEL_RADIUS);
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
