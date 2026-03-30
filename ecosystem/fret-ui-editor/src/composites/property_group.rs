//! Inspector-style property group (collapsible header + section body).

use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Axis, Corners, Edges, Px, TextAlign, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, SizeStyle, SpacerProps, SpacingLength, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::typography;

use crate::primitives::colors::{
    editor_panel_background, editor_property_group_border, editor_property_header_background,
    editor_property_header_border, editor_property_header_foreground,
};
use crate::primitives::icons::editor_icon;
use crate::primitives::inspector_layout::InspectorLayoutMetrics;
use crate::primitives::visuals::hover_overlay_bg;

pub type OnPropertyGroupToggle = Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, bool) + 'static>;

#[derive(Debug, Clone)]
pub struct PropertyGroupOptions {
    pub layout: LayoutStyle,
    pub enabled: bool,
    pub collapsible: bool,
    pub default_collapsed: bool,
    pub collapsed: Option<Model<bool>>,
    pub header_height: Option<Px>,
    pub gap: Option<Px>,
    pub test_id: Option<Arc<str>>,
    pub header_test_id: Option<Arc<str>>,
    pub content_test_id: Option<Arc<str>>,
}

impl Default for PropertyGroupOptions {
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
            enabled: true,
            collapsible: true,
            default_collapsed: false,
            collapsed: None,
            header_height: None,
            gap: None,
            test_id: None,
            header_test_id: None,
            content_test_id: None,
        }
    }
}

#[derive(Clone)]
pub struct PropertyGroup {
    label: Arc<str>,
    options: PropertyGroupOptions,
    on_toggle: Option<OnPropertyGroupToggle>,
}

impl PropertyGroup {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            options: PropertyGroupOptions::default(),
            on_toggle: None,
        }
    }

    pub fn options(mut self, options: PropertyGroupOptions) -> Self {
        self.options = options;
        self
    }

    pub fn on_toggle(mut self, on_toggle: Option<OnPropertyGroupToggle>) -> Self {
        self.on_toggle = on_toggle;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        header_actions: impl FnOnce(&mut ElementContext<'_, H>) -> Option<AnyElement>,
        contents: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        cx.scope(|cx| {
            let (
                metrics,
                header_height,
                header_bg,
                header_border,
                panel_bg,
                group_border,
                radius,
                header_fg,
            ) = {
                let theme = Theme::global(&*cx.app);
                let metrics = InspectorLayoutMetrics::resolve(theme);
                let header_height = self
                    .options
                    .header_height
                    .unwrap_or(metrics.group_header_height);
                let header_bg = editor_property_header_background(theme);
                let header_border = editor_property_header_border(theme);
                let panel_bg = editor_panel_background(theme);
                let group_border = editor_property_group_border(theme);
                let radius = theme.metric_token("metric.radius.sm");
                let header_fg = editor_property_header_foreground(theme);
                (
                    metrics,
                    header_height,
                    header_bg,
                    header_border,
                    panel_bg,
                    group_border,
                    radius,
                    header_fg,
                )
            };

            let density = metrics.density;
            let gap = self.options.gap.unwrap_or(metrics.group_content_gap);

            let collapsed_model = self
                .options
                .collapsed
                .clone()
                .unwrap_or_else(|| collapsed_model(cx, self.options.default_collapsed));
            let collapsed = cx
                .get_model_copied(&collapsed_model, Invalidation::Layout)
                .unwrap_or(self.options.default_collapsed);

            let disclosure_icon = self.options.collapsible.then_some({
                if collapsed {
                    fret_icons::ids::ui::CHEVRON_RIGHT
                } else {
                    fret_icons::ids::ui::CHEVRON_DOWN
                }
            });

            let label = self.label.clone();
            let collapsible = self.options.collapsible;
            let enabled = self.options.enabled;
            let on_toggle = self.on_toggle.clone();
            let collapsed_for_toggle = collapsed_model.clone();

            let mut header = cx.pressable(
                PressableProps {
                    enabled: enabled && collapsible,
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Auto,
                            min_height: Some(Length::Px(Px(header_height.0.max(0.0)))),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    a11y: PressableA11y {
                        label: Some(Arc::from(format!("Toggle {label}"))),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                move |cx, st| {
                    let on_activate: OnActivate = Arc::new({
                        let on_toggle = on_toggle.clone();
                        let collapsed_for_toggle = collapsed_for_toggle.clone();
                        move |host, action_cx: ActionCx, _reason: ActivateReason| {
                            let next = host
                                .models_mut()
                                .update(&collapsed_for_toggle, |b| {
                                    *b = !*b;
                                    *b
                                })
                                .unwrap_or(false);
                            if let Some(cb) = on_toggle.as_ref() {
                                cb(host, action_cx, next);
                            }
                            host.request_redraw(action_cx.window);
                        }
                    });
                    cx.pressable_add_on_activate(on_activate);

                    let theme = Theme::global(&*cx.app);
                    let header_bg = hover_overlay_bg(theme, header_bg, st.hovered, st.pressed);

                    let actions = header_actions(cx);
                    vec![cx.container(
                        fret_ui::element::ContainerProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Auto,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            background: Some(header_bg),
                            corner_radii: Corners {
                                top_left: radius,
                                top_right: radius,
                                bottom_right: Px(0.0),
                                bottom_left: Px(0.0),
                            },
                            border: Edges {
                                top: Px(0.0),
                                right: Px(0.0),
                                bottom: Px(1.0),
                                left: Px(0.0),
                            },
                            border_color: Some(header_border),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![
                                cx.flex(
                                    FlexProps {
                                        layout: LayoutStyle {
                                            size: SizeStyle {
                                                width: Length::Fill,
                                                height: Length::Auto,
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        direction: Axis::Horizontal,
                                        gap: SpacingLength::Px(Px(6.0)),
                                        padding: Edges {
                                            top: Px(density.padding_y.0 + 2.0),
                                            right: density.padding_x,
                                            bottom: Px(density.padding_y.0 + 2.0),
                                            left: density.padding_x,
                                        }
                                        .into(),
                                        justify: MainAlign::Start,
                                        align: CrossAlign::Center,
                                        wrap: false,
                                    },
                                    move |cx| {
                                        let mut out = Vec::new();
                                        let header_text_style =
                                            typography::as_control_text(TextStyle {
                                                size: Px(12.0),
                                                weight: fret_core::FontWeight::SEMIBOLD,
                                                line_height: Some(header_height),
                                                ..Default::default()
                                            });
                                        if let Some(icon) = disclosure_icon.clone() {
                                            out.push(editor_icon(
                                                cx,
                                                density,
                                                icon,
                                                Some(Px(12.0)),
                                            ));
                                        }
                                        out.push(cx.text_props(TextProps {
                                            layout: LayoutStyle {
                                                size: SizeStyle {
                                                    width: Length::Fill,
                                                    height: Length::Auto,
                                                    ..Default::default()
                                                },
                                                ..Default::default()
                                            },
                                            text: label.clone(),
                                            style: Some(header_text_style),
                                            color: Some(header_fg),
                                            wrap: TextWrap::None,
                                            overflow: TextOverflow::Ellipsis,
                                            align: TextAlign::Start,
                                            ink_overflow: Default::default(),
                                        }));
                                        out.push(cx.spacer(SpacerProps::default()));
                                        if let Some(actions) = actions {
                                            out.push(actions);
                                        }
                                        out
                                    },
                                ),
                            ]
                        },
                    )]
                },
            );

            if let Some(test_id) = self.options.header_test_id.as_ref() {
                header = header.test_id(test_id.clone());
            }

            let mut out = Vec::new();
            out.push(header);

            if !collapsed || !self.options.collapsible {
                let mut content = cx.flex(
                    FlexProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Auto,
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        direction: Axis::Vertical,
                        gap: SpacingLength::Px(gap),
                        padding: Edges {
                            top: Px(density.padding_y.0 + 2.0),
                            right: density.padding_x,
                            bottom: Px(density.padding_y.0 + 4.0),
                            left: density.padding_x,
                        }
                        .into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                        wrap: false,
                    },
                    contents,
                );
                if let Some(test_id) = self.options.content_test_id.as_ref() {
                    content = content.test_id(test_id.clone());
                }
                out.push(content);
            }

            let mut root = cx.flex(
                FlexProps {
                    layout: self.options.layout,
                    direction: Axis::Vertical,
                    gap: SpacingLength::Px(Px(0.0)),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    wrap: false,
                },
                move |_cx| out,
            );

            if let Some(test_id) = self.options.test_id.as_ref() {
                root = root.test_id(test_id.clone());
            }

            cx.container(
                ContainerProps {
                    layout: self.options.layout,
                    padding: Edges::all(Px(0.0)).into(),
                    background: Some(panel_bg),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(group_border),
                    corner_radii: Corners::all(radius),
                    ..Default::default()
                },
                move |_cx| vec![root],
            )
        })
    }
}

#[track_caller]
fn collapsed_model<H: UiHost>(cx: &mut ElementContext<'_, H>, default: bool) -> Model<bool> {
    cx.local_model(move || default)
}
