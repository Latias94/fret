//! PropertyRow column-layout branch owner.

use fret_core::{Axis, Color, Edges, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, LayoutStyle, Length,
    MainAlign, Overflow, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use super::super::PropertyRowReset;
use super::super::reset;
use super::super::slot::property_row_trailing_slot;
use super::{mark_property_row_value_slot, property_row_land_child};
use crate::primitives::EditorDensity;

pub(super) struct PropertyRowColumnElementOptions {
    pub(super) layout: LayoutStyle,
    pub(super) density: EditorDensity,
    pub(super) affordance_extent: Px,
    pub(super) trailing_gap: Px,
    pub(super) reset_fg: Color,
    pub(super) value_max_w: Px,
    pub(super) status_slot_w: Px,
    pub(super) reset_slot_w: Px,
    pub(super) has_reset_slot: bool,
    pub(super) reset: Option<PropertyRowReset>,
    pub(super) actions_el: Option<AnyElement>,
}

pub(super) fn property_row_column_element<H, Label, Value>(
    cx: &mut ElementContext<'_, H>,
    options: PropertyRowColumnElementOptions,
    label: Label,
    value: Value,
) -> AnyElement
where
    H: UiHost,
    Label: FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    Value: FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
{
    let PropertyRowColumnElementOptions {
        layout,
        density,
        affordance_extent,
        trailing_gap,
        reset_fg,
        value_max_w,
        status_slot_w,
        reset_slot_w,
        has_reset_slot,
        reset,
        actions_el,
    } = options;

    let header_gap = trailing_gap;
    let stack_gap = Px(density.padding_y.0.max(4.0));
    let has_trailing_slots = has_reset_slot || actions_el.is_some();

    let label = property_row_land_child(
        cx,
        move |cx| label(cx),
        move |layout| {
            layout.size.width = Length::Fill;
            layout.size.height = Length::Px(density.row_height);
            layout.size.min_height = Some(Length::Px(density.row_height));
            layout.size.max_height = Some(Length::Px(density.row_height));
            layout.flex = FlexItemStyle {
                order: 0,
                grow: 1.0,
                shrink: 1.0,
                basis: Length::Px(Px(0.0)),
                align_self: None,
            };
            layout.overflow = Overflow::Clip;
        },
        move |cx, label| {
            cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Px(density.row_height),
                            min_height: Some(Length::Px(density.row_height)),
                            max_height: Some(Length::Px(density.row_height)),
                            ..Default::default()
                        },
                        flex: FlexItemStyle {
                            order: 0,
                            grow: 1.0,
                            shrink: 1.0,
                            basis: Length::Px(Px(0.0)),
                            align_self: None,
                        },
                        overflow: Overflow::Clip,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |_cx| vec![label],
            )
        },
    );

    let value = mark_property_row_value_slot(property_row_land_child(
        cx,
        move |cx| value(cx),
        move |layout| {
            layout.size.width = Length::Fill;
            layout.size.height = Length::Auto;
            layout.size.min_height = Some(Length::Px(density.row_height));
            layout.size.max_width = Some(Length::Px(value_max_w));
        },
        move |cx, value| {
            cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            min_height: Some(Length::Px(density.row_height)),
                            max_width: Some(Length::Px(value_max_w)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |_cx| vec![value],
            )
        },
    ));

    cx.flex(
        FlexProps {
            layout,
            direction: Axis::Vertical,
            gap: SpacingLength::Px(stack_gap),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            if !has_trailing_slots {
                return vec![label, value];
            }

            let header = cx.flex(
                FlexProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            min_height: Some(Length::Px(density.row_height)),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    direction: Axis::Horizontal,
                    gap: SpacingLength::Px(header_gap),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |cx| {
                    let mut out = vec![label];

                    if has_reset_slot {
                        let reset_for_slot = reset.clone();
                        out.push(property_row_trailing_slot(
                            cx,
                            reset_slot_w,
                            density.row_height,
                            Px(0.0),
                            move |cx| {
                                reset::property_row_reset_element(
                                    cx,
                                    reset_for_slot.clone(),
                                    affordance_extent,
                                    reset_fg,
                                )
                                .into_iter()
                                .collect::<Vec<AnyElement>>()
                            },
                        ));
                    }

                    if let Some(action_el) = actions_el {
                        out.push(property_row_trailing_slot(
                            cx,
                            status_slot_w,
                            density.row_height,
                            Px(0.0),
                            move |_cx| vec![action_el],
                        ));
                    }

                    out
                },
            );

            vec![header, value]
        },
    )
}
