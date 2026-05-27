use fret_core::{CursorIcon, Px};
use fret_ui::element::{InsetStyle, LayoutStyle, Length, PositionStyle};

const CHILD_REGION_RESIZE_X_HANDLE_WIDTH: Px = Px(6.0);
const CHILD_REGION_RESIZE_Y_HANDLE_HEIGHT: Px = Px(6.0);

#[derive(Clone, Copy)]
pub(super) enum ChildRegionResizeAxis {
    X,
    Y,
}

impl ChildRegionResizeAxis {
    pub(super) fn key(self) -> &'static str {
        match self {
            Self::X => "child-region-resize-x",
            Self::Y => "child-region-resize-y",
        }
    }

    pub(super) fn cursor(self) -> CursorIcon {
        match self {
            Self::X => CursorIcon::ColResize,
            Self::Y => CursorIcon::RowResize,
        }
    }

    pub(super) fn layout(self) -> LayoutStyle {
        let mut layout = LayoutStyle {
            position: PositionStyle::Absolute,
            ..Default::default()
        };
        match self {
            Self::X => {
                layout.inset = InsetStyle {
                    top: Some(Px(0.0)).into(),
                    right: Some(Px(0.0)).into(),
                    bottom: Some(Px(0.0)).into(),
                    ..Default::default()
                };
                layout.size.width = Length::Px(CHILD_REGION_RESIZE_X_HANDLE_WIDTH);
                layout.size.height = Length::Fill;
            }
            Self::Y => {
                layout.inset = InsetStyle {
                    left: Some(Px(0.0)).into(),
                    right: Some(Px(0.0)).into(),
                    bottom: Some(Px(0.0)).into(),
                    ..Default::default()
                };
                layout.size.width = Length::Fill;
                layout.size.height = Length::Px(CHILD_REGION_RESIZE_Y_HANDLE_HEIGHT);
            }
        }
        layout
    }
}
