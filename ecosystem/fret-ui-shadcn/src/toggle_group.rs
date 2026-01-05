use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, MainAlign, PressableA11y, PressableProps, RovingFlexProps,
    RovingFocusProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::headless::roving_focus;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Space};

use crate::toggle::{ToggleSize, ToggleVariant};

fn toggle_bg_hover(theme: &Theme) -> Color {
    theme
        .color_by_key("muted")
        .unwrap_or(theme.colors.hover_background)
}

fn toggle_bg_on(theme: &Theme) -> Color {
    theme.color_by_key("accent").unwrap_or(theme.colors.accent)
}

fn toggle_border(theme: &Theme) -> Color {
    theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or(theme.colors.panel_border)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleGroupKind {
    Single,
    Multiple,
}

/// Matches Radix ToggleGroup `orientation` outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToggleGroupOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone)]
enum ToggleGroupModel {
    Single(Model<Option<Arc<str>>>),
    Multiple(Model<Vec<Arc<str>>>),
}

#[derive(Clone)]
pub struct ToggleGroupItem {
    value: Arc<str>,
    children: Vec<AnyElement>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
}

impl std::fmt::Debug for ToggleGroupItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToggleGroupItem")
            .field("value", &self.value.as_ref())
            .field("children_len", &self.children.len())
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .finish()
    }
}

impl ToggleGroupItem {
    pub fn new(value: impl Into<Arc<str>>, children: Vec<AnyElement>) -> Self {
        Self {
            value: value.into(),
            children,
            disabled: false,
            a11y_label: None,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }
}

#[derive(Clone)]
pub struct ToggleGroup {
    model: ToggleGroupModel,
    items: Vec<ToggleGroupItem>,
    disabled: bool,
    orientation: ToggleGroupOrientation,
    loop_navigation: bool,
    variant: ToggleVariant,
    size: ToggleSize,
    spacing: Space,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for ToggleGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match &self.model {
            ToggleGroupModel::Single(_) => ToggleGroupKind::Single,
            ToggleGroupModel::Multiple(_) => ToggleGroupKind::Multiple,
        };
        f.debug_struct("ToggleGroup")
            .field("kind", &kind)
            .field("items_len", &self.items.len())
            .field("disabled", &self.disabled)
            .field("orientation", &self.orientation)
            .field("loop_navigation", &self.loop_navigation)
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("spacing", &self.spacing)
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .finish()
    }
}

impl ToggleGroup {
    pub fn single(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model: ToggleGroupModel::Single(model),
            items: Vec::new(),
            disabled: false,
            orientation: ToggleGroupOrientation::default(),
            loop_navigation: true,
            variant: ToggleVariant::default(),
            size: ToggleSize::default(),
            spacing: Space::N0,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn multiple(model: Model<Vec<Arc<str>>>) -> Self {
        Self {
            model: ToggleGroupModel::Multiple(model),
            items: Vec::new(),
            disabled: false,
            orientation: ToggleGroupOrientation::default(),
            loop_navigation: true,
            variant: ToggleVariant::default(),
            size: ToggleSize::default(),
            spacing: Space::N0,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn orientation(mut self, orientation: ToggleGroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// When `true` (default), roving navigation loops at the ends (Radix `loop` behavior).
    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self
    }

    pub fn spacing(mut self, spacing: Space) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn item(mut self, item: ToggleGroupItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ToggleGroupItem>) -> Self {
        self.items.extend(items);
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
        let group_disabled = self.disabled;
        let orientation = self.orientation;
        let loop_navigation = self.loop_navigation;
        let variant = self.variant;
        let size = self.size.component_size();
        let spacing = self.spacing;
        let chrome = self.chrome;
        let layout = self.layout;

        let theme = Theme::global(&*cx.app).clone();

        let (selected_single, selected_multi) = match &model {
            ToggleGroupModel::Single(m) => (cx.watch_model(m).layout().cloned().flatten(), None),
            ToggleGroupModel::Multiple(m) => (None, cx.watch_model(m).layout().cloned()),
        };

        let values: Vec<Arc<str>> = items.iter().map(|i| i.value.clone()).collect();
        let disabled_flags: Vec<bool> =
            items.iter().map(|i| group_disabled || i.disabled).collect();

        let tab_stop = match (selected_single.as_deref(), selected_multi.as_ref()) {
            (Some(selected), _) => {
                roving_focus::active_index_from_str_keys(&values, Some(selected), &disabled_flags)
            }
            (_, Some(selected)) => {
                let first_selected_enabled = values.iter().enumerate().find_map(|(idx, v)| {
                    let enabled = !disabled_flags.get(idx).copied().unwrap_or(true);
                    let on = selected.iter().any(|s| s.as_ref() == v.as_ref());
                    (enabled && on).then_some(idx)
                });
                first_selected_enabled.or_else(|| roving_focus::first_enabled(&disabled_flags))
            }
            _ => roving_focus::first_enabled(&disabled_flags),
        };

        let gap = MetricRef::space(spacing).resolve(&theme);
        let radius = size.control_radius(&theme);
        let ring = decl_style::focus_ring(&theme, radius);
        let pad_x = size.button_px(&theme);
        let pad_y = size.button_py(&theme);

        let bg_hover = toggle_bg_hover(&theme);
        let bg_on = toggle_bg_on(&theme);
        let border = toggle_border(&theme);

        let group_props = decl_style::container_props(&theme, chrome, layout);

        let base_chrome = match variant {
            ToggleVariant::Default => ChromeRefinement {
                radius: Some(MetricRef::Px(radius)),
                ..Default::default()
            },
            ToggleVariant::Outline => ChromeRefinement {
                radius: Some(MetricRef::Px(radius)),
                border_width: Some(MetricRef::Px(Px(1.0))),
                border_color: Some(ColorRef::Color(border)),
                ..Default::default()
            },
        };

        let (model_single, model_multi) = match &model {
            ToggleGroupModel::Single(m) => (Some(m.clone()), None),
            ToggleGroupModel::Multiple(m) => (None, Some(m.clone())),
        };

        let roving = RovingFocusProps {
            enabled: !group_disabled,
            wrap: loop_navigation,
            disabled: Arc::from(disabled_flags.clone().into_boxed_slice()),
            ..Default::default()
        };

        cx.container(group_props, move |cx| {
            vec![cx.roving_flex(
                RovingFlexProps {
                    flex: FlexProps {
                        direction: match orientation {
                            ToggleGroupOrientation::Horizontal => fret_core::Axis::Horizontal,
                            ToggleGroupOrientation::Vertical => fret_core::Axis::Vertical,
                        },
                        gap,
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        wrap: false,
                        ..Default::default()
                    },
                    roving,
                },
                move |cx| {
                    cx.roving_nav_apg();
                    let n = items.len();
                    let mut out = Vec::with_capacity(n);

                    for (idx, item) in items.into_iter().enumerate() {
                        let item_disabled = disabled_flags.get(idx).copied().unwrap_or(true);
                        let enabled = !item_disabled;
                        let focusable = tab_stop.is_some_and(|i| i == idx);
                        let on = selected_single
                            .as_deref()
                            .is_some_and(|v| v == item.value.as_ref())
                            || selected_multi.as_ref().is_some_and(|selected| {
                                selected.iter().any(|v| v.as_ref() == item.value.as_ref())
                            });

                        let corners = if gap.0 <= 0.0 {
                            let first = idx == 0;
                            let last = idx + 1 == n;
                            match orientation {
                                ToggleGroupOrientation::Horizontal => Corners {
                                    top_left: if first { radius } else { Px(0.0) },
                                    bottom_left: if first { radius } else { Px(0.0) },
                                    top_right: if last { radius } else { Px(0.0) },
                                    bottom_right: if last { radius } else { Px(0.0) },
                                },
                                ToggleGroupOrientation::Vertical => Corners {
                                    top_left: if first { radius } else { Px(0.0) },
                                    top_right: if first { radius } else { Px(0.0) },
                                    bottom_left: if last { radius } else { Px(0.0) },
                                    bottom_right: if last { radius } else { Px(0.0) },
                                },
                            }
                        } else {
                            Corners::all(radius)
                        };

                        let mut base_props = decl_style::container_props(
                            &theme,
                            base_chrome.clone(),
                            LayoutRefinement::default(),
                        );
                        base_props.padding = Edges {
                            top: pad_y,
                            right: pad_x,
                            bottom: pad_y,
                            left: pad_x,
                        };
                        base_props.corner_radii = corners;

                        if gap.0 <= 0.0
                            && variant == ToggleVariant::Outline
                            && idx > 0
                            && (base_props.border.left.0 > 0.0 || base_props.border.top.0 > 0.0)
                        {
                            match orientation {
                                ToggleGroupOrientation::Horizontal => {
                                    base_props.border.left = Px(0.0);
                                }
                                ToggleGroupOrientation::Vertical => {
                                    base_props.border.top = Px(0.0);
                                }
                            }
                        }

                        let value = item.value.clone();
                        let a11y_label = item.a11y_label.clone().unwrap_or_else(|| value.clone());
                        let children = item.children;
                        let model_single = model_single.clone();
                        let model_multi = model_multi.clone();

                        out.push(
                            cx.pressable(
                                PressableProps {
                                    layout: decl_style::layout_style(
                                        &theme,
                                        LayoutRefinement::default()
                                            .min_h(MetricRef::Px(size.button_h(&theme)))
                                            .min_w_0()
                                            .flex_none(),
                                    ),
                                    enabled,
                                    focusable,
                                    focus_ring: Some(ring),
                                    a11y: PressableA11y {
                                        role: Some(SemanticsRole::Button),
                                        label: Some(a11y_label),
                                        selected: on,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                move |cx, state| {
                                    if let Some(m) = model_single.as_ref() {
                                        let model = m.clone();
                                        let value = value.clone();
                                        cx.pressable_add_on_activate(Arc::new(
                                            move |host, _action_cx, _reason| {
                                                let current =
                                                    host.models_mut().get_cloned(&model).flatten();
                                                let next = if current
                                                    .as_ref()
                                                    .is_some_and(|cur| cur.as_ref() == value.as_ref())
                                                {
                                                    None
                                                } else {
                                                    Some(value.clone())
                                                };
                                                let _ =
                                                    host.models_mut().update(&model, |v| *v = next);
                                            },
                                        ));
                                    }
                                    if let Some(m) = model_multi.as_ref() {
                                        cx.pressable_toggle_vec_arc_str(m, value.clone());
                                    }

                                    let hovered = state.hovered && !state.pressed;
                                    let pressed = state.pressed;

                                    let bg = if on && !item_disabled {
                                        Some(bg_on)
                                    } else if (hovered || pressed) && !item_disabled {
                                        Some(bg_hover)
                                    } else {
                                        None
                                    };

                                    let mut props = base_props;
                                    if bg.is_some() {
                                        props.background = bg;
                                    }

                                    vec![cx.container(props, move |_cx| children)]
                                },
                            ),
                        );
                    }

                    out
                },
            )]
        })
    }
}

pub fn toggle_group_single<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<ToggleGroupItem>,
) -> AnyElement {
    ToggleGroup::single(model).items(f(cx)).into_element(cx)
}

pub fn toggle_group_multiple<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Vec<Arc<str>>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<ToggleGroupItem>,
) -> AnyElement {
    ToggleGroup::multiple(model).items(f(cx)).into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{AppWindowId, Modifiers, Point, Px, Rect, SemanticsRole, Size, SvgId, SvgService};
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle};
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

    fn render_single(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
    ) -> fret_core::NodeId {
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "toggle-group-single",
            |cx| {
                let items = vec![
                    ToggleGroupItem::new("alpha", vec![]),
                    ToggleGroupItem::new("beta", vec![]),
                    ToggleGroupItem::new("gamma", vec![]),
                ];
                vec![ToggleGroup::single(model).items(items).into_element(cx)]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn toggle_group_single_deactivates_when_activating_selected_item() {
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

        let root = render_single(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
        );

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable item");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyUp {
                key: fret_core::KeyCode::Space,
                modifiers: Modifiers::default(),
            },
        );

        let _ = render_single(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected, None);
    }

    #[test]
    fn toggle_group_single_arrow_moves_focus_without_changing_value() {
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

        let root = render_single(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
        );

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable item");
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

        let _ = render_single(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
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
        assert_eq!(focused_node.role, SemanticsRole::Button);
        assert_eq!(focused_node.label.as_deref(), Some("beta"));
    }
}
