//! Inspector-style property row composite (label + value + actions).
mod layout;
mod reset;

use std::sync::Arc;

use fret_core::{Axis, Edges, Px};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, LayoutStyle, Length,
    MainAlign, Overflow, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::primitives::colors::editor_muted_foreground;
use crate::primitives::inspector_layout::InspectorLayoutMetrics;
use crate::primitives::readout::editor_property_row_label_text_props;

pub use layout::PropertyRowLayoutVariant;
use layout::{
    PropertyRowResolvedLayout, apply_property_row_min_height, resolve_property_row_layout,
    resolve_property_row_layout_variant,
};
pub use reset::{OnPropertyRowReset, PropertyRowReset, PropertyRowResetOptions};

#[cfg(test)]
const PROPERTY_ROW_VALUE_SLOT: &str = "fret-ui-editor.property-row.value";

#[cfg(test)]
fn mark_property_row_value_slot(element: AnyElement) -> AnyElement {
    element.component_slot(PROPERTY_ROW_VALUE_SLOT)
}

#[cfg(not(test))]
fn mark_property_row_value_slot(element: AnyElement) -> AnyElement {
    element
}

pub(crate) fn property_row_label_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: impl Into<Arc<str>>,
) -> AnyElement {
    let (fg, row_height) = {
        let theme = Theme::global(&*cx.app);
        let metrics = InspectorLayoutMetrics::resolve(theme);
        (editor_muted_foreground(theme), metrics.density.row_height)
    };

    cx.text_props(editor_property_row_label_text_props(
        text.into(),
        fg,
        row_height,
    ))
}

#[derive(Debug, Clone)]
pub struct PropertyRowOptions {
    pub layout: LayoutStyle,
    pub label_width: Option<Px>,
    pub gap: Option<Px>,
    pub trailing_gap: Option<Px>,
    pub value_max_width: Option<Px>,
    pub status_slot_width: Option<Px>,
    pub reset_slot_width: Option<Px>,
    pub variant: PropertyRowLayoutVariant,
    pub auto_stack_below: Option<Px>,
    /// Explicit identity source for internal policy state (auto layout heuristics).
    ///
    /// This is the editor-composite equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    /// Use this when building rows in a loop where the callsite is not unique per row.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
}

impl Default for PropertyRowOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            label_width: None,
            gap: None,
            trailing_gap: None,
            value_max_width: None,
            status_slot_width: None,
            reset_slot_width: None,
            variant: PropertyRowLayoutVariant::Row,
            auto_stack_below: None,
            id_source: None,
            test_id: None,
        }
    }
}

#[derive(Clone, Default)]
pub struct PropertyRow {
    pub options: PropertyRowOptions,
    pub reset: Option<PropertyRowReset>,
}

impl PropertyRow {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn options(mut self, options: PropertyRowOptions) -> Self {
        self.options = options;
        self
    }

    pub fn reset(mut self, reset: Option<PropertyRowReset>) -> Self {
        self.reset = reset;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        label: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        value: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        actions: impl FnOnce(&mut ElementContext<'_, H>) -> Option<AnyElement>,
    ) -> AnyElement {
        let id_source = self.options.id_source.clone();
        if let Some(id_source) = id_source.as_deref() {
            // Only key when the caller provides an explicit identity source. Keying by callsite
            // alone breaks loop-built rows by collapsing them into a single element identity.
            cx.keyed(("fret-ui-editor.property_row", id_source), move |cx| {
                self.into_element_inner(cx, label, value, actions)
            })
        } else {
            self.into_element_inner(cx, label, value, actions)
        }
    }

    fn into_element_inner<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        label: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        value: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        actions: impl FnOnce(&mut ElementContext<'_, H>) -> Option<AnyElement>,
    ) -> AnyElement {
        let bounds = cx.layout_query_bounds(cx.root_id(), Invalidation::Layout);

        let has_reset_slot = self
            .reset
            .as_ref()
            .is_some_and(|reset| reset.options.enabled);

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
        } = resolve_property_row_layout(Theme::global(&*cx.app), &self.options, has_reset_slot);

        let variant = resolve_property_row_layout_variant(self.options.variant, bounds, auto_below);

        let mut layout = self.options.layout;
        apply_property_row_min_height(&mut layout, density.row_height);

        let reset = self.reset.clone();

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
                                out.push(cx.flex(
                                    FlexProps {
                                        layout: LayoutStyle {
                                            size: SizeStyle {
                                                width: Length::Px(reset_slot_w),
                                                height: Length::Auto,
                                                min_height: Some(Length::Px(density.row_height)),
                                                ..Default::default()
                                            },
                                            flex: FlexItemStyle {
                                                order: 0,
                                                grow: 0.0,
                                                shrink: 0.0,
                                                basis: Length::Px(reset_slot_w),
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
                                out.push(cx.flex(
                                    FlexProps {
                                        layout: LayoutStyle {
                                            size: SizeStyle {
                                                width: Length::Px(status_slot_w),
                                                height: Length::Auto,
                                                min_height: Some(Length::Px(density.row_height)),
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
                                    out.push(cx.flex(
                                        FlexProps {
                                            layout: LayoutStyle {
                                                size: SizeStyle {
                                                    width: Length::Px(reset_slot_w),
                                                    height: Length::Auto,
                                                    min_height: Some(Length::Px(
                                                        density.row_height,
                                                    )),
                                                    ..Default::default()
                                                },
                                                flex: FlexItemStyle {
                                                    order: 0,
                                                    grow: 0.0,
                                                    shrink: 0.0,
                                                    basis: Length::Px(reset_slot_w),
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
                                    out.push(cx.flex(
                                        FlexProps {
                                            layout: LayoutStyle {
                                                size: SizeStyle {
                                                    width: Length::Px(status_slot_w),
                                                    height: Length::Auto,
                                                    min_height: Some(Length::Px(
                                                        density.row_height,
                                                    )),
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

        if let Some(test_id) = self.options.test_id.as_ref() {
            row.test_id(test_id.clone())
        } else {
            row
        }
    }
}

#[cfg(test)]
mod tests;
