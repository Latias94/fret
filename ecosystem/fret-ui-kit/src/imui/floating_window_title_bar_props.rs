use std::sync::Arc;

use fret_core::{Px, SemanticsRole};
use fret_ui::element::{
    LayoutStyle, Length, PointerRegionProps, PressableA11y, PressableProps, RowProps, SpacingLength,
};

pub(super) fn title_bar_row_props(resizable_layout: bool) -> RowProps {
    let mut row = RowProps::default();
    row.layout.size.width = if resizable_layout {
        Length::Fill
    } else {
        Length::Auto
    };
    row.layout.size.height = Length::Fill;
    row.gap = SpacingLength::Px(Px(4.0));
    row.align = fret_ui::element::CrossAlign::Center;
    row
}

pub(super) fn title_bar_drag_surface_props(
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

pub(super) fn title_bar_close_button_props(close_button_test_id: Arc<str>) -> PressableProps {
    let mut props = PressableProps::default();
    props.a11y = PressableA11y {
        role: Some(SemanticsRole::Button),
        label: Some(Arc::from("Close")),
        test_id: Some(close_button_test_id),
        ..Default::default()
    };
    props.layout.size.width = Length::Px(Px(20.0));
    props.layout.size.height = Length::Px(Px(20.0));
    props.layout.flex.shrink = 0.0;
    props
}
