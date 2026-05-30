use fret_core::{Color, Corners, Edges, Px, Size};
use fret_ui::element::{ContainerProps, Length};

pub(in crate::imui::floating_window_shell) fn window_frame_props(
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
    props.corner_radii = Corners::all(crate::imui::control_chrome::PANEL_RADIUS);
    props
}
