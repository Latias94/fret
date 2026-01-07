use std::sync::Arc;

use fret_core::{Color, Corners, Edges, FontId, FontWeight, Point, Px, SemanticsRole, TextStyle};
use fret_core::{TextWrap, Transform2D};
use fret_runtime::{CommandId, Model};
use fret_ui::element::{
    AnyElement, ContainerProps, FlexProps, LayoutStyle, Length, MainAlign, OpacityProps,
    PointerRegionProps, PressableA11y, PressableProps, SizeStyle, StackProps, TextProps,
    VisualTransformProps,
};
use fret_ui::overlay_placement::{Align, LayoutDirection, Side};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::navigation_menu as radix_navigation_menu;
use fret_ui_kit::primitives::presence as radix_presence;
use fret_ui_kit::primitives::{popper, popper_content};
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, OverlayPresence, Radius, Space};

use crate::overlay_motion;

fn shadcn_zoom_transform(origin: Point, scale: f32) -> Transform2D {
    Transform2D::translation(origin)
        * Transform2D::scale_uniform(scale)
        * Transform2D::translation(Point::new(Px(-origin.x.0), Px(-origin.y.0)))
}

fn nav_menu_trigger_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.navigation_menu.trigger.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.navigation_menu.trigger.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_required("font.line_height"));
    TextStyle {
        font: FontId::default(),
        size: px,
        weight: FontWeight::MEDIUM,
        slant: Default::default(),
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

fn nav_menu_trigger_padding_x(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.navigation_menu.trigger.pad_x")
        .unwrap_or_else(|| MetricRef::space(Space::N3).resolve(theme))
}

fn nav_menu_trigger_padding_y(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.navigation_menu.trigger.pad_y")
        .unwrap_or_else(|| MetricRef::space(Space::N2).resolve(theme))
}

fn nav_menu_trigger_radius(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.navigation_menu.trigger.radius")
        .unwrap_or_else(|| MetricRef::radius(Radius::Md).resolve(theme))
}

fn nav_menu_trigger_bg_hover(theme: &Theme) -> Color {
    theme.color_required("accent")
}

fn nav_menu_trigger_fg(theme: &Theme) -> Color {
    theme.color_required("foreground")
}

fn nav_menu_trigger_fg_muted(theme: &Theme) -> Color {
    theme.color_required("muted-foreground")
}

fn nav_menu_viewport_bg(theme: &Theme) -> Color {
    theme.color_required("popover")
}

fn nav_menu_viewport_border(theme: &Theme) -> Color {
    theme.color_required("border")
}

fn nav_menu_viewport_side_offset(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.navigation_menu.viewport.side_offset")
        .unwrap_or_else(|| MetricRef::space(Space::N2).resolve(theme))
}

fn nav_menu_viewport_window_margin(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.navigation_menu.viewport.window_margin")
        .unwrap_or(Px(8.0))
}

fn nav_menu_content_switch_slide_px(theme: &Theme) -> Px {
    // Matches shadcn/ui's `slide-*-52` distance (13rem ≈ 208px).
    theme
        .metric_by_key("component.navigation_menu.content.switch_slide_px")
        .unwrap_or(Px(208.0))
}

fn nav_menu_indicator_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.navigation_menu.indicator.size")
        .unwrap_or(Px(14.0))
}

/// shadcn/ui `NavigationMenuTrigger` (v4).
///
/// In the upstream DOM implementation this is an element; in Fret this is a "spec" that provides
/// trigger children for [`NavigationMenuItem`].
#[derive(Debug, Clone, Default)]
pub struct NavigationMenuTrigger {
    children: Vec<AnyElement>,
}

impl NavigationMenuTrigger {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self { children }
    }

    pub fn child(child: AnyElement) -> Self {
        Self {
            children: vec![child],
        }
    }

    pub fn children(self) -> Vec<AnyElement> {
        self.children
    }
}

/// shadcn/ui `NavigationMenuLink` (v4).
///
/// In the upstream DOM implementation this is an element that participates in Radix's
/// root-dismiss-on-select behavior. Fret does not use implicit context objects, so this wrapper
/// requires the navigation menu `model` and closes it on selection (unless the click is modified
/// with Ctrl/Meta, matching Radix semantics).
#[derive(Debug, Clone)]
pub struct NavigationMenuLink {
    model: Model<Option<Arc<str>>>,
    children: Vec<AnyElement>,
    label: Option<Arc<str>>,
    command: Option<CommandId>,
    disabled: bool,
    dismiss_on_ctrl_or_meta: bool,
}

impl NavigationMenuLink {
    pub fn new(model: Model<Option<Arc<str>>>, children: Vec<AnyElement>) -> Self {
        Self {
            model,
            children,
            label: None,
            command: None,
            disabled: false,
            dismiss_on_ctrl_or_meta: false,
        }
    }

    pub fn child(model: Model<Option<Arc<str>>>, child: AnyElement) -> Self {
        Self::new(model, vec![child])
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn on_click(mut self, command: impl Into<CommandId>) -> Self {
        self.command = Some(command.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// When `false` (default), activation with Ctrl/Meta pressed does not dismiss the menu.
    pub fn dismiss_on_ctrl_or_meta(mut self, dismiss_on_ctrl_or_meta: bool) -> Self {
        self.dismiss_on_ctrl_or_meta = dismiss_on_ctrl_or_meta;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        #[derive(Default)]
        struct ModifierState {
            suppress_dismiss_for_next_activate: bool,
        }

        let model = self.model.clone();
        let disabled = self.disabled;
        let command = self.command;
        let label = self.label.clone();
        let children = std::rc::Rc::new(self.children);
        let dismiss_on_ctrl_or_meta = self.dismiss_on_ctrl_or_meta;

        cx.pressable_with_id_props(move |cx, _st, link_id| {
            let modifier_state: Arc<std::sync::Mutex<ModifierState>> = cx.with_state_for(
                link_id,
                || Arc::new(std::sync::Mutex::new(ModifierState::default())),
                |s| s.clone(),
            );

            let modifier_state_for_pointer = modifier_state.clone();
            cx.pressable_add_on_pointer_down(Arc::new(move |_host, _cx, down| {
                use fret_ui::action::PressablePointerDownResult as R;

                let suppress =
                    (down.modifiers.ctrl || down.modifiers.meta) && !dismiss_on_ctrl_or_meta;
                let mut st = modifier_state_for_pointer
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                st.suppress_dismiss_for_next_activate = suppress;
                R::Continue
            }));

            let modifier_state_for_activate = modifier_state.clone();
            let model_for_activate = model.clone();
            cx.pressable_add_on_activate(Arc::new(move |host, action_cx, _reason| {
                if disabled {
                    return;
                }

                if let Some(command) = command.as_ref() {
                    host.dispatch_command(Some(action_cx.window), command.clone());
                }

                let mut st = modifier_state_for_activate
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                let suppress = st.suppress_dismiss_for_next_activate;
                st.suppress_dismiss_for_next_activate = false;
                if suppress {
                    return;
                }

                let _ = host.models_mut().update(&model_for_activate, |v| *v = None);
            }));

            let mut pressable = PressableProps::default();
            pressable.enabled = !disabled;
            pressable.focusable = !disabled;
            pressable.a11y = PressableA11y {
                role: Some(SemanticsRole::Button),
                label: label.clone(),
                ..Default::default()
            };

            (pressable, children.as_ref().clone())
        })
    }
}

/// shadcn/ui `NavigationMenuContent` (v4).
///
/// In the upstream DOM implementation this is an element; in Fret this is a "spec" that provides
/// viewport content for [`NavigationMenuItem`].
#[derive(Debug, Clone, Default)]
pub struct NavigationMenuContent {
    children: Vec<AnyElement>,
}

impl NavigationMenuContent {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self { children }
    }

    pub fn child(child: AnyElement) -> Self {
        Self {
            children: vec![child],
        }
    }

    pub fn children(self) -> Vec<AnyElement> {
        self.children
    }
}

#[derive(Debug, Clone)]
pub struct NavigationMenuItem {
    value: Arc<str>,
    label: Arc<str>,
    content: Vec<AnyElement>,
    trigger: Option<Vec<AnyElement>>,
    disabled: bool,
}

impl NavigationMenuItem {
    pub fn new(
        value: impl Into<Arc<str>>,
        label: impl Into<Arc<str>>,
        content: Vec<AnyElement>,
    ) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            content,
            trigger: None,
            disabled: false,
        }
    }

    pub fn trigger_children(mut self, children: Vec<AnyElement>) -> Self {
        self.trigger = Some(children);
        self
    }

    pub fn trigger(mut self, trigger: NavigationMenuTrigger) -> Self {
        self.trigger = Some(trigger.children());
        self
    }

    pub fn trigger_child(mut self, child: AnyElement) -> Self {
        self.trigger = Some(vec![child]);
        self
    }

    pub fn content(mut self, content: NavigationMenuContent) -> Self {
        self.content = content.children();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// shadcn/ui `NavigationMenuList` (v4).
///
/// In the upstream DOM implementation this is a structural wrapper. In Fret it is a named
/// container for `NavigationMenuItem` specs so recipes read closer to shadcn docs.
#[derive(Debug, Clone, Default)]
pub struct NavigationMenuList {
    items: Vec<NavigationMenuItem>,
}

impl NavigationMenuList {
    pub fn new(items: Vec<NavigationMenuItem>) -> Self {
        Self { items }
    }

    pub fn items(mut self, items: Vec<NavigationMenuItem>) -> Self {
        self.items = items;
        self
    }

    pub fn into_items(self) -> Vec<NavigationMenuItem> {
        self.items
    }
}

#[derive(Clone)]
pub struct NavigationMenu {
    model: Option<Model<Option<Arc<str>>>>,
    default_value: Option<Arc<str>>,
    items: Vec<NavigationMenuItem>,
    disabled: bool,
    viewport: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    config: radix_navigation_menu::NavigationMenuConfig,
}

impl std::fmt::Debug for NavigationMenu {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NavigationMenu")
            .field("model", &"<model>")
            .field("items_len", &self.items.len())
            .field("disabled", &self.disabled)
            .field("viewport", &self.viewport)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .field("config", &self.config)
            .finish()
    }
}

impl NavigationMenu {
    pub fn new(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model: Some(model),
            default_value: None,
            items: Vec::new(),
            disabled: false,
            viewport: true,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            config: radix_navigation_menu::NavigationMenuConfig::default(),
        }
    }

    pub fn uncontrolled<T: Into<Arc<str>>>(default_value: Option<T>) -> Self {
        Self {
            model: None,
            default_value: default_value.map(Into::into),
            items: Vec::new(),
            disabled: false,
            viewport: true,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            config: radix_navigation_menu::NavigationMenuConfig::default(),
        }
    }

    pub fn items(mut self, items: Vec<NavigationMenuItem>) -> Self {
        self.items = items;
        self
    }

    pub fn list(mut self, list: NavigationMenuList) -> Self {
        self.items = list.into_items();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// When `true` (default), content is presented through a shared "viewport" panel with
    /// size interpolation, matching shadcn/ui composition.
    ///
    /// When `false`, content uses its own measured size without viewport interpolation (closer to
    /// Radix's "no Viewport component mounted" behavior).
    pub fn viewport(mut self, viewport: bool) -> Self {
        self.viewport = viewport;
        self
    }

    pub fn refine_chrome(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn config(mut self, config: radix_navigation_menu::NavigationMenuConfig) -> Self {
        self.config = config;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let controlled_model = self.model;
        let default_value = self.default_value;
        let items = self.items;
        let menu_disabled = self.disabled;
        let viewport_enabled = self.viewport;
        let chrome = self.chrome;
        let layout = self.layout;
        let cfg = self.config;

        let value_model =
            radix_navigation_menu::navigation_menu_use_value_model(cx, controlled_model, || {
                default_value.clone()
            })
            .model();

        let theme = Theme::global(&*cx.app).clone();

        let trigger_pad_x = nav_menu_trigger_padding_x(&theme);
        let trigger_pad_y = nav_menu_trigger_padding_y(&theme);
        let trigger_radius = nav_menu_trigger_radius(&theme);
        let trigger_bg_hover = nav_menu_trigger_bg_hover(&theme);
        let trigger_fg = nav_menu_trigger_fg(&theme);
        let trigger_fg_muted = nav_menu_trigger_fg_muted(&theme);
        let trigger_text_style = nav_menu_trigger_text_style(&theme);

        let viewport_bg = nav_menu_viewport_bg(&theme);
        let viewport_border = nav_menu_viewport_border(&theme);
        let viewport_radius = theme
            .metric_by_key("component.navigation_menu.viewport.radius")
            .unwrap_or_else(|| MetricRef::radius(Radius::Lg).resolve(&theme));
        let viewport_pad = theme
            .metric_by_key("component.navigation_menu.viewport.padding")
            .unwrap_or_else(|| MetricRef::space(Space::N4).resolve(&theme));
        let root_gap = MetricRef::space(Space::N3).resolve(&theme);
        let content_switch_slide_px = nav_menu_content_switch_slide_px(&theme);

        let root_props = decl_style::container_props(&theme, chrome, layout);

        cx.container(root_props, move |cx| {
            let root_id = cx.root_id();
            let nav_ctx = radix_navigation_menu::NavigationMenuRoot::new(value_model.clone())
                .config(cfg)
                .disabled(menu_disabled)
                .context(cx, root_id);
            let root_state = nav_ctx.root_state.clone();

            #[derive(Default)]
            struct OpenModelState {
                model: Option<Model<bool>>,
            }

            #[derive(Default)]
            struct SelectionSyncState {
                last_selected: Option<Arc<str>>,
            }

            let open_model =
                cx.with_state_for(root_id, OpenModelState::default, |st| st.model.clone());
            let open_model = if let Some(model) = open_model {
                model
            } else {
                let model = cx.app.models_mut().insert(false);
                cx.with_state_for(root_id, OpenModelState::default, |st| {
                    st.model = Some(model.clone());
                });
                model
            };

            let selected: Option<Arc<str>> =
                cx.watch_model(&value_model).layout().cloned().flatten();
            let selected_changed = cx.with_state_for(root_id, SelectionSyncState::default, |st| {
                let changed = selected != st.last_selected;
                if changed {
                    st.last_selected = selected.clone();
                }
                changed
            });

            if selected_changed {
                let selected = selected.clone();
                let _ = cx
                    .app
                    .models_mut()
                    .update(&open_model, |v| *v = selected.is_some());
            }

            let open: bool = cx
                .watch_model(&open_model)
                .layout()
                .copied()
                .unwrap_or(false);
            let open_for_motion = open && selected.is_some();
            let motion = radix_presence::scale_fade_presence_with_durations_and_easing(
                cx,
                open_for_motion,
                overlay_motion::SHADCN_MOTION_TICKS_100,
                overlay_motion::SHADCN_MOTION_TICKS_100,
                0.95,
                1.0,
                overlay_motion::shadcn_ease,
            );

            let mut selected_local = radix_navigation_menu::navigation_menu_viewport_selected_value(
                cx,
                root_id,
                selected.clone(),
                motion.present,
            );

            if !open_for_motion && selected.is_some() && !motion.present {
                let mut host = fret_ui::action::UiActionHostAdapter { app: &mut *cx.app };
                let action_cx = fret_ui::action::ActionCx {
                    window: cx.window,
                    target: root_id,
                };
                let mut st = root_state.lock().unwrap_or_else(|e| e.into_inner());
                st.on_item_dismiss(&mut host, action_cx, &value_model, cfg);
                selected_local = None;
            }

            let active_idx = selected_local.as_deref().and_then(|v| {
                items
                    .iter()
                    .position(|it| it.value.as_ref() == v)
                    .filter(|_| !menu_disabled)
            });

            let values: Vec<Arc<str>> = items.iter().map(|it| it.value.clone()).collect();
            let transition = radix_navigation_menu::navigation_menu_content_transition(
                cx,
                root_id,
                open_for_motion,
                selected.clone(),
                &values,
            );

            let list_props = FlexProps {
                layout: LayoutStyle::default(),
                direction: fret_core::Axis::Horizontal,
                gap: Px(0.0),
                padding: Edges::all(Px(0.0)),
                justify: MainAlign::Start,
                align: fret_ui::element::CrossAlign::Center,
                wrap: false,
                ..Default::default()
            };

            let items_for_children = items.clone();
            let value_for_viewport = value_model.clone();
            let trigger_text_style_for_list = trigger_text_style.clone();
            let nav_ctx_for_list = nav_ctx.clone();

            let list = cx.flex(list_props, move |cx| {
                items_for_children
                    .iter()
                    .map(|item| {
                        let item = item.clone();
                        let item_value = item.value.clone();
                        let label = item.label.clone();
                        let disabled = menu_disabled || item.disabled;
                        let trigger_text_style_for_item = trigger_text_style_for_list.clone();
                        let nav_ctx_for_item = nav_ctx_for_list.clone();

                        cx.keyed(item_value.clone(), |cx| {
                            let trigger_text_style = trigger_text_style_for_item.clone();

                            let mut pressable = PressableProps::default();
                            pressable.enabled = !disabled;
                            pressable.focusable = !disabled;
                            pressable.a11y = PressableA11y {
                                role: Some(SemanticsRole::Button),
                                label: Some(label.clone()),
                                ..Default::default()
                            };

                            let pointer_props = PointerRegionProps {
                                layout: LayoutStyle::default(),
                                enabled: true,
                            };

                            let trigger_children = item.trigger.clone();
                            let item_label = item.label.clone();
                            radix_navigation_menu::NavigationMenuTrigger::new(item_value.clone())
                                .label(label.clone())
                                .disabled(disabled)
                                .into_element(
                                    cx,
                                    &nav_ctx_for_item,
                                    pressable,
                                    pointer_props,
                                    move |cx, st, is_open| {
                                        let hovered = st.hovered && !st.pressed;
                                        let pressed = st.pressed;
                                        let fg = if disabled {
                                            trigger_fg_muted
                                        } else {
                                            trigger_fg
                                        };
                                        let bg = (hovered || pressed || is_open)
                                            .then_some(trigger_bg_hover);

                                        let mut layout = LayoutStyle::default();
                                        layout.size.width = Length::Auto;

                                        let wrapper = ContainerProps {
                                            layout,
                                            padding: Edges {
                                                top: trigger_pad_y,
                                                right: trigger_pad_x,
                                                bottom: trigger_pad_y,
                                                left: trigger_pad_x,
                                            },
                                            background: bg,
                                            shadow: None,
                                            border: Edges::all(Px(0.0)),
                                            border_color: None,
                                            corner_radii: Corners::all(trigger_radius),
                                        };

                                        let content_children =
                                            trigger_children.clone().unwrap_or_else(|| {
                                                vec![cx.text_props(TextProps {
                                                    layout: LayoutStyle::default(),
                                                    text: item_label.clone(),
                                                    style: Some(trigger_text_style.clone()),
                                                    color: Some(fg),
                                                    wrap: TextWrap::None,
                                                    overflow: fret_core::TextOverflow::Clip,
                                                })]
                                            });

                                        vec![cx.container(wrapper, move |_cx| content_children)]
                                    },
                                )
                        })
                    })
                    .collect()
            });

            let viewport = active_idx
                .and_then(|idx| items.get(idx))
                .map(|active| active.content.clone())
                .unwrap_or_default();

            let has_content = !viewport.is_empty();
            let is_open = selected_local.is_some() && has_content && open_for_motion;
            let overlay_presence = OverlayPresence {
                present: motion.present && has_content,
                interactive: is_open,
            };

            let content_switch = radix_navigation_menu::navigation_menu_content_switch(transition)
                .map(|sw| {
                    let from_children = items
                        .get(sw.from_idx)
                        .map(|it| it.content.clone())
                        .unwrap_or_default();
                    (sw.progress, sw.forward, from_children)
                });

            if overlay_presence.present {
                let side_offset = nav_menu_viewport_side_offset(&theme);
                let window_margin = nav_menu_viewport_window_margin(&theme);
                let indicator_size = nav_menu_indicator_size(&theme);

                let estimated = fret_core::Size::new(Px(320.0), Px(240.0));
                let measured = selected_local
                    .as_deref()
                    .and_then(|value| {
                        radix_navigation_menu::navigation_menu_viewport_content_id(
                            cx, root_id, value,
                        )
                    })
                    .and_then(|id| cx.last_bounds_for_element(id).map(|r| r.size));
                if viewport_enabled {
                    if let (Some(selected_value), Some(size)) = (selected_local.clone(), measured) {
                        radix_navigation_menu::navigation_menu_register_viewport_size(
                            cx,
                            root_id,
                            selected_value,
                            size,
                        );
                    }
                }

                let fallback = measured.unwrap_or(estimated);
                let content_size = if viewport_enabled {
                    radix_navigation_menu::navigation_menu_viewport_size_for_transition(
                        cx,
                        root_id,
                        selected_local.clone(),
                        &values,
                        transition,
                        fallback,
                    )
                    .size
                } else {
                    fallback
                };

                let root_state_for_viewport = root_state.clone();
                let value_for_hover = value_for_viewport.clone();
                let viewport_children = viewport.clone();
                let content_switch = content_switch.clone();
                let content_switch_slide_px = content_switch_slide_px;

                let viewport_props = if viewport_enabled {
                    ContainerProps {
                        layout: LayoutStyle {
                            overflow: fret_ui::element::Overflow::Clip,
                            ..Default::default()
                        },
                        padding: Edges::all(viewport_pad),
                        background: Some(viewport_bg),
                        shadow: None,
                        border: Edges::all(Px(1.0)),
                        border_color: Some(viewport_border),
                        corner_radii: Corners::all(viewport_radius),
                    }
                } else {
                    ContainerProps {
                        layout: LayoutStyle {
                            overflow: fret_ui::element::Overflow::Visible,
                            ..Default::default()
                        },
                        padding: Edges::all(Px(0.0)),
                        background: None,
                        shadow: None,
                        border: Edges::all(Px(0.0)),
                        border_color: None,
                        corner_radii: Corners::all(Px(0.0)),
                    }
                };

                let placement = popper::PopperContentPlacement::new(
                    LayoutDirection::Ltr,
                    Side::Bottom,
                    Align::Start,
                    side_offset,
                );

                let args = radix_navigation_menu::NavigationMenuViewportOverlayRequestArgs {
                    window_margin,
                    placement,
                    content_size,
                    indicator_size,
                };

                let opacity = motion.opacity;
                let scale = motion.scale;
                let selected_value_for_content_id = selected_local.clone();
                let selected_for_overlay = selected_local.clone();
                radix_navigation_menu::navigation_menu_request_viewport_overlay(
                    cx,
                    root_id,
                    open_model.clone(),
                    overlay_presence,
                    selected_for_overlay.as_deref(),
                    args,
                    move |cx, layout| {
                        let root_state_for_hover = root_state_for_viewport.clone();
                        let value_for_hover = value_for_hover.clone();
                        let viewport_props = viewport_props;
                        let viewport_children = viewport_children;
                        let content_switch = content_switch;
                        let content_switch_slide_px = content_switch_slide_px;

                        let content = cx.pressable(
                            PressableProps {
                                layout: LayoutStyle::default(),
                                enabled: true,
                                focusable: false,
                                focus_ring: None,
                                a11y: PressableA11y::default(),
                            },
                            move |cx, _st| {
                                let root_state_for_hover = root_state_for_hover.clone();
                                let value_for_hover = value_for_hover.clone();
                                cx.pressable_on_hover_change(Arc::new(
                                    move |host, action_cx, hovered| {
                                        let mut root = root_state_for_hover
                                            .lock()
                                            .unwrap_or_else(|e| e.into_inner());
                                        if hovered {
                                            root.on_content_enter(host);
                                        } else {
                                            root.on_content_leave(
                                                host,
                                                action_cx,
                                                &value_for_hover,
                                                cfg,
                                            );
                                        }
                                    },
                                ));

                                vec![cx.container(viewport_props, move |cx| {
                                    let Some((t, forward, from_children)) = content_switch.clone()
                                    else {
                                        return viewport_children.clone();
                                    };

                                    let to_children = viewport_children.clone();
                                    let t = t.clamp(0.0, 1.0);
                                    let slide = content_switch_slide_px.0;

                                    let (from_dx, to_dx) = if forward {
                                        (-slide * t, slide * (1.0 - t))
                                    } else {
                                        (slide * t, -slide * (1.0 - t))
                                    };

                                    let mut layout = LayoutStyle::default();
                                    layout.size = SizeStyle {
                                        width: Length::Fill,
                                        height: Length::Fill,
                                        ..Default::default()
                                    };
                                    layout.overflow = fret_ui::element::Overflow::Clip;
                                    let layout_for_layers = layout;

                                    vec![cx.stack_props(
                                        StackProps {
                                            layout: layout_for_layers,
                                        },
                                        move |cx| {
                                            let mut layer_layout = LayoutStyle::default();
                                            layer_layout.size = SizeStyle {
                                                width: Length::Fill,
                                                height: Length::Fill,
                                                ..Default::default()
                                            };

                                            let from_opacity = 1.0 - t;
                                            let to_opacity = t;

                                            let from = cx.opacity_props(
                                                OpacityProps {
                                                    layout: layer_layout,
                                                    opacity: from_opacity,
                                                },
                                                move |cx| {
                                                    let layer = cx.visual_transform_props(
                                                        VisualTransformProps {
                                                            layout: layer_layout,
                                                            transform: Transform2D::translation(
                                                                Point::new(Px(from_dx), Px(0.0)),
                                                            ),
                                                        },
                                                        move |_cx| from_children.clone(),
                                                    );
                                                    vec![layer]
                                                },
                                            );

                                            let to = cx.opacity_props(
                                                OpacityProps {
                                                    layout: layer_layout,
                                                    opacity: to_opacity,
                                                },
                                                move |cx| {
                                                    let layer = cx.visual_transform_props(
                                                        VisualTransformProps {
                                                            layout: layer_layout,
                                                            transform: Transform2D::translation(
                                                                Point::new(Px(to_dx), Px(0.0)),
                                                            ),
                                                        },
                                                        move |_cx| to_children.clone(),
                                                    );
                                                    vec![layer]
                                                },
                                            );

                                            vec![from, to]
                                        },
                                    )]
                                })]
                            },
                        );

                        if let Some(selected_value) = selected_value_for_content_id.clone() {
                            radix_navigation_menu::navigation_menu_register_viewport_content_id(
                                cx,
                                root_id,
                                selected_value,
                                content.id,
                            );
                        }

                        let transform = shadcn_zoom_transform(layout.transform_origin, scale);

                        let panel = popper_content::popper_wrapper_panel_at(
                            cx,
                            layout.placed,
                            Edges::all(Px(0.0)),
                            fret_ui::element::Overflow::Visible,
                            move |_cx| vec![content],
                        );

                        let indicator = popper_content::popper_wrapper_panel_at(
                            cx,
                            layout.indicator_rect,
                            Edges::all(Px(0.0)),
                            fret_ui::element::Overflow::Visible,
                            move |cx| {
                                let mut layout = LayoutStyle::default();
                                layout.size = SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Fill,
                                    ..Default::default()
                                };

                                let center = Point::new(
                                    Px(indicator_size.0 * 0.5),
                                    Px(indicator_size.0 * 0.5),
                                );
                                let rotate = Transform2D::rotation_about_degrees(45.0, center);

                                vec![cx.visual_transform_props(
                                    VisualTransformProps {
                                        layout,
                                        transform: rotate,
                                    },
                                    move |cx| {
                                        let mut layout = LayoutStyle::default();
                                        layout.size = SizeStyle {
                                            width: Length::Fill,
                                            height: Length::Fill,
                                            ..Default::default()
                                        };
                                        vec![cx.container(
                                            ContainerProps {
                                                layout,
                                                padding: Edges::all(Px(0.0)),
                                                background: Some(viewport_bg),
                                                shadow: None,
                                                border: Edges::all(Px(1.0)),
                                                border_color: Some(viewport_border),
                                                corner_radii: Corners::all(Px(2.0)),
                                            },
                                            |_cx| Vec::new(),
                                        )]
                                    },
                                )]
                            },
                        );

                        radix_navigation_menu::NavigationMenuViewportOverlayRenderOutput {
                            opacity,
                            transform,
                            children: vec![indicator, panel],
                        }
                    },
                );
            }

            vec![cx.flex(
                FlexProps {
                    layout: LayoutStyle::default(),
                    direction: fret_core::Axis::Vertical,
                    gap: root_gap,
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::Start,
                    align: fret_ui::element::CrossAlign::Stretch,
                    wrap: false,
                    ..Default::default()
                },
                move |_cx| {
                    let mut out = Vec::new();
                    out.push(list);
                    out
                },
            )]
        })
    }
}

pub fn navigation_menu<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<NavigationMenuItem>,
) -> AnyElement {
    NavigationMenu::new(model).items(f(cx)).into_element(cx)
}

pub fn navigation_menu_list<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<NavigationMenuItem>,
) -> NavigationMenuList {
    NavigationMenuList::new(f(cx))
}

pub fn navigation_menu_uncontrolled<H: UiHost, T: Into<Arc<str>>>(
    cx: &mut ElementContext<'_, H>,
    default_value: Option<T>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<NavigationMenuItem>,
) -> AnyElement {
    NavigationMenu::uncontrolled(default_value)
        .items(f(cx))
        .into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, KeyCode, Modifiers, MouseButton, MouseButtons, Point, PointerEvent,
        PointerType, Px, Rect, Size,
    };
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{SvgId, SvgService};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_runtime::{FrameId, TickId};
    use fret_ui::tree::UiTree;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: &TextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    fn bump_frame(app: &mut App) {
        app.set_tick_id(TickId(app.tick_id().0.saturating_add(1)));
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    #[test]
    fn hovering_trigger_opens_after_delay_like_radix() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        bump_frame(&mut app);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "navigation-menu",
            |cx| {
                let items = vec![
                    NavigationMenuItem::new("alpha", "Alpha", vec![cx.text("A")]),
                    NavigationMenuItem::new("beta", "Beta", vec![cx.text("B")]),
                ];
                vec![
                    NavigationMenu::new(model.clone())
                        .items(items)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha_btn = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Alpha"))
            .expect("alpha button semantics");
        let pos = Point::new(
            Px(alpha_btn.bounds.origin.x.0 + alpha_btn.bounds.size.width.0 * 0.5),
            Px(alpha_btn.bounds.origin.y.0 + alpha_btn.bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Move {
                position: pos,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );

        let effects = app.flush_effects();
        let token = effects
            .iter()
            .find_map(|e| match e {
                fret_runtime::Effect::SetTimer { token, after, .. }
                    if *after
                        == radix_navigation_menu::NavigationMenuConfig::default()
                            .delay_duration =>
                {
                    Some(*token)
                }
                _ => None,
            })
            .expect("expected delayed-open timer");

        ui.dispatch_event(&mut app, &mut services, &fret_core::Event::Timer { token });

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));
    }

    #[test]
    fn viewport_disabled_still_opens_after_delay_like_radix() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        bump_frame(&mut app);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "navigation-menu-no-viewport",
            |cx| {
                let items = vec![
                    NavigationMenuItem::new("alpha", "Alpha", vec![cx.text("A")]),
                    NavigationMenuItem::new("beta", "Beta", vec![cx.text("B")]),
                ];
                vec![
                    NavigationMenu::new(model.clone())
                        .viewport(false)
                        .items(items)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha_btn = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Alpha"))
            .expect("alpha button semantics");
        let pos = Point::new(
            Px(alpha_btn.bounds.origin.x.0 + alpha_btn.bounds.size.width.0 * 0.5),
            Px(alpha_btn.bounds.origin.y.0 + alpha_btn.bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Move {
                position: pos,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );

        let effects = app.flush_effects();
        let token = effects
            .iter()
            .find_map(|e| match e {
                fret_runtime::Effect::SetTimer { token, after, .. }
                    if *after
                        == radix_navigation_menu::NavigationMenuConfig::default()
                            .delay_duration =>
                {
                    Some(*token)
                }
                _ => None,
            })
            .expect("expected delayed-open timer");

        ui.dispatch_event(&mut app, &mut services, &fret_core::Event::Timer { token });

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));
    }

    #[test]
    fn escape_close_sets_trigger_gate_and_does_not_reopen_on_pointer_move() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(None::<Arc<str>>);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        bump_frame(&mut app);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "navigation-menu-escape",
            |cx| {
                let items = vec![
                    NavigationMenuItem::new("alpha", "Alpha", vec![cx.text("A")]),
                    NavigationMenuItem::new("beta", "Beta", vec![cx.text("B")]),
                ];
                vec![
                    NavigationMenu::new(model.clone())
                        .items(items)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let alpha_btn = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Alpha"))
            .expect("alpha button semantics");
        let pos = Point::new(
            Px(alpha_btn.bounds.origin.x.0 + alpha_btn.bounds.size.width.0 * 0.5),
            Px(alpha_btn.bounds.origin.y.0 + alpha_btn.bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Down {
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Up {
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: KeyCode::Escape,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected, None);

        app.flush_effects();

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Move {
                position: pos,
                buttons: MouseButtons::default(),
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
            }),
        );

        let effects = app.flush_effects();
        let has_open_timer = effects.iter().any(|e| matches!(e,
            fret_runtime::Effect::SetTimer { after, .. }
                if *after == radix_navigation_menu::NavigationMenuConfig::default().delay_duration
        ));
        assert!(
            !has_open_timer,
            "expected no delayed-open timer after escape gating"
        );
    }

    #[test]
    fn navigation_menu_link_does_not_dismiss_on_ctrl_click() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        bump_frame(&mut app);
        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "navigation-menu-link",
            |cx| {
                vec![
                    NavigationMenuLink::new(model.clone(), vec![cx.text("Go")])
                        .label("Go")
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let go_btn = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some("Go"))
            .expect("Go button semantics");
        let pos = Point::new(
            Px(go_btn.bounds.origin.x.0 + go_btn.bounds.size.width.0 * 0.5),
            Px(go_btn.bounds.origin.y.0 + go_btn.bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Down {
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Up {
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers {
                    ctrl: true,
                    ..Default::default()
                },
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Down {
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(PointerEvent::Up {
                position: pos,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                click_count: 1,
                pointer_type: PointerType::Mouse,
            }),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected, None);
    }
}
