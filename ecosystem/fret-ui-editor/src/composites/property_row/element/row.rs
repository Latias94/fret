//! PropertyRow row-layout branch owner.

use fret_core::{Axis, Color, Edges, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, LayoutStyle, Length,
    MainAlign, Overflow, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use super::super::PropertyRowReset;
use super::super::reset;
use super::super::slot::property_row_trailing_slot;
use super::mark_property_row_value_slot;
use crate::primitives::EditorDensity;

pub(super) struct PropertyRowRowElementOptions {
    pub(super) layout: LayoutStyle,
    pub(super) density: EditorDensity,
    pub(super) affordance_extent: Px,
    pub(super) gap: Px,
    pub(super) trailing_gap: Px,
    pub(super) reset_fg: Color,
    pub(super) label_w: Px,
    pub(super) value_max_w: Px,
    pub(super) status_slot_w: Px,
    pub(super) reset_slot_w: Px,
    pub(super) has_reset_slot: bool,
    pub(super) reset: Option<PropertyRowReset>,
    pub(super) actions_el: Option<AnyElement>,
}

pub(super) fn property_row_row_element<H, Label, Value>(
    cx: &mut ElementContext<'_, H>,
    options: PropertyRowRowElementOptions,
    label: Label,
    value: Value,
) -> AnyElement
where
    H: UiHost,
    Label: FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    Value: FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
{
    let PropertyRowRowElementOptions {
        layout,
        density,
        affordance_extent,
        gap,
        trailing_gap,
        reset_fg,
        label_w,
        value_max_w,
        status_slot_w,
        reset_slot_w,
        has_reset_slot,
        reset,
        actions_el,
    } = options;

    let has_trailing_slots = has_reset_slot || actions_el.is_some();

    cx.flex(
        FlexProps {
            layout,
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(gap),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |cx| {
            let label = cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(label_w),
                            height: Length::Px(density.row_height),
                            min_height: Some(Length::Px(density.row_height)),
                            max_height: Some(Length::Px(density.row_height)),
                            ..Default::default()
                        },
                        flex: FlexItemStyle {
                            order: 0,
                            grow: 0.0,
                            shrink: 0.0,
                            basis: Length::Px(label_w),
                            align_self: None,
                        },
                        overflow: Overflow::Clip,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |cx| vec![label(cx)],
            );

            let value = mark_property_row_value_slot(cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            min_height: Some(Length::Px(density.row_height)),
                            max_width: Some(Length::Px(value_max_w)),
                            ..Default::default()
                        },
                        flex: FlexItemStyle {
                            order: 0,
                            grow: 1.0,
                            shrink: 1.0,
                            basis: Length::Px(Px(0.0)),
                            align_self: None,
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                |cx| vec![value(cx)],
            ));

            if !has_trailing_slots {
                return vec![label, value];
            }

            let body = cx.flex(
                FlexProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            min_height: Some(Length::Px(density.row_height)),
                            ..Default::default()
                        },
                        flex: FlexItemStyle {
                            order: 0,
                            grow: 1.0,
                            shrink: 1.0,
                            basis: Length::Px(Px(0.0)),
                            align_self: None,
                        },
                        ..Default::default()
                    },
                    direction: Axis::Horizontal,
                    gap: SpacingLength::Px(trailing_gap),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |cx| {
                    let mut out = vec![value];

                    if has_reset_slot {
                        let reset_for_slot = reset.clone();
                        out.push(property_row_trailing_slot(
                            cx,
                            reset_slot_w,
                            density.row_height,
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
                            move |_cx| vec![action_el],
                        ));
                    }

                    out
                },
            );

            vec![label, body]
        },
    )
}
