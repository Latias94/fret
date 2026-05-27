use fret_core::{CursorIcon, Px};
use fret_ui::element::{InsetStyle, LayoutStyle, Length, PositionStyle};

use super::super::super::FloatWindowResizeHandle;

pub(super) fn resize_handle_layout(handle: FloatWindowResizeHandle) -> (CursorIcon, LayoutStyle) {
    match handle {
        FloatWindowResizeHandle::Left => {
            let mut layout = LayoutStyle::default();
            layout.position = PositionStyle::Absolute;
            layout.inset = InsetStyle {
                left: Some(Px(0.0)).into(),
                top: Some(Px(0.0)).into(),
                bottom: Some(Px(0.0)).into(),
                ..Default::default()
            };
            layout.size.width = Length::Px(Px(6.0));
            layout.size.height = Length::Fill;
            (CursorIcon::ColResize, layout)
        }
        FloatWindowResizeHandle::Right => {
            let mut layout = LayoutStyle::default();
            layout.position = PositionStyle::Absolute;
            layout.inset = InsetStyle {
                right: Some(Px(0.0)).into(),
                top: Some(Px(0.0)).into(),
                bottom: Some(Px(0.0)).into(),
                ..Default::default()
            };
            layout.size.width = Length::Px(Px(6.0));
            layout.size.height = Length::Fill;
            (CursorIcon::ColResize, layout)
        }
        FloatWindowResizeHandle::Top => {
            let mut layout = LayoutStyle::default();
            layout.position = PositionStyle::Absolute;
            layout.inset = InsetStyle {
                left: Some(Px(0.0)).into(),
                right: Some(Px(0.0)).into(),
                top: Some(Px(0.0)).into(),
                ..Default::default()
            };
            layout.size.width = Length::Fill;
            layout.size.height = Length::Px(Px(6.0));
            (CursorIcon::RowResize, layout)
        }
        FloatWindowResizeHandle::Bottom => {
            let mut layout = LayoutStyle::default();
            layout.position = PositionStyle::Absolute;
            layout.inset = InsetStyle {
                left: Some(Px(0.0)).into(),
                right: Some(Px(0.0)).into(),
                bottom: Some(Px(0.0)).into(),
                ..Default::default()
            };
            layout.size.width = Length::Fill;
            layout.size.height = Length::Px(Px(6.0));
            (CursorIcon::RowResize, layout)
        }
        FloatWindowResizeHandle::TopLeft => {
            let mut layout = LayoutStyle::default();
            layout.position = PositionStyle::Absolute;
            layout.inset = InsetStyle {
                left: Some(Px(0.0)).into(),
                top: Some(Px(0.0)).into(),
                ..Default::default()
            };
            layout.size.width = Length::Px(Px(10.0));
            layout.size.height = Length::Px(Px(10.0));
            (CursorIcon::NwseResize, layout)
        }
        FloatWindowResizeHandle::TopRight => {
            let mut layout = LayoutStyle::default();
            layout.position = PositionStyle::Absolute;
            layout.inset = InsetStyle {
                right: Some(Px(0.0)).into(),
                top: Some(Px(0.0)).into(),
                ..Default::default()
            };
            layout.size.width = Length::Px(Px(10.0));
            layout.size.height = Length::Px(Px(10.0));
            (CursorIcon::NeswResize, layout)
        }
        FloatWindowResizeHandle::BottomLeft => {
            let mut layout = LayoutStyle::default();
            layout.position = PositionStyle::Absolute;
            layout.inset = InsetStyle {
                left: Some(Px(0.0)).into(),
                bottom: Some(Px(0.0)).into(),
                ..Default::default()
            };
            layout.size.width = Length::Px(Px(10.0));
            layout.size.height = Length::Px(Px(10.0));
            (CursorIcon::NeswResize, layout)
        }
        FloatWindowResizeHandle::BottomRight => {
            let mut layout = LayoutStyle::default();
            layout.position = PositionStyle::Absolute;
            layout.inset = InsetStyle {
                right: Some(Px(0.0)).into(),
                bottom: Some(Px(0.0)).into(),
                ..Default::default()
            };
            layout.size.width = Length::Px(Px(10.0));
            layout.size.height = Length::Px(Px(10.0));
            (CursorIcon::NwseResize, layout)
        }
    }
}
