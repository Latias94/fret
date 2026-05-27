use fret_core::{Color, Corners, Edges, Px, Size};
use fret_ui::element::{ColumnProps, ContainerProps, Length, Overflow};

use super::super::control_chrome::PANEL_RADIUS;

pub(super) fn window_frame_props(
    window_size: Size,
    resizable_layout: bool,
    collapsed: bool,
    popover: Color,
    border: Color,
) -> ContainerProps {
    let mut props = ContainerProps::default();
    if resizable_layout {
        props.layout.size.width = Length::Px(window_size.width);
        if !collapsed {
            props.layout.size.height = Length::Px(window_size.height);
        }
    }
    props.background = Some(popover);
    props.border = Edges::all(Px(1.0));
    props.border_color = Some(border);
    props.corner_radii = Corners::all(PANEL_RADIUS);
    props
}

pub(super) fn shell_column_props(resizable_layout: bool, collapsed: bool) -> ColumnProps {
    let mut props = ColumnProps::default();
    props.layout.size.width = if resizable_layout {
        Length::Fill
    } else {
        Length::Auto
    };
    props.layout.size.height = if resizable_layout && !collapsed {
        Length::Fill
    } else {
        Length::Auto
    };
    props
}

pub(super) fn title_bar_container_props(
    resizable_layout: bool,
    muted: Color,
    border: Color,
) -> ContainerProps {
    let mut props = ContainerProps::default();
    props.layout.size.width = if resizable_layout {
        Length::Fill
    } else {
        Length::Auto
    };
    props.layout.size.height = Length::Px(Px(24.0));
    // Prevent multi-line title text from painting into the content area at
    // non-1.0 DPI when the layout engine probes min-content widths.
    props.layout.overflow = Overflow::Clip;
    props.padding = Edges {
        left: Px(6.0),
        right: Px(4.0),
        top: Px(4.0),
        bottom: Px(4.0),
    }
    .into();
    props.background = Some(muted);
    props.border = Edges {
        left: Px(0.0),
        right: Px(0.0),
        top: Px(0.0),
        bottom: Px(1.0),
    };
    props.border_color = Some(border);
    props.corner_radii = Corners {
        top_left: PANEL_RADIUS,
        top_right: PANEL_RADIUS,
        bottom_left: Px(0.0),
        bottom_right: Px(0.0),
    };
    props
}

pub(super) fn clipped_body_props(resizable_layout: bool, collapsed: bool) -> ContainerProps {
    let mut props = ContainerProps::default();
    // Clip window contents to the window bounds (ImGui-style): items should not paint outside the
    // window chrome even when they don't wrap. Keep this as an inner clip container so resize
    // handles can still receive hits near rounded corners.
    props.layout.overflow = Overflow::Clip;
    props.layout.size.width = if resizable_layout {
        Length::Fill
    } else {
        Length::Auto
    };
    props.layout.size.height = if resizable_layout && !collapsed {
        Length::Fill
    } else {
        Length::Auto
    };
    props.corner_radii = Corners::all(PANEL_RADIUS);
    props
}
