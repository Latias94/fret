use std::sync::Arc;

use fret_ui::ElementContext;
use fret_ui::UiHost;
use fret_ui::element::{AnyElement, LayoutStyle, Length, PointerRegionProps, PositionStyle};

pub(super) fn floating_window_blocker_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    inputs_enabled: bool,
) -> Option<AnyElement> {
    (!inputs_enabled).then(|| {
        let mut layout = LayoutStyle::default();
        layout.position = PositionStyle::Absolute;
        layout.inset = fret_ui::element::InsetStyle {
            left: Some(fret_core::Px(0.0)).into(),
            right: Some(fret_core::Px(0.0)).into(),
            top: Some(fret_core::Px(0.0)).into(),
            bottom: Some(fret_core::Px(0.0)).into(),
        };
        layout.size.width = Length::Fill;
        layout.size.height = Length::Fill;

        cx.pointer_region(
            PointerRegionProps {
                layout,
                enabled: true,
                ..Default::default()
            },
            move |cx| {
                cx.pointer_region_clear_on_pointer_down();
                cx.pointer_region_clear_on_pointer_move();
                cx.pointer_region_clear_on_pointer_up();

                cx.pointer_region_on_pointer_down(Arc::new(|_host, _acx, _down| true));
                cx.pointer_region_on_pointer_move(Arc::new(|_host, _acx, _mv| true));
                cx.pointer_region_on_pointer_up(Arc::new(|_host, _acx, _up| true));
                Vec::new()
            },
        )
    })
}
