use fret_core::{Axis, Edges, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, LayoutStyle, Length,
    MainAlign, Overflow, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use super::layout::{
    PropertyRowResolvedLayout, apply_property_row_min_height, resolve_property_row_layout,
    resolve_property_row_layout_variant,
};
use super::reset;
use super::slot::property_row_trailing_slot;
use super::{PropertyRowLayoutVariant, PropertyRowOptions, PropertyRowReset};

#[cfg(test)]
pub(crate) const PROPERTY_ROW_VALUE_SLOT: &str = "fret-ui-editor.property-row.value";

#[cfg(test)]
fn mark_property_row_value_slot(element: AnyElement) -> AnyElement {
    element.component_slot(PROPERTY_ROW_VALUE_SLOT)
}

#[cfg(not(test))]
fn mark_property_row_value_slot(element: AnyElement) -> AnyElement {
    element
}

pub(super) fn property_row_element<H, Label, Value, Actions>(
    cx: &mut ElementContext<'_, H>,
    options: PropertyRowOptions,
    reset: Option<PropertyRowReset>,
    label: Label,
    value: Value,
    actions: Actions,
) -> AnyElement
where
    H: UiHost,
    Label: FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    Value: FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    Actions: FnOnce(&mut ElementContext<'_, H>) -> Option<AnyElement>,
{
    let bounds = cx.layout_query_bounds(cx.root_id(), Invalidation::Layout);

    let has_reset_slot = reset.as_ref().is_some_and(|reset| reset.options.enabled);

    let PropertyRowResolvedLayout {
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
    } = resolve_property_row_layout(Theme::global(&*cx.app), &options, has_reset_slot);

    let variant = resolve_property_row_layout_variant(options.variant, bounds, auto_below);

    let mut layout = options.layout;
    apply_property_row_min_height(&mut layout, density.row_height);

    let actions_el = actions(cx);
    let has_action_slot = actions_el.is_some();
    let status_slot_w = if has_action_slot {
        status_slot_w
    } else {
        Px(0.0)
    };
    let reset_slot_w = if has_reset_slot {
        reset_slot_w
    } else {
        Px(0.0)
    };

    let row = match variant {
        PropertyRowLayoutVariant::Row => cx.flex(
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
        ),
        PropertyRowLayoutVariant::Column => {
            let header_gap = trailing_gap;
            let stack_gap = Px(density.padding_y.0.max(4.0));

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
                            let label = cx.container(
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
                                |cx| vec![label(cx)],
                            );

                            let mut out = vec![label];

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
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |cx| vec![value(cx)],
                    ));

                    vec![header, value]
                },
            )
        }
        PropertyRowLayoutVariant::Auto => unreachable!("auto is resolved above"),
    };

    if let Some(test_id) = options.test_id.as_ref() {
        row.test_id(test_id.clone())
    } else {
        row
    }
}
