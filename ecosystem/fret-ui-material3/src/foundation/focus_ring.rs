use fret_core::{Corners, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;

pub fn material_focus_ring_for_component(
    theme: &Theme,
    component_prefix: &str,
    corner_radii: Corners,
) -> fret_ui::element::RingStyle {
    let tokens = MaterialTokenResolver::new(theme);

    let mut color = theme
        .color_by_key(&format!("{component_prefix}.focus.indicator.color"))
        .unwrap_or_else(|| tokens.color_sys("md.sys.color.secondary"));
    color.a = 1.0;

    let thickness = theme
        .metric_by_key(&format!("{component_prefix}.focus.indicator.thickness"))
        .or_else(|| theme.metric_by_key("md.sys.state.focus-indicator.thickness"))
        .unwrap_or(Px(3.0));

    let outline_offset = theme
        .metric_by_key(&format!(
            "{component_prefix}.focus.indicator.outline.offset"
        ))
        .or_else(|| theme.metric_by_key(&format!("{component_prefix}.focus.indicator.offset")))
        .or_else(|| theme.metric_by_key("md.sys.state.focus-indicator.outer-offset"))
        .unwrap_or(Px(2.0));

    let (placement, offset) = if outline_offset.0 < 0.0 {
        (
            fret_ui::element::RingPlacement::Inset,
            Px(outline_offset.0.abs()),
        )
    } else {
        (fret_ui::element::RingPlacement::Outset, outline_offset)
    };

    fret_ui::element::RingStyle {
        placement,
        width: thickness,
        offset,
        color,
        offset_color: None,
        corner_radii,
    }
}
