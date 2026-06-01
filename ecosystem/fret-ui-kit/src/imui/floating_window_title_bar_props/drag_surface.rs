use fret_core::Px;
use fret_ui::element::{LayoutStyle, Length, PointerRegionProps};

pub(in crate::imui) fn title_bar_drag_surface_props(
    resizable_layout: bool,
    can_interact: bool,
) -> PointerRegionProps {
    PointerRegionProps {
        layout: title_bar_drag_surface_layout(resizable_layout),
        enabled: can_interact,
        ..Default::default()
    }
}

fn title_bar_drag_surface_layout(resizable_layout: bool) -> LayoutStyle {
    let mut layout = LayoutStyle::default();
    layout.size.width = if resizable_layout {
        Length::Fill
    } else {
        Length::Auto
    };
    layout.size.height = Length::Fill;
    if resizable_layout {
        // Ensure the drag surface claims remaining row space (and can shrink)
        // instead of being measured in min-content mode (which can force wrapped
        // titles like "Window" + "A").
        layout.flex.grow = 1.0;
        layout.flex.shrink = 1.0;
        layout.flex.basis = Length::Px(Px(0.0));
        layout.size.min_width = Some(Length::Px(Px(0.0)));
    }
    layout
}
