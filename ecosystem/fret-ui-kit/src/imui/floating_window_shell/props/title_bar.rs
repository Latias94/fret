use fret_core::{Color, Corners, Edges, Px};
use fret_ui::element::{ContainerProps, Length, Overflow};

pub(in crate::imui::floating_window_shell) fn title_bar_container_props(
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
        top_left: crate::imui::control_chrome::PANEL_RADIUS,
        top_right: crate::imui::control_chrome::PANEL_RADIUS,
        bottom_left: Px(0.0),
        bottom_right: Px(0.0),
    };
    props
}
