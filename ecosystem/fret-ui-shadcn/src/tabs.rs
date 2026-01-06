use std::cell::Cell;
use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableProps, RovingFlexProps, RovingFocusProps, SemanticsProps, SpinnerProps, SvgIconProps,
    TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space};

fn apply_trigger_inherited_style(
    mut element: AnyElement,
    fg: Color,
    text_style: &TextStyle,
) -> AnyElement {
    match &mut element.kind {
        fret_ui::element::ElementKind::Text(props) => {
            if props.style.is_none() {
                props.style = Some(text_style.clone());
            }
            if props.color.is_none() {
                props.color = Some(fg);
            }
        }
        fret_ui::element::ElementKind::SvgIcon(SvgIconProps { color, .. }) => {
            let is_default = *color
                == Color {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                };
            if is_default {
                *color = fg;
            }
        }
        fret_ui::element::ElementKind::Spinner(SpinnerProps { color, .. }) => {
            color.get_or_insert(fg);
        }
        _ => {}
    }

    element.children = element
        .children
        .into_iter()
        .map(|child| apply_trigger_inherited_style(child, fg, text_style))
        .collect();
    element
}

fn tabs_gap(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.tabs.gap")
        .unwrap_or_else(|| MetricRef::space(Space::N2).resolve(theme))
}

fn tabs_list_height(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.tabs.list_height")
        .unwrap_or(Px(36.0))
}

fn tabs_list_padding(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.tabs.list_padding")
        .unwrap_or(Px(3.0))
}

fn tabs_list_bg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted")
        .unwrap_or(theme.colors.panel_background)
}

fn tabs_list_fg_muted(theme: &Theme) -> Color {
    theme
        .color_by_key("muted.foreground")
        .or_else(|| theme.color_by_key("muted-foreground"))
        .unwrap_or(theme.colors.text_muted)
}

fn tabs_trigger_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.tabs.trigger.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or(theme.metrics.font_size);
    let line_height = theme
        .metric_by_key("component.tabs.trigger.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or(theme.metrics.font_line_height);
    TextStyle {
        font: FontId::default(),
        size: px,
        weight: FontWeight::MEDIUM,
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

fn tabs_trigger_radius(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.tabs.trigger.radius")
        .unwrap_or_else(|| MetricRef::radius(Radius::Md).resolve(theme))
}

fn tabs_trigger_bg_active(theme: &Theme) -> Color {
    theme
        .color_by_key("background")
        .unwrap_or(theme.colors.surface_background)
}

fn tabs_trigger_border_active(theme: &Theme) -> Color {
    theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or(theme.colors.panel_border)
}

fn tabs_trigger_border_width(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.tabs.trigger.border_width")
        .unwrap_or(Px(1.0))
}

pub use fret_ui_kit::primitives::tabs::{TabsActivationMode, TabsOrientation};

#[derive(Debug, Clone)]
pub struct TabsItem {
    value: Arc<str>,
    label: Arc<str>,
    content: Vec<AnyElement>,
    trigger: Option<Vec<AnyElement>>,
    disabled: bool,
}

impl TabsItem {
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
pub struct Tabs {
    model: Model<Option<Arc<str>>>,
    items: Vec<TabsItem>,
    disabled: bool,
    orientation: TabsOrientation,
    activation_mode: TabsActivationMode,
    loop_navigation: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    force_mount_content: bool,
}

impl std::fmt::Debug for Tabs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tabs")
            .field("model", &"<model>")
            .field("items_len", &self.items.len())
            .field("disabled", &self.disabled)
            .field("orientation", &self.orientation)
            .field("activation_mode", &self.activation_mode)
            .field("loop_navigation", &self.loop_navigation)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .field("force_mount_content", &self.force_mount_content)
            .finish()
    }
}

impl Tabs {
    pub fn new(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model,
            items: Vec::new(),
            disabled: false,
            orientation: TabsOrientation::default(),
            activation_mode: TabsActivationMode::default(),
            loop_navigation: true,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            force_mount_content: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn orientation(mut self, orientation: TabsOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn activation_mode(mut self, activation_mode: TabsActivationMode) -> Self {
        self.activation_mode = activation_mode;
        self
    }

    /// When `true` (default), arrow key navigation loops at the ends (Radix `loop` behavior).
    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    pub fn item(mut self, item: TabsItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = TabsItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// When `true`, all tab panel subtrees remain mounted even when inactive.
    ///
    /// This approximates Radix `TabsContent forceMount` by keeping each panel subtree in the
    /// declarative element tree while gating layout/paint/semantics and interactivity via
    /// `InteractivityGate`.
    pub fn force_mount_content(mut self, force_mount_content: bool) -> Self {
        self.force_mount_content = force_mount_content;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let model = self.model;
        let items = self.items;
        let tabs_disabled = self.disabled;
        let orientation = self.orientation;
        let activation_mode = self.activation_mode;
        let loop_navigation = self.loop_navigation;
        let chrome = self.chrome;
        let layout = self.layout;
        let force_mount_content = self.force_mount_content;

        let theme = Theme::global(&*cx.app).clone();
        let gap = tabs_gap(&theme);
        let text_style = tabs_trigger_text_style(&theme);

        let selected: Option<Arc<str>> = cx.watch_model(&model).layout().cloned().flatten();

        let values: Vec<Arc<str>> = items.iter().map(|i| i.value.clone()).collect();
        let disabled_flags: Vec<bool> = items.iter().map(|i| tabs_disabled || i.disabled).collect();
        let active_idx = fret_ui_kit::primitives::tabs::active_index_from_values(
            &values,
            selected.as_deref(),
            &disabled_flags,
        );

        let values_arc: Arc<[Arc<str>]> = Arc::from(values.into_boxed_slice());
        let roving = RovingFocusProps {
            enabled: !tabs_disabled,
            wrap: loop_navigation,
            disabled: Arc::from(disabled_flags.clone().into_boxed_slice()),
            ..Default::default()
        };
        let tab_set_size = u32::try_from(items.len())
            .ok()
            .and_then(|size| (size > 0).then_some(size));

        let list_height = tabs_list_height(&theme);
        let list_padding = tabs_list_padding(&theme);
        let mut list_props = decl_style::container_props(
            &theme,
            ChromeRefinement::default()
                .rounded(Radius::Lg)
                .bg(ColorRef::Color(tabs_list_bg(&theme))),
            LayoutRefinement::default().h_px(MetricRef::Px(list_height)),
        );
        list_props.padding = Edges::all(list_padding);

        let active_label = active_idx
            .and_then(|active| items.get(active))
            .map(|item| item.label.clone())
            .unwrap_or_else(|| Arc::from(""));
        let active_children = active_idx
            .and_then(|active| items.get(active))
            .and_then(|item| (!force_mount_content).then_some(item.content.clone()))
            .unwrap_or_default();

        let root_props = decl_style::container_props(&theme, chrome, layout);

        cx.container(root_props, move |cx| {
            let selected_tab_element: Cell<Option<u64>> = Cell::new(None);
            let selected_tab_element = &selected_tab_element;
            let tab_trigger_elements: Vec<Cell<Option<u64>>> =
                (0..items.len()).map(|_| Cell::new(None)).collect();
            let tab_trigger_elements = &tab_trigger_elements;
            let items_for_list = items.clone();
            let mut children: Vec<AnyElement> = Vec::new();

            children.push(cx.semantics(
                SemanticsProps {
                    role: SemanticsRole::TabList,
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.container(list_props, |cx| {
                        vec![cx.roving_flex(
                            RovingFlexProps {
                                flex: FlexProps {
                                    direction: match orientation {
                                        TabsOrientation::Horizontal => fret_core::Axis::Horizontal,
                                        TabsOrientation::Vertical => fret_core::Axis::Vertical,
                                    },
                                    gap: Px(0.0),
                                    padding: Edges::all(Px(0.0)),
                                    justify: MainAlign::Center,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                    ..Default::default()
                                },
                                roving,
                            },
                            |cx| {
                                cx.roving_nav_apg();
                                if activation_mode == TabsActivationMode::Automatic {
                                    cx.roving_select_option_arc_str(&model, values_arc.clone());
                                }

                                let fg_muted = tabs_list_fg_muted(&theme);
                                let fg_disabled = theme.colors.text_disabled;
                                let fg_active = theme
                                    .color_by_key("foreground")
                                    .unwrap_or(theme.colors.text_primary);
                                let radius = tabs_trigger_radius(&theme);
                                let ring = decl_style::focus_ring(&theme, radius);
                                let bg_active = tabs_trigger_bg_active(&theme);
                                let border_active = tabs_trigger_border_active(&theme);
                                let border_w = tabs_trigger_border_width(&theme);

                                let pad_x = MetricRef::space(Space::N2).resolve(&theme);
                                let pad_y = MetricRef::space(Space::N1).resolve(&theme);
                                // new-york-v4: trigger uses `h-[calc(100%-1px)]` relative to the list
                                // content box (after list padding).
                                let trigger_h = Px(
                                    (list_height.0 - list_padding.0 * 2.0 - 1.0).max(0.0),
                                );
                                let trigger_layout = decl_style::layout_style(
                                    &theme,
                                    LayoutRefinement::default()
                                        .flex_1()
                                        .h_px(MetricRef::Px(trigger_h)),
                                );

                                let mut out: Vec<AnyElement> =
                                    Vec::with_capacity(disabled_flags.len());
                                for (idx, item) in items_for_list.iter().cloned().enumerate() {
                                    let item_disabled =
                                        disabled_flags.get(idx).copied().unwrap_or(true);
                                    let tab_stop = active_idx.is_some_and(|a| a == idx);
                                    let active = tab_stop;

                                    let fg = if item_disabled {
                                        fg_disabled
                                    } else if active {
                                        fg_active
                                    } else {
                                        fg_muted
                                    };
                                    let bg = (active && !item_disabled).then_some(bg_active);
                                    let border = (active && !item_disabled)
                                        .then_some(border_active)
                                        .unwrap_or(Color::TRANSPARENT);
                                    let shadow = (active && !item_disabled)
                                        .then(|| decl_style::shadow_sm(&theme, radius));

                                    let value = item.value.clone();
                                    let label = item.label.clone();
                                    let trigger_children = item.trigger.clone();
                                    let model = model.clone();
                                    let text_style = text_style.clone();

                                    out.push(cx.pressable_with_id_props(move |cx, st, _id| {
                                        cx.pressable_set_option_arc_str(&model, value.clone());
                                        if active {
                                            selected_tab_element.set(Some(_id.0));
                                        }
                                        if force_mount_content
                                            && let Some(cell) = tab_trigger_elements.get(idx)
                                        {
                                            cell.set(Some(_id.0));
                                        }

                                        let props = PressableProps {
                                            layout: trigger_layout,
                                            enabled: !item_disabled,
                                            focusable: tab_stop || st.focused,
                                            focus_ring: Some(ring),
                                            a11y: fret_ui_kit::primitives::tabs::tab_a11y_with_collection(
                                                Some(label.clone()),
                                                active,
                                                u32::try_from(idx + 1).ok(),
                                                tab_set_size,
                                            ),
                                            ..Default::default()
                                        };

                                        let children = vec![cx.container(
                                            ContainerProps {
                                                layout: {
                                                    let mut layout = LayoutStyle::default();
                                                    layout.size.width = Length::Fill;
                                                    layout.size.height = Length::Fill;
                                                    layout
                                                },
                                                padding: Edges {
                                                    top: pad_y,
                                                    right: pad_x,
                                                    bottom: pad_y,
                                                    left: pad_x,
                                                },
                                                background: bg,
                                                shadow,
                                                border: Edges::all(border_w),
                                                border_color: Some(border),
                                                corner_radii: Corners::all(radius),
                                                ..Default::default()
                                            },
                                            move |cx| {
                                                let base =
                                                    trigger_children.clone().unwrap_or_else(|| {
                                                        vec![cx.text_props(TextProps {
                                                            layout: Default::default(),
                                                            text: label.clone(),
                                                            style: Some(text_style.clone()),
                                                            color: Some(fg),
                                                            wrap: TextWrap::None,
                                                            overflow: TextOverflow::Clip,
                                                        })]
                                                    });

                                                let styled: Vec<AnyElement> = base
                                                    .into_iter()
                                                    .map(|child| {
                                                        apply_trigger_inherited_style(
                                                            child,
                                                            fg,
                                                            &text_style,
                                                        )
                                                    })
                                                    .collect();

                                                vec![cx.flex(
                                                    FlexProps {
                                                        layout: LayoutStyle::default(),
                                                        direction: fret_core::Axis::Horizontal,
                                                        gap: Px(6.0),
                                                        padding: Edges::all(Px(0.0)),
                                                        justify: MainAlign::Center,
                                                        align: CrossAlign::Center,
                                                        wrap: false,
                                                    },
                                                    move |_cx| styled,
                                                )]
                                            },
                                        )];

                                        (props, children)
                                    }));
                                }
                                out
                            },
                        )]
                    })]
                },
            ));

            if !force_mount_content {
                children.push(cx.semantics(
                    SemanticsProps {
                        role: SemanticsRole::TabPanel,
                        label: (!active_label.is_empty()).then_some(active_label),
                        labelled_by_element: selected_tab_element.get(),
                        ..Default::default()
                    },
                    move |_cx| active_children,
                ));
            }

            if force_mount_content {
                for (idx, item) in items.iter().cloned().enumerate() {
                    let active = active_idx.is_some_and(|a| a == idx);
                    let labelled_by_element = tab_trigger_elements
                        .get(idx)
                        .and_then(|cell| cell.get());
                    let label = item.label.clone();
                    let content = item.content.clone();

                    children.push(cx.interactivity_gate(active, active, move |cx| {
                        vec![cx.semantics(
                            SemanticsProps {
                                role: SemanticsRole::TabPanel,
                                label: (!label.is_empty()).then_some(label),
                                labelled_by_element,
                                ..Default::default()
                            },
                            move |_cx| content,
                        )]
                    }));
                }
            }

            vec![cx.flex(
                FlexProps {
                    direction: match orientation {
                        TabsOrientation::Horizontal => fret_core::Axis::Vertical,
                        TabsOrientation::Vertical => fret_core::Axis::Horizontal,
                    },
                    gap,
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                    wrap: false,
                    ..Default::default()
                },
                move |_cx| children,
            )]
        })
    }
}

pub fn tabs<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<TabsItem>,
) -> AnyElement {
    Tabs::new(model).items(f(cx)).into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, Point, Px, Rect, SemanticsRole, Size, SvgId, SvgService,
    };
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle};
    use fret_runtime::{FrameId, TickId};
    use fret_ui::elements::{ElementRuntime, GlobalElementId, node_for_element};
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

    fn render(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        activation_mode: TabsActivationMode,
    ) -> fret_core::NodeId {
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "tabs", |cx| {
                let items = vec![
                    TabsItem::new("alpha", "Alpha", vec![]),
                    TabsItem::new("beta", "Beta", vec![]),
                    TabsItem::new("gamma", "Gamma", vec![]),
                ];
                vec![
                    Tabs::new(model)
                        .activation_mode(activation_mode)
                        .items(items)
                        .into_element(cx),
                ]
            });
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn bump_frame(app: &mut App) {
        app.set_tick_id(TickId(app.tick_id().0.saturating_add(1)));
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));
    }

    fn render_force_mount_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        force_mount: bool,
        alpha_content_id_out: &Cell<Option<GlobalElementId>>,
    ) {
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "tabs-force-mount",
            |cx| {
                let alpha_content = cx.pressable_with_id(
                    PressableProps {
                        layout: LayoutStyle::default(),
                        enabled: true,
                        focusable: true,
                        ..Default::default()
                    },
                    |_cx, _st, id| {
                        alpha_content_id_out.set(Some(id));
                        Vec::new()
                    },
                );

                let items = vec![
                    TabsItem::new("alpha", "Alpha", vec![alpha_content]),
                    TabsItem::new("beta", "Beta", vec![]),
                ];

                vec![
                    Tabs::new(model)
                        .force_mount_content(force_mount)
                        .items(items)
                        .into_element(cx),
                ]
            },
        );

        ui.set_root(root);
        ui.layout_all(app, services, bounds, 1.0);
    }

    #[test]
    fn tabs_manual_activation_does_not_change_model_on_arrow_navigation() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            TabsActivationMode::Manual,
        );

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable tab");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            TabsActivationMode::Manual,
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let focus = snap.focus.expect("focus");
        let focused_node = snap
            .nodes
            .iter()
            .find(|n| n.id == focus)
            .expect("focused node");
        assert_eq!(focused_node.role, SemanticsRole::Tab);
        assert_eq!(focused_node.label.as_deref(), Some("Beta"));

        assert!(
            snap.nodes.iter().any(|n| n.role == SemanticsRole::TabList),
            "tab list role"
        );

        let selected_tab = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::Tab && n.flags.selected)
            .expect("selected tab");
        assert_eq!(selected_tab.set_size, Some(3));
        assert_eq!(selected_tab.pos_in_set, Some(1));

        let mut other_tabs: Vec<_> = snap
            .nodes
            .iter()
            .filter(|n| n.role == SemanticsRole::Tab && !n.flags.selected)
            .collect();
        other_tabs.sort_by_key(|n| n.pos_in_set.unwrap_or(u32::MAX));
        assert_eq!(other_tabs.len(), 2);
        assert_eq!(other_tabs[0].set_size, Some(3));
        assert_eq!(other_tabs[1].set_size, Some(3));
        assert_eq!(other_tabs[0].pos_in_set, Some(2));
        assert_eq!(other_tabs[1].pos_in_set, Some(3));

        let panel = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::TabPanel)
            .expect("tab panel");
        assert_eq!(panel.label.as_deref(), Some("Alpha"));
        assert!(
            panel.labelled_by.iter().any(|id| *id == selected_tab.id),
            "tabpanel should be labelled by selected tab"
        );
        assert!(
            selected_tab.controls.iter().any(|id| *id == panel.id),
            "selected tab should control the active tabpanel"
        );
    }

    #[test]
    fn tabs_without_force_mount_allows_inactive_panel_nodes_to_be_swept() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let alpha_content_id: Cell<Option<GlobalElementId>> = Cell::new(None);

        bump_frame(&mut app);
        render_force_mount_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            false,
            &alpha_content_id,
        );

        let alpha_content_id = alpha_content_id.get().expect("alpha content id");
        assert!(node_for_element(&mut app, window, alpha_content_id).is_some());

        let lag = app.with_global_mut(ElementRuntime::new, |rt, _app| rt.gc_lag_frames());

        let _ = app
            .models_mut()
            .update(&model, |v| *v = Some(Arc::from("beta")));

        for _ in 0..=lag {
            bump_frame(&mut app);
            render_force_mount_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                model.clone(),
                false,
                &Cell::new(Some(alpha_content_id)),
            );
        }

        assert!(node_for_element(&mut app, window, alpha_content_id).is_none());
    }

    #[test]
    fn tabs_force_mount_keeps_inactive_panel_nodes_alive() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let model = app.models_mut().insert(Some(Arc::from("alpha")));
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let alpha_content_id: Cell<Option<GlobalElementId>> = Cell::new(None);

        bump_frame(&mut app);
        render_force_mount_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            true,
            &alpha_content_id,
        );

        let alpha_content_id = alpha_content_id.get().expect("alpha content id");
        assert!(node_for_element(&mut app, window, alpha_content_id).is_some());

        let lag = app.with_global_mut(ElementRuntime::new, |rt, _app| rt.gc_lag_frames());

        let _ = app
            .models_mut()
            .update(&model, |v| *v = Some(Arc::from("beta")));

        for _ in 0..=(lag + 2) {
            bump_frame(&mut app);
            render_force_mount_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                model.clone(),
                true,
                &Cell::new(Some(alpha_content_id)),
            );
        }

        assert!(node_for_element(&mut app, window, alpha_content_id).is_some());
    }
}
