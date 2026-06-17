use std::sync::Arc;

use fret_core::{Axis, Color, Px};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use super::{VecEditLayoutVariant, VecEditOptions};
use crate::primitives::EditorTokenKeys;
use crate::primitives::style::EditorStyle;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy)]
pub(super) struct VecEditAxisColors {
    pub(super) x: Color,
    pub(super) y: Color,
    pub(super) z: Color,
    pub(super) w: Color,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct VecEditLayoutPlan {
    pub(super) direction: Axis,
    pub(super) grow: bool,
    pub(super) axis_colors: VecEditAxisColors,
}

pub(super) fn derived_id_source(base: Option<&Arc<str>>, suffix: &str) -> Option<Arc<str>> {
    base.map(|id| Arc::<str>::from(format!("{}.{}", id.as_ref(), suffix)))
}

pub(super) fn resolve_vec_edit_layout_plan<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    options: &VecEditOptions,
    axis_count: usize,
) -> VecEditLayoutPlan {
    let bounds = if matches!(options.variant, VecEditLayoutVariant::Auto) {
        cx.layout_query_bounds(cx.root_id(), Invalidation::Layout)
    } else {
        None
    };
    let (axis_colors, default_auto_below, axis_min_width) = {
        let theme = Theme::global(&*cx.app);
        let style = EditorStyle::resolve(theme);

        (
            VecEditAxisColors {
                x: axis_color(
                    theme,
                    EditorTokenKeys::AXIS_X_COLOR,
                    Color::from_srgb_hex_rgb(0xf2_59_59),
                ),
                y: axis_color(
                    theme,
                    EditorTokenKeys::AXIS_Y_COLOR,
                    Color::from_srgb_hex_rgb(0x59_f2_59),
                ),
                z: axis_color(
                    theme,
                    EditorTokenKeys::AXIS_Z_COLOR,
                    Color::from_srgb_hex_rgb(0x59_8c_f2),
                ),
                w: axis_color(
                    theme,
                    EditorTokenKeys::AXIS_W_COLOR,
                    Color::from_srgb_hex_rgb(0xcc_cc_cc),
                ),
            },
            style.vec_auto_stack_below,
            style.vec_axis_min_width,
        )
    };
    let requested_auto_below = options.auto_stack_below.unwrap_or(default_auto_below);
    let auto_below = Px(requested_auto_below
        .0
        .max(minimum_auto_stack_width(axis_min_width, options.gap, axis_count).0));
    let variant = resolve_vec_edit_variant(
        options.variant,
        bounds.map(|bounds| bounds.size.width),
        auto_below,
    );
    let direction = direction_for_variant(variant);

    VecEditLayoutPlan {
        direction,
        grow: variant == VecEditLayoutVariant::Row,
        axis_colors,
    }
}

fn axis_color(theme: &Theme, key: &'static str, fallback: Color) -> Color {
    theme.color_by_key(key).unwrap_or(fallback)
}

fn direction_for_variant(variant: VecEditLayoutVariant) -> Axis {
    match variant {
        VecEditLayoutVariant::Row => Axis::Horizontal,
        VecEditLayoutVariant::Column => Axis::Vertical,
        VecEditLayoutVariant::Auto => Axis::Horizontal,
    }
}

fn minimum_auto_stack_width(axis_min_width: Px, gap: Px, axis_count: usize) -> Px {
    let axis_count = axis_count.max(1) as f32;
    let gap_count = (axis_count - 1.0).max(0.0);
    Px(axis_min_width.0 * axis_count + gap.0 * gap_count)
}

fn resolve_vec_edit_variant(
    variant: VecEditLayoutVariant,
    bounds_width: Option<Px>,
    auto_below: Px,
) -> VecEditLayoutVariant {
    match variant {
        VecEditLayoutVariant::Row => VecEditLayoutVariant::Row,
        VecEditLayoutVariant::Column => VecEditLayoutVariant::Column,
        VecEditLayoutVariant::Auto => {
            if bounds_width.is_some_and(|width| width.0 > 0.0 && width.0 < auto_below.0) {
                VecEditLayoutVariant::Column
            } else {
                VecEditLayoutVariant::Row
            }
        }
    }
}
