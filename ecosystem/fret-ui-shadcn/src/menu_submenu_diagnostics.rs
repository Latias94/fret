use fret_core::{Point, Px, Rect};
use fret_ui::overlay_placement::Side;
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::primitives::menu;

pub(crate) fn submenu_geometry_side(geometry: menu::sub::MenuSubmenuGeometry) -> Option<Side> {
    let reference = geometry.reference;
    let floating = geometry.floating;
    let reference_left = reference.origin.x.0;
    let reference_right = reference.origin.x.0 + reference.size.width.0;
    let reference_top = reference.origin.y.0;
    let reference_bottom = reference.origin.y.0 + reference.size.height.0;
    let floating_left = floating.origin.x.0;
    let floating_right = floating.origin.x.0 + floating.size.width.0;
    let floating_top = floating.origin.y.0;
    let floating_bottom = floating.origin.y.0 + floating.size.height.0;
    let eps = 0.5;

    if floating_left >= reference_right - eps {
        return Some(Side::Right);
    }
    if floating_right <= reference_left + eps {
        return Some(Side::Left);
    }
    if floating_top >= reference_bottom - eps {
        return Some(Side::Bottom);
    }
    if floating_bottom <= reference_top + eps {
        return Some(Side::Top);
    }

    let reference_center = Point::new(
        Px(reference.origin.x.0 + reference.size.width.0 * 0.5),
        Px(reference.origin.y.0 + reference.size.height.0 * 0.5),
    );
    let floating_center = Point::new(
        Px(floating.origin.x.0 + floating.size.width.0 * 0.5),
        Px(floating.origin.y.0 + floating.size.height.0 * 0.5),
    );
    let dx = floating_center.x.0 - reference_center.x.0;
    let dy = floating_center.y.0 - reference_center.y.0;

    if dx.abs() >= dy.abs() {
        Some(if dx >= 0.0 { Side::Right } else { Side::Left })
    } else {
        Some(if dy >= 0.0 { Side::Bottom } else { Side::Top })
    }
}

pub(crate) fn record_submenu_placement<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    overlay_root_name: Option<&str>,
    models: &menu::sub::MenuSubmenuModels,
    outer: Rect,
    geometry: menu::sub::MenuSubmenuGeometry,
) {
    let anchor_element = cx
        .app
        .models_mut()
        .read(&models.trigger, |v| *v)
        .ok()
        .flatten();
    cx.diagnostics_record_overlay_placement_placed_rect(
        overlay_root_name,
        anchor_element,
        None,
        outer,
        geometry.reference,
        geometry.floating,
        submenu_geometry_side(geometry),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_core::Size;

    #[test]
    fn submenu_geometry_side_tracks_floating_position() {
        let reference = Rect::new(
            Point::new(Px(100.0), Px(100.0)),
            Size::new(Px(40.0), Px(30.0)),
        );
        let geometry = |floating: Rect| menu::sub::MenuSubmenuGeometry {
            reference,
            floating,
        };

        assert_eq!(
            submenu_geometry_side(geometry(Rect::new(
                Point::new(Px(142.0), Px(100.0)),
                Size::new(Px(90.0), Px(60.0)),
            ))),
            Some(Side::Right)
        );
        assert_eq!(
            submenu_geometry_side(geometry(Rect::new(
                Point::new(Px(8.0), Px(100.0)),
                Size::new(Px(90.0), Px(60.0)),
            ))),
            Some(Side::Left)
        );
        assert_eq!(
            submenu_geometry_side(geometry(Rect::new(
                Point::new(Px(100.0), Px(132.0)),
                Size::new(Px(90.0), Px(60.0)),
            ))),
            Some(Side::Bottom)
        );
        assert_eq!(
            submenu_geometry_side(geometry(Rect::new(
                Point::new(Px(100.0), Px(38.0)),
                Size::new(Px(90.0), Px(60.0)),
            ))),
            Some(Side::Top)
        );
    }
}
