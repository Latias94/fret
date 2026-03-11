//! Inspector-style property row composite (label + value + actions).
use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Axis, Corners, Edges, Px, TextAlign, TextStyle};
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, LayoutStyle, Length,
    MainAlign, Overflow, PressableA11y, PressableProps, SizeStyle, SpacingLength, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::typography;

use crate::primitives::EditorTokenKeys;
use crate::primitives::style::EditorStyle;
use crate::primitives::visuals::{editor_icon_button_bg, editor_icon_button_border};

pub type OnPropertyRowReset = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx) + 'static>;

#[derive(Debug, Clone)]
pub struct PropertyRowResetOptions {
    pub enabled: bool,
    pub glyph: Arc<str>,
    pub a11y_label: Arc<str>,
    pub test_id: Option<Arc<str>>,
}

impl Default for PropertyRowResetOptions {
    fn default() -> Self {
        Self {
            enabled: true,
            // ASCII fallback (avoid missing-glyph tofu on default fonts).
            glyph: Arc::from("R"),
            a11y_label: Arc::from("Reset to default"),
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
            variant: PropertyRowLayoutVariant::Row,
            auto_stack_below: None,
            id_source: None,
            test_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyRowLayoutVariant {
    Row,
    Column,
    /// Choose `Row` vs `Column` based on last frame bounds.
    Auto,
}

impl Default for PropertyRowLayoutVariant {
    fn default() -> Self {
        Self::Row
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

        let (density, gap, reset_fg, auto_below, label_w) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            let density = style.density;
            let gap = self
                .options
                .gap
                .or_else(|| theme.metric_by_key(EditorTokenKeys::PROPERTY_COLUMN_GAP))
                .unwrap_or(Px(8.0));
            let reset_fg = theme
                .color_by_key("muted-foreground")
                .or_else(|| theme.color_by_key("muted_foreground"))
                .unwrap_or_else(|| theme.color_token("foreground"));
            let auto_below = self
                .options
                .auto_stack_below
                .unwrap_or(style.property_auto_stack_below);
            let label_w = self
                .options
                .label_width
                .or_else(|| theme.metric_by_key(EditorTokenKeys::PROPERTY_LABEL_WIDTH))
                .unwrap_or(Px(160.0));

            (density, gap, reset_fg, auto_below, label_w)
        };

        let variant = match self.options.variant {
            PropertyRowLayoutVariant::Row => PropertyRowLayoutVariant::Row,
            PropertyRowLayoutVariant::Column => PropertyRowLayoutVariant::Column,
            PropertyRowLayoutVariant::Auto => bounds
                .is_some_and(|b| b.size.width.0 > 0.0 && b.size.width.0 < auto_below.0)
                .then_some(PropertyRowLayoutVariant::Column)
                .unwrap_or(PropertyRowLayoutVariant::Row),
        };

        let mut layout = self.options.layout;
        if layout.size.min_height.is_none() {
            layout.size.min_height = Some(Length::Px(density.row_height));
        }

        let reset = self.reset.clone();
        let reset_el = reset.and_then(|reset| {
            if !reset.options.enabled {
                return None;
            }

            let glyph = reset.options.glyph.clone();
            let a11y_label = reset.options.a11y_label.clone();
            let on_reset = reset.on_reset.clone();

            let mut el = cx.pressable(
                PressableProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(density.hit_thickness),
                            height: Length::Px(density.hit_thickness),
                            ..Default::default()
                        },
                        flex: FlexItemStyle {
                            order: 0,
                            grow: 0.0,
                            shrink: 0.0,
                            basis: Length::Px(density.hit_thickness),
                            align_self: None,
                        },
                        ..Default::default()
                    },
                    a11y: PressableA11y {
                        label: Some(a11y_label),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |cx, st| {
                    let on_activate: OnActivate = Arc::new({
                        let on_reset = on_reset.clone();
                        move |host, action_cx, _reason: ActivateReason| {
                            on_reset(host, action_cx);
                        }
                    });
                    cx.pressable_add_on_activate(on_activate);

                    let theme = Theme::global(&*cx.app);
                    let hovered = st.hovered || st.hovered_raw;
                    let pressed = st.pressed;
                    let bg = editor_icon_button_bg(theme, true, hovered, pressed);
                    let border = editor_icon_button_border(theme, true, hovered, pressed);
                    let border_width = if border.is_some() { Px(1.0) } else { Px(0.0) };

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
                            background: bg,
                            border: Edges::all(border_width),
                            border_color: border,
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
                                    // Keep this conservative: allow the theme's defaults to dominate.
                                    size: Px(12.0),
                                    line_height: Some(density.hit_thickness),
                                    ..Default::default()
                                })),
                                color: Some(reset_fg),
                                wrap: TextWrap::None,
                                overflow: TextOverflow::Clip,
                                align: TextAlign::Center,
                                ink_overflow: Default::default(),
                            })]
                        },
                    )]
                },
            );

            if let Some(test_id) = reset.options.test_id.as_ref() {
                el = el.test_id(test_id.clone());
            }
            Some(el)
        });

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

                    let value = cx.container(
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
                                    basis: Length::Fill,
                                    align_self: None,
                                },
                                overflow: Overflow::Clip,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        |cx| vec![value(cx)],
                    );

                    let mut out = vec![label, value];
                    if let Some(reset) = reset_el {
                        out.push(reset);
                    }
                    if let Some(actions) = actions(cx) {
                        out.push(actions);
                    }
                    out
                },
            ),
            PropertyRowLayoutVariant::Column => {
                let header_gap = Px(6.0);
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
                                if let Some(reset) = reset_el {
                                    out.push(reset);
                                }
                                if let Some(actions) = actions(cx) {
                                    out.push(actions);
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
