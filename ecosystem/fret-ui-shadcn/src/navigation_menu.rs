use std::sync::{Arc, Mutex};

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, KeyCode, Px, SemanticsRole, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, FlexProps, LayoutStyle, Length, MainAlign, PointerRegionProps,
    PressableA11y, PressableProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::navigation_menu as radix_navigation_menu;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Radius, Space};

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

    pub fn trigger_child(mut self, child: AnyElement) -> Self {
        self.trigger = Some(vec![child]);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Clone)]
pub struct NavigationMenu {
    model: Option<Model<Option<Arc<str>>>>,
    default_value: Option<Arc<str>>,
    items: Vec<NavigationMenuItem>,
    disabled: bool,
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
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            config: radix_navigation_menu::NavigationMenuConfig::default(),
        }
    }

    pub fn items(mut self, items: Vec<NavigationMenuItem>) -> Self {
        self.items = items;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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

        let selected: Option<Arc<str>> = cx.watch_model(&value_model).layout().cloned().flatten();
        let active_idx = selected.as_deref().and_then(|v| {
            items
                .iter()
                .position(|it| it.value.as_ref() == v)
                .filter(|_| !menu_disabled)
        });

        let root_props = decl_style::container_props(&theme, chrome, layout);

        cx.container(root_props, move |cx| {
            let root_id = cx.root_id();
            let root_state: Arc<Mutex<radix_navigation_menu::NavigationMenuRootState>> = cx
                .with_state_for(
                    root_id,
                    || Arc::new(Mutex::new(radix_navigation_menu::NavigationMenuRootState::default())),
                    |s| s.clone(),
                );

            let value_model_for_timer = value_model.clone();
            let root_state_for_timer = root_state.clone();
            cx.timer_on_timer_for(
                root_id,
                Arc::new(move |host, action_cx, token| {
                    let mut st = root_state_for_timer
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    st.on_timer(host, action_cx, token, &value_model_for_timer, cfg)
                }),
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
            let value_for_list = value_model.clone();
            let value_for_viewport = value_model.clone();
            let root_state_for_list = root_state.clone();
            let selected_for_list = selected.clone();
            let trigger_text_style_for_list = trigger_text_style.clone();

            let list = cx.flex(list_props, move |cx| {
                items_for_children
                    .iter()
                    .map(|item| {
                        let item = item.clone();
                        let item_value = item.value.clone();
                        let label = item.label.clone();
                        let disabled = menu_disabled || item.disabled;
                        let is_open = selected_for_list
                            .as_deref()
                            .is_some_and(|v| v == item_value.as_ref());

                        let value_for_item = value_for_list.clone();
                        let root_state_for_item = root_state_for_list.clone();
                        let trigger_text_style_for_item = trigger_text_style_for_list.clone();

                        cx.keyed(item_value.clone(), |cx| {
                            let trigger_state: Arc<Mutex<radix_navigation_menu::NavigationMenuTriggerState>> =
                                cx.with_state_for(
                                    cx.root_id(),
                                    || {
                                        Arc::new(Mutex::new(
                                            radix_navigation_menu::NavigationMenuTriggerState::default(),
                                        ))
                                    },
                                    |s| s.clone(),
                                );

                            let value_for_trigger = value_for_item.clone();
                            let root_state_for_trigger = root_state_for_item.clone();
                            let root_state_for_hover = root_state_for_trigger.clone();
                            let trigger_text_style = trigger_text_style_for_item.clone();

                            let mut pressable = PressableProps::default();
                            pressable.enabled = !disabled;
                            pressable.focusable = !disabled;
                            pressable.a11y = PressableA11y {
                                role: Some(SemanticsRole::Button),
                                label: Some(label.clone()),
                                expanded: Some(is_open),
                                ..Default::default()
                            };

                            let pointer_props = PointerRegionProps {
                                layout: LayoutStyle::default(),
                                enabled: true,
                            };

                            cx.pointer_region(pointer_props, move |cx| {
                                if !disabled {
                                    let trigger_state_for_pointer_move = trigger_state.clone();
                                    let root_state_for_pointer_move = root_state_for_trigger.clone();
                                    let value_for_pointer_move = value_for_trigger.clone();
                                    let item_value_for_pointer_move = item_value.clone();
                                    cx.pointer_region_on_pointer_move(Arc::new(
                                        move |host, action_cx, mv| {
                                            let mut trigger = trigger_state_for_pointer_move
                                                .lock()
                                                .unwrap_or_else(|e| e.into_inner());
                                            match radix_navigation_menu::navigation_menu_trigger_pointer_move_action(
                                                mv.pointer_type,
                                                disabled,
                                                *trigger,
                                            ) {
                                                radix_navigation_menu::NavigationMenuTriggerPointerMoveAction::Ignore => {
                                                    return false;
                                                }
                                                radix_navigation_menu::NavigationMenuTriggerPointerMoveAction::Open => {
                                                    let mut root = root_state_for_pointer_move
                                                        .lock()
                                                        .unwrap_or_else(|e| e.into_inner());
                                                    root.on_trigger_enter(
                                                        host,
                                                        action_cx,
                                                        &value_for_pointer_move,
                                                        item_value_for_pointer_move.clone(),
                                                        cfg,
                                                    );
                                                    trigger.has_pointer_move_opened = true;
                                                    trigger.was_click_close = false;
                                                    trigger.was_escape_close = false;
                                                    false
                                                }
                                            }
                                        },
                                    ));
                                }

                                vec![cx.pressable(pressable, move |cx, st| {
                                    if !disabled {
                                        let element = cx.root_id();
                                        let root_state_for_escape = root_state_for_trigger.clone();
                                        let value_for_escape = value_for_trigger.clone();
                                        let trigger_state_for_escape = trigger_state.clone();
                                        cx.key_on_key_down_for(
                                            element,
                                            Arc::new(move |host, action_cx, it| {
                                                if it.repeat || it.key != KeyCode::Escape {
                                                    return false;
                                                }

                                                let is_open = host
                                                    .models_mut()
                                                    .read(&value_for_escape, |v| v.is_some())
                                                    .ok()
                                                    .unwrap_or(false);
                                                if !is_open {
                                                    return false;
                                                }

                                                let mut root = root_state_for_escape
                                                    .lock()
                                                    .unwrap_or_else(|e| e.into_inner());
                                                root.on_item_dismiss(host, action_cx, &value_for_escape, cfg);

                                                let mut trigger = trigger_state_for_escape
                                                    .lock()
                                                    .unwrap_or_else(|e| e.into_inner());
                                                trigger.was_escape_close = true;
                                                trigger.was_click_close = false;
                                                trigger.has_pointer_move_opened = false;

                                                true
                                            }),
                                        );
                                    }

                                    let root_state_for_activate = root_state_for_trigger.clone();
                                    let value_for_activate = value_for_trigger.clone();
                                    let trigger_state_for_activate = trigger_state.clone();
                                    let item_value_for_activate = item_value.clone();
                                    if !disabled {
                                        cx.pressable_add_on_activate(Arc::new(
                                            move |host, action_cx, _reason| {
                                                let mut root = root_state_for_activate
                                                    .lock()
                                                    .unwrap_or_else(|e| e.into_inner());
                                                root.on_item_select(
                                                    host,
                                                    action_cx,
                                                    &value_for_activate,
                                                    item_value_for_activate.clone(),
                                                    cfg,
                                                );

                                                let now_open = host
                                                    .models_mut()
                                                    .read(&value_for_activate, |v| v.clone())
                                                    .ok()
                                                    .flatten()
                                                    .is_some_and(|v| {
                                                        v.as_ref() == item_value_for_activate.as_ref()
                                                    });

                                                let mut trigger = trigger_state_for_activate
                                                    .lock()
                                                    .unwrap_or_else(|e| e.into_inner());
                                                trigger.was_click_close = !now_open;
                                                if now_open {
                                                    trigger.was_escape_close = false;
                                                }
                                                trigger.has_pointer_move_opened = false;
                                            },
                                        ));
                                    }

                                    if !disabled {
                                        let trigger_state_for_hover = trigger_state.clone();
                                        cx.pressable_on_hover_change(Arc::new(
                                            move |host, action_cx, hovered| {
                                                if hovered {
                                                    return;
                                                }
                                                let mut trigger = trigger_state_for_hover
                                                    .lock()
                                                    .unwrap_or_else(|e| e.into_inner());
                                                let mut root = root_state_for_hover
                                                    .lock()
                                                    .unwrap_or_else(|e| e.into_inner());
                                                root.on_trigger_leave(
                                                    host,
                                                    action_cx,
                                                    &value_for_trigger,
                                                    cfg,
                                                );
                                                *trigger =
                                                    radix_navigation_menu::NavigationMenuTriggerState::default();
                                            },
                                        ));
                                    }

                                    let hovered = st.hovered && !st.pressed;
                                    let pressed = st.pressed;
                                    let fg = if disabled { trigger_fg_muted } else { trigger_fg };
                                    let bg = (hovered || pressed || is_open).then_some(trigger_bg_hover);

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

                                    let content_children = item.trigger.clone().unwrap_or_else(|| {
                                        vec![cx.text_props(TextProps {
                                            layout: LayoutStyle::default(),
                                            text: item.label.clone(),
                                            style: Some(trigger_text_style.clone()),
                                            color: Some(fg),
                                            wrap: TextWrap::None,
                                            overflow: fret_core::TextOverflow::Clip,
                                        })]
                                    });

                                    vec![cx.container(wrapper, move |_cx| content_children)]
                                })]
                            })
                        })
                    })
                    .collect()
            });

            let viewport = active_idx
                .and_then(|idx| items.get(idx))
                .map(|active| active.content.clone())
                .unwrap_or_default();

            let viewport = if viewport.is_empty() {
                Vec::new()
            } else {
                let layout = {
                    let mut layout = LayoutStyle::default();
                    layout.size.width = Length::Fill;
                    layout
                };
                let root_state_for_viewport = root_state.clone();
                vec![cx.pressable(
                    PressableProps {
                        layout,
                        enabled: true,
                        focusable: false,
                        focus_ring: None,
                        a11y: PressableA11y::default(),
                    },
                    move |cx, _st| {
                        let root_state_for_hover = root_state_for_viewport.clone();
                        let value_for_hover = value_for_viewport.clone();
                        cx.pressable_on_hover_change(Arc::new(move |host, action_cx, hovered| {
                            let mut root = root_state_for_hover
                                .lock()
                                .unwrap_or_else(|e| e.into_inner());
                            if hovered {
                                root.on_content_enter(host);
                            } else {
                                root.on_content_leave(host, action_cx, &value_for_hover, cfg);
                            }
                        }));

                        let viewport_props = ContainerProps {
                            layout: LayoutStyle::default(),
                            padding: Edges::all(viewport_pad),
                            background: Some(viewport_bg),
                            shadow: None,
                            border: Edges::all(Px(1.0)),
                            border_color: Some(viewport_border),
                            corner_radii: Corners::all(viewport_radius),
                        };

                        vec![cx.container(viewport_props, move |_cx| viewport.clone())]
                    },
                )]
            };

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
                    out.extend(viewport);
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
}
