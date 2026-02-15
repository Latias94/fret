//! Inspector-style property row composite (label + value + actions).

use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Axis, Edges, Px, TextAlign, TextStyle};
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, LayoutStyle, Length,
    MainAlign, Overflow, PressableA11y, PressableProps, SizeStyle, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::{EditorDensity, EditorTokenKeys};

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
        let (density, gap, reset_fg) = {
            let theme = Theme::global(&*cx.app);
            let density = EditorDensity::resolve(theme);
            let gap = self
                .options
                .gap
                .or_else(|| theme.metric_by_key(EditorTokenKeys::PROPERTY_COLUMN_GAP))
                .unwrap_or(Px(8.0));
            let reset_fg = theme
                .color_by_key("muted-foreground")
                .or_else(|| theme.color_by_key("muted_foreground"))
                .unwrap_or_else(|| theme.color_token("foreground"));

            (density, gap, reset_fg)
        };

        let label_w = self.options.label_width.unwrap_or(Px(160.0));
        let mut layout = self.options.layout;
        if layout.size.min_height.is_none() {
            layout.size.min_height = Some(density.row_height);
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
                move |cx, _st| {
                    let on_activate: OnActivate = Arc::new({
                        let on_reset = on_reset.clone();
                        move |host, action_cx, _reason: ActivateReason| {
                            on_reset(host, action_cx);
                        }
                    });
                    cx.pressable_add_on_activate(on_activate);

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
                        style: Some(TextStyle {
                            // Keep this conservative: allow the theme's defaults to dominate.
                            size: Px(12.0),
                            line_height: Some(density.hit_thickness),
                            ..Default::default()
                        }),
                        color: Some(reset_fg),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                        align: TextAlign::Center,
                    })]
                },
            );

            if let Some(test_id) = reset.options.test_id.as_ref() {
                el = el.test_id(test_id.clone());
            }
            Some(el)
        });

        let row = cx.flex(
            FlexProps {
                layout,
                direction: Axis::Horizontal,
                gap,
                padding: Edges::all(Px(0.0)),
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
                                min_height: Some(density.row_height),
                                ..Default::default()
                            },
                            flex: FlexItemStyle {
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
                                min_height: Some(density.row_height),
                                ..Default::default()
                            },
                            flex: FlexItemStyle {
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
        );

        if let Some(test_id) = self.options.test_id.as_ref() {
            row.test_id(test_id.clone())
        } else {
            row
        }
    }
}
