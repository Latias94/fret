//! VecEdit keyed element assembly owner.

use std::sync::Arc;

use fret_core::{Axis, Color, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, MainAlign, SpacingLength};
use fret_ui::{ElementContext, UiHost};

use crate::controls::{NumericFormatFn, NumericParseFn, NumericValidateFn};
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::input_group::derived_test_id;

use super::super::VecEditOptions;
use super::super::axis::{AxisReset, OnVecEditAxisOutcome, VecEditAxis, axis_group};
use super::super::layout::{VecEditAxisColors, derived_id_source, resolve_vec_edit_layout_plan};

pub(super) struct VecEditElementAxis<T> {
    axis: VecEditAxis,
    id_suffix: &'static str,
    label: Arc<str>,
    model: Model<T>,
    reset: Option<AxisReset>,
}

impl<T> VecEditElementAxis<T> {
    pub(super) fn new(
        axis: VecEditAxis,
        id_suffix: &'static str,
        label: &'static str,
        model: Model<T>,
        reset: Option<AxisReset>,
    ) -> Self {
        Self {
            axis,
            id_suffix,
            label: Arc::from(label),
            model,
            reset,
        }
    }
}

pub(super) struct VecEditElementArgs<T> {
    pub(super) options: VecEditOptions,
    pub(super) axes: Vec<VecEditElementAxis<T>>,
    pub(super) format: NumericFormatFn<T>,
    pub(super) parse: NumericParseFn<T>,
    pub(super) validate: Option<NumericValidateFn<T>>,
    pub(super) on_axis_outcome: Option<OnVecEditAxisOutcome>,
}

struct VecEditAxisMount<T> {
    axis: VecEditAxis,
    reset: Option<AxisReset>,
    id_source: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    label: Arc<str>,
    color: Color,
    model: Model<T>,
}

pub(super) fn vec_edit_element<H: UiHost, T>(
    cx: &mut ElementContext<'_, H>,
    args: VecEditElementArgs<T>,
) -> AnyElement
where
    T: DragValueScalar + Default,
{
    let VecEditElementArgs {
        options,
        axes,
        format,
        parse,
        validate,
        on_axis_outcome,
    } = args;

    let layout = resolve_vec_edit_layout_plan(cx, &options, axes.len());
    let grow = layout.grow;
    let direction = layout.direction;
    let axis_gap = options.axis_gap;
    let prefix = options.prefix.clone();
    let suffix = options.suffix.clone();
    let root_test_id = options.test_id.clone();
    let axis_mounts: Vec<_> = axes
        .into_iter()
        .map(|axis| VecEditAxisMount {
            axis: axis.axis,
            reset: axis.reset,
            id_source: derived_id_source(options.id_source.as_ref(), axis.id_suffix),
            test_id: derived_test_id(options.test_id.as_ref(), axis.id_suffix),
            label: axis.label,
            color: axis_color(layout.axis_colors, axis.axis),
            model: axis.model,
        })
        .collect();

    let mut el = cx.flex(
        FlexProps {
            layout: options.layout,
            direction,
            gap: SpacingLength::Px(options.gap),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: if direction == Axis::Horizontal {
                CrossAlign::Center
            } else {
                CrossAlign::Stretch
            },
            wrap: false,
        },
        move |cx| {
            axis_mounts
                .into_iter()
                .map(|axis| {
                    axis_group(
                        cx,
                        axis.axis,
                        axis_gap,
                        axis.reset,
                        grow,
                        axis.id_source,
                        axis.test_id,
                        axis.label,
                        axis.color,
                        axis.model,
                        prefix.clone(),
                        suffix.clone(),
                        format.clone(),
                        parse.clone(),
                        validate.clone(),
                        on_axis_outcome.clone(),
                    )
                })
                .collect::<Vec<_>>()
        },
    );

    if let Some(test_id) = root_test_id.as_ref() {
        el = el.test_id(test_id.clone());
    }
    el
}

fn axis_color(colors: VecEditAxisColors, axis: VecEditAxis) -> Color {
    match axis {
        VecEditAxis::X => colors.x,
        VecEditAxis::Y => colors.y,
        VecEditAxis::Z => colors.z,
        VecEditAxis::W => colors.w,
    }
}
