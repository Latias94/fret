use fret_core::Corners;
use fret_ui::element::{ColumnProps, ContainerProps, Length, Overflow};

pub(in crate::imui::floating_window_shell) fn shell_column_props(
    resizable_layout: bool,
    collapsed: bool,
) -> ColumnProps {
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

pub(in crate::imui::floating_window_shell) fn clipped_body_props(
    resizable_layout: bool,
    collapsed: bool,
) -> ContainerProps {
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
    props.corner_radii = Corners::all(crate::imui::control_chrome::PANEL_RADIUS);
    props
}
