//! Inspector-style property row composite (label + value + actions).
use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Axis, Color, Corners, Edges, FontWeight, Px, TextAlign, TextStyle};
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, LayoutStyle, Length,
    MainAlign, Overflow, PressableA11y, PressableProps, SizeStyle, SpacingLength, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::typography;

use crate::primitives::colors::{
    editor_border, editor_foreground, editor_muted_foreground, editor_subtle_bg,
};
use crate::primitives::inspector_layout::InspectorLayoutMetrics;
use crate::primitives::visuals::{editor_icon_button_bg, editor_icon_button_border};

pub type OnPropertyRowReset = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>;

#[derive(Debug, Clone)]
pub struct PropertyRowResetOptions {
    pub enabled: bool,
    pub glyph: Arc<str>,
    pub a11y_label: Arc<str>,
    /// Explicit identity source for reset button state and action hooks.
    ///
    /// Falls back to `test_id` when omitted, which keeps diagnostics-addressable resets stable in
    /// loop-built property grids.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
}

impl Default for PropertyRowResetOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            // ASCII fallback (avoid missing-glyph tofu on default fonts).
            glyph: Arc::from("R"),
            a11y_label: Arc::from("Reset to default"),
            id_source: None,
            test_id: None,
        }
    }
}

#[derive(Clone)]
pub struct PropertyRowReset {
    pub options: PropertyRowResetOptions,
    pub on_reset: OnPropertyRowReset,
}

impl PropertyRowReset {
    pub fn new(on_reset: OnPropertyRowReset) -> Self {
        Self {
            options: PropertyRowResetOptions::default(),
            on_reset,
        }
    }

    pub fn options(mut self, options: PropertyRowResetOptions) -> Self {
        self.options = options;
        self
    }
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
                                    height: Length::Auto,
                                    min_height: Some(Length::Px(density.row_height)),
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
                            let value = cx.container(
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
                                        overflow: Overflow::Clip,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                |cx| vec![value(cx)],
                            );

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
                                        property_row_reset_element(
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
                                            property_row_reset_element(
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

                        let value = cx.container(
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
                        );

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

fn property_row_reset_element<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    reset: Option<PropertyRowReset>,
    affordance_extent: Px,
    reset_fg: Color,
) -> Option<AnyElement> {
    let reset = reset?;
    if !reset.options.enabled {
        return None;
    }

    let glyph = reset.options.glyph.clone();
    let a11y_label = reset.options.a11y_label.clone();
    let id_source = reset
        .options
        .id_source
        .clone()
        .or_else(|| reset.options.test_id.clone());
    let test_id = reset.options.test_id.clone();
    let on_reset = reset.on_reset.clone();

    if let Some(id_source) = id_source {
        Some(cx.keyed(
            ("fret-ui-editor.property_row.reset", id_source),
            move |cx| {
                property_row_reset_pressable(
                    cx,
                    glyph,
                    a11y_label,
                    test_id,
                    on_reset,
                    affordance_extent,
                    reset_fg,
                )
            },
        ))
    } else {
        Some(property_row_reset_pressable(
            cx,
            glyph,
            a11y_label,
            test_id,
            on_reset,
            affordance_extent,
            reset_fg,
        ))
    }
}

fn property_row_reset_pressable<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    glyph: Arc<str>,
    a11y_label: Arc<str>,
    test_id: Option<Arc<str>>,
    on_reset: OnPropertyRowReset,
    affordance_extent: Px,
    reset_fg: Color,
) -> AnyElement {
    cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(affordance_extent),
                    height: Length::Px(affordance_extent),
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    order: 0,
                    grow: 0.0,
                    shrink: 0.0,
                    basis: Length::Px(affordance_extent),
                    align_self: None,
                },
                ..Default::default()
            },
            a11y: PressableA11y {
                label: Some(a11y_label),
                test_id,
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            let on_activate: OnActivate = Arc::new({
                let on_reset = on_reset.clone();
                move |host, action_cx, _reason: ActivateReason| {
                    on_reset(host, action_cx);
                    host.notify(action_cx);
                }
            });
            cx.pressable_add_on_activate(on_activate);

            let theme = Theme::global(&*cx.app);
            let hovered = st.hovered || st.hovered_raw;
            let pressed = st.pressed;
            let mut idle_bg = editor_subtle_bg(theme);
            idle_bg.a = (idle_bg.a * 0.35).clamp(0.0, 1.0);
            let idle_border = editor_border(theme);
            let bg = editor_icon_button_bg(theme, true, hovered, pressed).unwrap_or(idle_bg);
            let border =
                editor_icon_button_border(theme, true, hovered, pressed).unwrap_or(idle_border);
            let fg = if hovered || pressed {
                editor_foreground(theme)
            } else {
                reset_fg
            };

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    background: Some(bg),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border),
                    corner_radii: Corners::all(Px(6.0)),
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.text_props(TextProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Fill,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        text: glyph.clone(),
                        style: Some(typography::as_control_text(TextStyle {
                            size: Px(11.0),
                            weight: FontWeight::SEMIBOLD,
                            line_height: Some(affordance_extent),
                            ..Default::default()
                        })),
                        color: Some(fg),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                        align: TextAlign::Center,
                        ink_overflow: Default::default(),
                    })]
                },
            )]
        },
    )
}
