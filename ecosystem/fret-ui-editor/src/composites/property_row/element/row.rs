//! PropertyRow row-layout branch owner.

use fret_core::{Axis, Color, Edges, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, LayoutStyle, Length,
    MainAlign, Overflow, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use super::super::PropertyRowReset;
use super::super::reset;
use super::kind_layout_mut;
use super::{mark_property_row_value_slot, property_row_land_child};
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
    let trailing_slot_leading_margin = Px(trailing_gap.0 - gap.0);

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
            let label = property_row_land_child(
                cx,
                move |cx| label(cx),
                move |layout| {
                    layout.size.width = Length::Px(label_w);
                    layout.size.height = Length::Px(density.row_height);
                    layout.size.min_height = Some(Length::Px(density.row_height));
                    layout.size.max_height = Some(Length::Px(density.row_height));
                    layout.flex = FlexItemStyle {
                        order: 0,
                        grow: 0.0,
                        shrink: 0.0,
                        basis: Length::Px(label_w),
                        align_self: None,
                    };
                    layout.overflow = Overflow::Clip;
                },
                move |cx, label| {
                    cx.container(
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
                    layout.flex = FlexItemStyle {
                        order: 0,
                        grow: 1.0,
                        shrink: 1.0,
                        basis: Length::Px(Px(0.0)),
                        align_self: None,
                    };
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
                        move |_cx| vec![value],
                    )
                },
            ));

            if !has_trailing_slots {
                return vec![label, value];
            }

            let mut out = vec![label, value];

            if has_reset_slot {
                let reset_for_slot = reset.clone();
                if let Some(reset) = reset::property_row_reset_element(
                    cx,
                    reset_for_slot,
                    affordance_extent,
                    reset_slot_w,
                    trailing_slot_leading_margin,
                    reset_fg,
                ) {
                    out.push(reset);
                }
            }

            if let Some(action_el) = actions_el {
                out.push(property_row_row_action_element(
                    cx,
                    action_el,
                    density,
                    status_slot_w,
                    trailing_slot_leading_margin,
                ));
            }

            out
        },
    )
}

fn property_row_row_action_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    mut action_el: AnyElement,
    density: EditorDensity,
    status_slot_w: Px,
    trailing_slot_leading_margin: Px,
) -> AnyElement {
    if let Some(layout) = kind_layout_mut(&mut action_el.kind)
        && let Length::Px(action_width) = layout.size.width
    {
        layout.flex = FlexItemStyle {
            order: 0,
            grow: 0.0,
            shrink: 0.0,
            basis: Length::Px(action_width),
            align_self: None,
        };
        layout.margin = fret_ui::element::MarginEdges {
            left: fret_ui::element::MarginEdge::Px(Px(
                status_slot_w.0 - action_width.0 + trailing_slot_leading_margin.0
            )),
            ..Default::default()
        };
        return action_el;
    }

    cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(status_slot_w),
                    height: Length::Auto,
                    min_height: Some(Length::Px(density.row_height)),
                    ..Default::default()
                },
                margin: fret_ui::element::MarginEdges {
                    left: fret_ui::element::MarginEdge::Px(trailing_slot_leading_margin),
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    order: 0,
                    grow: 0.0,
                    shrink: 0.0,
                    basis: Length::Px(status_slot_w),
                    align_self: None,
                },
                overflow: Overflow::Clip,
                ..Default::default()
            },
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(Px(0.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::End,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |_cx| vec![action_el],
    )
}
