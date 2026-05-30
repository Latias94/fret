//! Inspector-style property row composite (label + value + actions).
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PropertyRowLayoutVariant {
    #[default]
    Row,
    Column,
    /// Choose `Row` vs `Column` based on last frame bounds.
    Auto,
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

        let (
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
        ) = {
            let theme = Theme::global(&*cx.app);
            let metrics = InspectorLayoutMetrics::resolve(theme);
            let density = metrics.density;
            let affordance_extent = density.affordance_extent();
            let gap = self.options.gap.unwrap_or(metrics.column_gap);
            let trailing_gap = self.options.trailing_gap.unwrap_or(metrics.trailing_gap);
            let reset_fg = editor_muted_foreground(theme);
            let auto_below = self
                .options
                .auto_stack_below
                .unwrap_or(metrics.auto_stack_below);
            let label_w = self.options.label_width.unwrap_or(metrics.label_width);
            let value_max_w = self
                .options
                .value_max_width
                .unwrap_or(metrics.value_max_width);
            let status_slot_w = self
                .options
                .status_slot_width
                .unwrap_or(metrics.status_slot_width);
            let status_slot_w = if status_slot_w.0 > 0.0 {
                status_slot_w.max(affordance_extent)
            } else {
                status_slot_w
            };
            let reset_slot_w = self
                .options
                .reset_slot_width
                .unwrap_or(metrics.reset_slot_width);
            let reset_slot_w = if has_reset_slot {
                reset_slot_w.max(affordance_extent)
            } else {
                reset_slot_w
            };

            (
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
            )
        };

        let variant = match self.options.variant {
            PropertyRowLayoutVariant::Row => PropertyRowLayoutVariant::Row,
            PropertyRowLayoutVariant::Column => PropertyRowLayoutVariant::Column,
            PropertyRowLayoutVariant::Auto => {
                if bounds.is_some_and(|b| b.size.width.0 > 0.0 && b.size.width.0 < auto_below.0) {
                    PropertyRowLayoutVariant::Column
                } else {
                    PropertyRowLayoutVariant::Row
                }
            }
        };

        let mut layout = self.options.layout;
        if layout.size.min_height.is_none() {
            layout.size.min_height = Some(Length::Px(density.row_height));
        }

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
mod tests {
    use std::sync::{Arc, Mutex};

    use fret_app::App;
    use fret_core::{AppWindowId, Color, Point, Px, Rect, Size, TextStyle};
    use fret_ui::element::{AnyElement, ElementKind, Overflow};
    use fret_ui::elements::GlobalElementId;
    use fret_ui::{Theme, UiTree, declarative};

    use super::{
        PROPERTY_ROW_VALUE_SLOT, PropertyRow, PropertyRowLayoutVariant, PropertyRowOptions,
        property_row_label_text,
    };
    use crate::primitives::inspector_layout::InspectorLayoutMetrics;
    use crate::primitives::readout::editor_validation_message_text_props;
    use crate::test_support::WrappingTextServices;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(180.0), Px(120.0)),
        )
    }

    fn find_component_slot<'a>(element: &'a AnyElement, slot: &str) -> Option<&'a AnyElement> {
        if element.component_slot.as_deref() == Some(slot) {
            return Some(element);
        }
        element
            .children
            .iter()
            .find_map(|child| find_component_slot(child, slot))
    }

    #[test]
    fn row_value_slot_keeps_overflow_visible_for_wrapping_value_children() {
        let mut app = App::new();
        let window = AppWindowId::default();
        let row =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "property-row", |cx| {
                PropertyRow::new()
                    .options(PropertyRowOptions {
                        variant: PropertyRowLayoutVariant::Row,
                        test_id: Some(Arc::from("inspector.exposure")),
                        ..Default::default()
                    })
                    .into_element(
                        cx,
                        |cx| property_row_label_text(cx, "Exposure"),
                        |cx| {
                            cx.text_props(editor_validation_message_text_props(
                                Arc::from(
                                    "Value must stay between 0.0 and 1.0 for this render target.",
                                ),
                                Color::from_srgb_hex_rgb(0xCC_44_44),
                                TextStyle::default(),
                            ))
                        },
                        |_cx| None,
                    )
            });

        let value_slot = find_component_slot(&row, PROPERTY_ROW_VALUE_SLOT)
            .expect("property row should mark its value slot for contract tests");
        let ElementKind::Container(props) = &value_slot.kind else {
            panic!(
                "property row value slot should be a container, got {:?}",
                value_slot.kind
            );
        };

        assert_eq!(
            props.layout.overflow,
            Overflow::Visible,
            "row value slot must let wrapping value children grow and paint inside their measured line boxes; fixed chrome slots may clip themselves"
        );
    }

    #[test]
    fn row_label_slot_keeps_fixed_line_box_when_label_text_wraps_under_narrow_layout() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let mut services = WrappingTextServices;
        let row_id = Arc::new(Mutex::new(None::<GlobalElementId>));
        let expected_row_height = Arc::new(Mutex::new(None::<Px>));

        let row_id_for_render = Arc::clone(&row_id);
        let expected_row_height_for_render = Arc::clone(&expected_row_height);
        let root = declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds(),
            "property-row-wrapping-label-layout",
            move |cx| {
                let metrics = InspectorLayoutMetrics::resolve(Theme::global(&*cx.app));
                *expected_row_height_for_render.lock().unwrap() = Some(metrics.density.row_height);

                let row = PropertyRow::new()
                    .options(PropertyRowOptions {
                        variant: PropertyRowLayoutVariant::Row,
                        label_width: Some(Px(48.0)),
                        gap: Some(Px(8.0)),
                        trailing_gap: Some(Px(0.0)),
                        value_max_width: Some(Px(1024.0)),
                        test_id: Some(Arc::from("inspector.long-label")),
                        ..Default::default()
                    })
                    .into_element(
                        cx,
                        |cx| {
                            cx.text(
                                "Very long property label that would normally wrap under resize",
                            )
                        },
                        |cx| cx.text("0.50"),
                        |_cx| None,
                    );
                *row_id_for_render.lock().unwrap() = Some(row.id);
                vec![row]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds(), 1.0);

        let row_id = row_id.lock().unwrap().expect("row id");
        let expected_row_height = expected_row_height
            .lock()
            .unwrap()
            .expect("expected row height");
        let row_bounds = fret_ui::elements::current_bounds_for_element(&mut app, window, row_id)
            .expect("row bounds");

        assert!(
            row_bounds.size.height.0 <= expected_row_height.0 + 0.5,
            "property-row label chrome must not grow fixed-height rows when bare/default text wraps under resize: row={row_bounds:?} expected_row_height={expected_row_height:?}"
        );
    }

    #[test]
    fn row_value_slot_grows_to_wrapping_value_text_under_narrow_layout() {
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        let window = AppWindowId::default();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(180.0), Px(120.0)),
        );
        let mut services = WrappingTextServices;
        let row_id = Arc::new(std::sync::Mutex::new(None::<GlobalElementId>));
        let value_slot_id = Arc::new(std::sync::Mutex::new(None::<GlobalElementId>));
        let validation_text_id = Arc::new(std::sync::Mutex::new(None::<GlobalElementId>));

        let row_id_for_render = Arc::clone(&row_id);
        let value_slot_id_for_render = Arc::clone(&value_slot_id);
        let validation_text_id_for_render = Arc::clone(&validation_text_id);
        let root = declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "property-row-wrapping-value-layout",
            move |cx| {
                let row = PropertyRow::new()
                    .options(PropertyRowOptions {
                        variant: PropertyRowLayoutVariant::Row,
                        label_width: Some(Px(104.0)),
                        gap: Some(Px(8.0)),
                        trailing_gap: Some(Px(0.0)),
                        value_max_width: Some(Px(1024.0)),
                        test_id: Some(Arc::from("inspector.exposure")),
                        ..Default::default()
                    })
                    .into_element(
                        cx,
                        |cx| property_row_label_text(cx, "Exposure"),
                        |cx| {
                            let text = cx.text_props(editor_validation_message_text_props(
                                Arc::from(
                                    "Value must stay between 0.0 and 1.0 for this render target.",
                                ),
                                Color::from_srgb_hex_rgb(0xCC_44_44),
                                TextStyle::default(),
                            ));
                            *validation_text_id_for_render.lock().unwrap() = Some(text.id);
                            text
                        },
                        |_cx| None,
                    );

                let value_slot = find_component_slot(&row, PROPERTY_ROW_VALUE_SLOT)
                    .expect("property row should mark its value slot for layout tests");
                *row_id_for_render.lock().unwrap() = Some(row.id);
                *value_slot_id_for_render.lock().unwrap() = Some(value_slot.id);

                vec![row]
            },
        );
        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let row_id = row_id.lock().unwrap().expect("row id");
        let value_slot_id = value_slot_id.lock().unwrap().expect("value slot id");
        let validation_text_id = validation_text_id
            .lock()
            .unwrap()
            .expect("validation text id");

        let row_bounds = fret_ui::elements::current_bounds_for_element(&mut app, window, row_id)
            .expect("row bounds");
        let value_bounds =
            fret_ui::elements::current_bounds_for_element(&mut app, window, value_slot_id)
                .expect("value slot bounds");
        let text_bounds =
            fret_ui::elements::current_bounds_for_element(&mut app, window, validation_text_id)
                .expect("validation text bounds");

        assert!(
            text_bounds.size.height.0 > 28.0,
            "validation text should wrap to multiple measured lines under narrow layout: {text_bounds:?}"
        );
        assert!(
            value_bounds.size.height.0 + 0.5 >= text_bounds.size.height.0,
            "value slot should grow to contain wrapping validation text: value={value_bounds:?} text={text_bounds:?}"
        );
        assert!(
            row_bounds.size.height.0 + 0.5 >= value_bounds.size.height.0,
            "property row should grow to contain its value slot: row={row_bounds:?} value={value_bounds:?}"
        );
        assert!(
            text_bounds.origin.y.0 + text_bounds.size.height.0
                <= value_bounds.origin.y.0 + value_bounds.size.height.0 + 0.5,
            "validation text bottom should stay inside value slot bottom: value={value_bounds:?} text={text_bounds:?}"
        );
    }
}
