use fret_core::Px;
use fret_ui::element::{ContainerProps, LayoutStyle, Length, Overflow};

pub(super) fn content_surface_layout(resizable_layout: bool) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    if resizable_layout {
        layout.size.width = Length::Fill;
        layout.size.height = Length::Fill;
    } else {
        layout.size.width = Length::Auto;
        layout.size.height = Length::Auto;
    }
    layout
}

pub(super) fn content_scroll_layout(resizable_layout: bool) -> LayoutStyle {
    let mut layout = content_surface_layout(resizable_layout);
    layout.overflow = Overflow::Clip;
    layout
}

pub(super) fn content_container_props(resizable_layout: bool) -> ContainerProps {
    let mut props = ContainerProps::default();
    props.layout.size.width = if resizable_layout {
        Length::Fill
    } else {
        Length::Auto
    };
    props.padding = fret_core::Edges::all(Px(6.0)).into();
    props
}
