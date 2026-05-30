//! Property row layout policy and resolved chrome metrics.

use fret_core::{Color, Px, Rect};
use fret_ui::Theme;
use fret_ui::element::{LayoutStyle, Length};

use super::PropertyRowOptions;
use crate::primitives::EditorDensity;
use crate::primitives::colors::editor_muted_foreground;
use crate::primitives::inspector_layout::InspectorLayoutMetrics;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PropertyRowLayoutVariant {
    #[default]
    Row,
    Column,
    /// Choose `Row` vs `Column` based on last frame bounds.
    Auto,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct PropertyRowResolvedLayout {
    pub(super) density: EditorDensity,
    pub(super) affordance_extent: Px,
    pub(super) gap: Px,
    pub(super) trailing_gap: Px,
    pub(super) reset_fg: Color,
    pub(super) auto_below: Px,
    pub(super) label_w: Px,
    pub(super) value_max_w: Px,
    pub(super) status_slot_w: Px,
    pub(super) reset_slot_w: Px,
}

pub(super) fn resolve_property_row_layout(
    theme: &Theme,
    options: &PropertyRowOptions,
    has_reset_slot: bool,
) -> PropertyRowResolvedLayout {
    let metrics = InspectorLayoutMetrics::resolve(theme);
    let density = metrics.density;
    let affordance_extent = density.affordance_extent();
    let gap = options.gap.unwrap_or(metrics.column_gap);
    let trailing_gap = options.trailing_gap.unwrap_or(metrics.trailing_gap);
    let reset_fg = editor_muted_foreground(theme);
    let auto_below = options.auto_stack_below.unwrap_or(metrics.auto_stack_below);
    let label_w = options.label_width.unwrap_or(metrics.label_width);
    let value_max_w = options.value_max_width.unwrap_or(metrics.value_max_width);
    let status_slot_w = options
        .status_slot_width
        .unwrap_or(metrics.status_slot_width);
    let status_slot_w = if status_slot_w.0 > 0.0 {
        status_slot_w.max(affordance_extent)
    } else {
        status_slot_w
    };
    let reset_slot_w = options.reset_slot_width.unwrap_or(metrics.reset_slot_width);
    let reset_slot_w = if has_reset_slot {
        reset_slot_w.max(affordance_extent)
    } else {
        reset_slot_w
    };

    PropertyRowResolvedLayout {
        density,
        affordance_extent,
        gap,
        trailing_gap,
        reset_fg,
        auto_below,
        label_w,
        value_max_w,
        status_slot_w,
        reset_slot_w,
    }
}

pub(super) fn resolve_property_row_layout_variant(
    requested: PropertyRowLayoutVariant,
    bounds: Option<Rect>,
    auto_below: Px,
) -> PropertyRowLayoutVariant {
    match requested {
        PropertyRowLayoutVariant::Row => PropertyRowLayoutVariant::Row,
        PropertyRowLayoutVariant::Column => PropertyRowLayoutVariant::Column,
        PropertyRowLayoutVariant::Auto => {
            if bounds.is_some_and(|b| b.size.width.0 > 0.0 && b.size.width.0 < auto_below.0) {
                PropertyRowLayoutVariant::Column
            } else {
                PropertyRowLayoutVariant::Row
            }
        }
    }
}

pub(super) fn apply_property_row_min_height(layout: &mut LayoutStyle, row_height: Px) {
    if layout.size.min_height.is_none() {
        layout.size.min_height = Some(Length::Px(row_height));
    }
}

#[cfg(test)]
mod tests {
    use fret_app::App;
    use fret_core::{Point, Px, Rect, Size};
    use fret_ui::Theme;
    use fret_ui::element::{LayoutStyle, Length};

    use super::{
        PropertyRowLayoutVariant, apply_property_row_min_height, resolve_property_row_layout,
        resolve_property_row_layout_variant,
    };
    use crate::composites::property_row::PropertyRowOptions;

    fn bounds(width: Px) -> Rect {
        Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(width, Px(40.0)))
    }

    #[test]
    fn property_row_auto_layout_variant_stacks_only_below_nonzero_width_threshold() {
        assert_eq!(
            resolve_property_row_layout_variant(
                PropertyRowLayoutVariant::Auto,
                Some(bounds(Px(320.0))),
                Px(520.0),
            ),
            PropertyRowLayoutVariant::Column
        );
        assert_eq!(
            resolve_property_row_layout_variant(
                PropertyRowLayoutVariant::Auto,
                Some(bounds(Px(640.0))),
                Px(520.0),
            ),
            PropertyRowLayoutVariant::Row
        );
        assert_eq!(
            resolve_property_row_layout_variant(
                PropertyRowLayoutVariant::Auto,
                Some(bounds(Px(0.0))),
                Px(520.0),
            ),
            PropertyRowLayoutVariant::Row
        );
        assert_eq!(
            resolve_property_row_layout_variant(PropertyRowLayoutVariant::Auto, None, Px(520.0)),
            PropertyRowLayoutVariant::Row
        );
    }

    #[test]
    fn property_row_resolved_layout_preserves_minimum_affordance_slots() {
        let app = App::new();
        let options = PropertyRowOptions {
            status_slot_width: Some(Px(4.0)),
            reset_slot_width: Some(Px(5.0)),
            ..Default::default()
        };
        let resolved = resolve_property_row_layout(Theme::global(&app), &options, true);

        assert_eq!(resolved.status_slot_w, resolved.affordance_extent);
        assert_eq!(resolved.reset_slot_w, resolved.affordance_extent);
    }

    #[test]
    fn property_row_min_height_applies_density_row_height_without_clobbering_override() {
        let mut layout = LayoutStyle::default();
        apply_property_row_min_height(&mut layout, Px(24.0));
        assert_eq!(layout.size.min_height, Some(Length::Px(Px(24.0))));

        layout.size.min_height = Some(Length::Px(Px(32.0)));
        apply_property_row_min_height(&mut layout, Px(24.0));
        assert_eq!(layout.size.min_height, Some(Length::Px(Px(32.0))));
    }
}
