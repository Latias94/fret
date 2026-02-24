//! Inspector-style property group (collapsible header + section body).

use std::sync::Arc;

use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Axis, Corners, Edges, Px, TextAlign, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, SizeStyle, SpacerProps, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::typography;

use crate::primitives::icons::editor_icon;
use crate::primitives::visuals::hover_overlay_bg;
use crate::primitives::{EditorDensity, EditorTokenKeys};

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
                density,
                header_height,
                header_bg,
                header_border,
                panel_bg,
                panel_border,
                radius,
                header_fg,
            ) = {
                let theme = Theme::global(&*cx.app);
                let density = EditorDensity::resolve(theme);
                let header_height = self
                    .options
                    .header_height
                    .or_else(|| theme.metric_by_key(EditorTokenKeys::PROPERTY_GROUP_HEADER_HEIGHT))
                    .unwrap_or(density.row_height);
                let header_bg = theme
                    .color_by_key("muted")
                    .or_else(|| theme.color_by_key("component.card.bg"))
                    .unwrap_or_else(|| theme.color_token("background"));
                let header_border = theme
                    .color_by_key("border")
                    .or_else(|| theme.color_by_key("component.card.border"))
                    .unwrap_or_else(|| theme.color_token("foreground"));
                let panel_bg = theme
                    .color_by_key("card")
                    .or_else(|| theme.color_by_key("component.card.bg"))
                    .unwrap_or_else(|| theme.color_token("background"));
                let panel_border = theme
                    .color_by_key("border")
                    .or_else(|| theme.color_by_key("component.card.border"))
                    .unwrap_or_else(|| theme.color_token("foreground"));
                let radius = theme.metric_token("metric.radius.sm");
                let header_fg = theme
                    .color_by_key("foreground")
                    .unwrap_or_else(|| theme.color_token("foreground"));
                (
                    density,
                    header_height,
                    header_bg,
                    header_border,
                    panel_bg,
                    panel_border,
                    radius,
                    header_fg,
                )
            };

            let gap = self.options.gap.unwrap_or(Px(4.0));

            let collapsed_model = self
                .options
                .collapsed
                .clone()
                .unwrap_or_else(|| collapsed_model(cx, self.options.default_collapsed));
            let collapsed = cx
                .get_model_copied(&collapsed_model, Invalidation::Layout)
                .unwrap_or(self.options.default_collapsed);

            let disclosure_icon = self.options.collapsible.then(|| {
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
                    let header_bg = hover_overlay_bg(&theme, header_bg, st.hovered, st.pressed);

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
                            vec![cx.flex(
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
                                    gap: Px(6.0),
                                    padding: Edges::symmetric(density.padding_x, density.padding_y),
                                    justify: MainAlign::Start,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                },
                                move |cx| {
                                    let mut out = Vec::new();
                                    let header_text_style =
                                        typography::as_control_text(TextStyle {
                                            size: Px(12.0),
                                            line_height: Some(header_height),
                                            ..Default::default()
                                        });
                                    if let Some(icon) = disclosure_icon.clone() {
                                        out.push(editor_icon(cx, density, icon, Some(Px(12.0))));
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
                            )]
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
                        gap,
                        padding: Edges::symmetric(density.padding_x, density.padding_y),
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
                    gap: Px(0.0),
                    padding: Edges::all(Px(0.0)),
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
                    padding: Edges::all(Px(0.0)),
                    background: Some(panel_bg),
                    border: Edges::all(Px(1.0)),
                    border_color: Some(panel_border),
                    corner_radii: Corners::all(radius),
                    ..Default::default()
                },
                move |_cx| vec![root],
            )
        })
    }
}

fn collapsed_model<H: UiHost>(cx: &mut ElementContext<'_, H>, default: bool) -> Model<bool> {
    let m = cx.with_state(|| None::<Model<bool>>, |st| st.clone());
    match m {
        Some(m) => m,
        None => {
            let m = cx.app.models_mut().insert(default);
            cx.with_state(
                || None::<Model<bool>>,
                |st| {
                    if st.is_none() {
                        *st = Some(m.clone());
                    }
                },
            );
            m
        }
    }
}
