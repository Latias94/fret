use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, MainAlign, PressableProps, RovingFlexProps, RovingFocusProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt;
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, WidgetState,
    WidgetStateProperty, WidgetStates,
};

use crate::layout as shadcn_layout;

use crate::toggle::{ToggleSize, ToggleVariant};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn toggle_bg_hover(theme: &Theme) -> Color {
    theme
        .color_by_key("muted")
        .unwrap_or_else(|| theme.color_required("muted"))
}

fn toggle_bg_on(theme: &Theme) -> Color {
    theme
        .color_by_key("accent")
        .unwrap_or_else(|| theme.color_required("accent"))
}

fn toggle_border(theme: &Theme) -> Color {
    theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("border"))
        .unwrap_or_else(|| theme.color_required("border"))
}

fn toggle_ring_color(theme: &Theme) -> Color {
    theme
        .color_by_key("ring")
        .unwrap_or_else(|| theme.color_required("ring"))
}

fn toggle_group_item_h(theme: &Theme, size: ToggleSize) -> Px {
    let (key, fallback) = match size {
        ToggleSize::Default => ("component.toggle_group.item_h", Px(36.0)),
        ToggleSize::Sm => ("component.toggle_group.item_h_sm", Px(32.0)),
        ToggleSize::Lg => ("component.toggle_group.item_h_lg", Px(40.0)),
    };
    theme.metric_by_key(key).unwrap_or(fallback)
}

fn toggle_group_item_pad_x(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.toggle_group.item_pad_x")
        .unwrap_or(Px(12.0))
}

pub use fret_ui_kit::primitives::toggle_group::{ToggleGroupKind, ToggleGroupOrientation};

#[derive(Clone)]
enum ToggleGroupModel {
    Single {
        model: Option<Model<Option<Arc<str>>>>,
        default_value: Option<Arc<str>>,
    },
    Multiple {
        model: Option<Model<Vec<Arc<str>>>>,
        default_value: Vec<Arc<str>>,
    },
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
    pub fn new(value: impl Into<Arc<str>>, children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            value: value.into(),
            children: children.into_iter().collect(),
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

#[derive(Debug, Clone, Default)]
pub struct ToggleGroupStyle {
    pub item_background: Option<WidgetStateProperty<Option<ColorRef>>>,
    pub item_border_color: Option<WidgetStateProperty<Option<ColorRef>>>,
}

impl ToggleGroupStyle {
    pub fn item_background(
        mut self,
        item_background: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.item_background = Some(item_background);
        self
    }

    pub fn item_border_color(
        mut self,
        item_border_color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.item_border_color = Some(item_border_color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.item_background.is_some() {
            self.item_background = other.item_background;
        }
        if other.item_border_color.is_some() {
            self.item_border_color = other.item_border_color;
        }
        self
    }
}

#[derive(Clone)]
pub struct ToggleGroup {
    model: ToggleGroupModel,
    items: Vec<ToggleGroupItem>,
    disabled: bool,
    roving_focus: bool,
    orientation: ToggleGroupOrientation,
    loop_navigation: bool,
    variant: ToggleVariant,
    size: ToggleSize,
    spacing: Space,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    style: ToggleGroupStyle,
}

impl std::fmt::Debug for ToggleGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match &self.model {
            ToggleGroupModel::Single { .. } => ToggleGroupKind::Single,
            ToggleGroupModel::Multiple { .. } => ToggleGroupKind::Multiple,
        };
        f.debug_struct("ToggleGroup")
            .field("kind", &kind)
            .field("items_len", &self.items.len())
            .field("disabled", &self.disabled)
            .field("roving_focus", &self.roving_focus)
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
            model: ToggleGroupModel::Single {
                model: Some(model),
                default_value: None,
            },
            items: Vec::new(),
            disabled: false,
            roving_focus: true,
            orientation: ToggleGroupOrientation::default(),
            loop_navigation: true,
            variant: ToggleVariant::default(),
            size: ToggleSize::default(),
            spacing: Space::N0,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: ToggleGroupStyle::default(),
        }
    }

    /// Creates an uncontrolled single-select toggle group with an optional initial value (Radix
    /// `defaultValue`).
    pub fn single_uncontrolled<T: Into<Arc<str>>>(default_value: Option<T>) -> Self {
        Self {
            model: ToggleGroupModel::Single {
                model: None,
                default_value: default_value.map(Into::into),
            },
            items: Vec::new(),
            disabled: false,
            roving_focus: true,
            orientation: ToggleGroupOrientation::default(),
            loop_navigation: true,
            variant: ToggleVariant::default(),
            size: ToggleSize::default(),
            spacing: Space::N0,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: ToggleGroupStyle::default(),
        }
    }

    pub fn multiple(model: Model<Vec<Arc<str>>>) -> Self {
        Self {
            model: ToggleGroupModel::Multiple {
                model: Some(model),
                default_value: Vec::new(),
            },
            items: Vec::new(),
            disabled: false,
            roving_focus: true,
            orientation: ToggleGroupOrientation::default(),
            loop_navigation: true,
            variant: ToggleVariant::default(),
            size: ToggleSize::default(),
            spacing: Space::N0,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: ToggleGroupStyle::default(),
        }
    }

    /// Creates an uncontrolled multi-select toggle group with an initial set of values (Radix
    /// `defaultValue`).
    pub fn multiple_uncontrolled(default_value: Vec<Arc<str>>) -> Self {
        Self {
            model: ToggleGroupModel::Multiple {
                model: None,
                default_value,
            },
            items: Vec::new(),
            disabled: false,
            roving_focus: true,
            orientation: ToggleGroupOrientation::default(),
            loop_navigation: true,
            variant: ToggleVariant::default(),
            size: ToggleSize::default(),
            spacing: Space::N0,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: ToggleGroupStyle::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// When `true` (default), installs roving focus behavior (Radix `rovingFocus`).
    pub fn roving_focus(mut self, roving_focus: bool) -> Self {
        self.roving_focus = roving_focus;
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

    pub fn style(mut self, style: ToggleGroupStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let model = self.model;
        let items = self.items;
        let group_disabled = self.disabled;
        let roving_focus = self.roving_focus;
        let orientation = self.orientation;
        let loop_navigation = self.loop_navigation;
        let variant = self.variant;
        let size_token = self.size;
        let spacing = self.spacing;
        let chrome = self.chrome;
        let layout = self.layout;
        let style_override = self.style;

        let theme = Theme::global(&*cx.app).clone();

        let (model_single, model_multi, selected_single, selected_multi) = match &model {
            ToggleGroupModel::Single {
                model: controlled,
                default_value,
            } => {
                let model = fret_ui_kit::primitives::toggle_group::toggle_group_use_single_model(
                    cx,
                    controlled.clone(),
                    || default_value.clone(),
                )
                .model();
                let selected = cx.watch_model(&model).layout().cloned().flatten();
                (Some(model), None, selected, None)
            }
            ToggleGroupModel::Multiple {
                model: controlled,
                default_value,
            } => {
                let model = fret_ui_kit::primitives::toggle_group::toggle_group_use_multiple_model(
                    cx,
                    controlled.clone(),
                    || default_value.clone(),
                )
                .model();
                let selected = cx.watch_model(&model).layout().cloned();
                (None, Some(model), None, selected)
            }
        };

        let values: Vec<Arc<str>> = items.iter().map(|i| i.value.clone()).collect();
        let disabled_flags: Vec<bool> =
            items.iter().map(|i| group_disabled || i.disabled).collect();

        let tab_stop = if roving_focus {
            match (selected_single.as_deref(), selected_multi.as_ref()) {
                (Some(selected), _) => {
                    fret_ui_kit::primitives::toggle_group::tab_stop_index_single(
                        &values,
                        Some(selected),
                        &disabled_flags,
                    )
                }
                (_, Some(selected)) => {
                    fret_ui_kit::primitives::toggle_group::tab_stop_index_multiple(
                        &values,
                        selected,
                        &disabled_flags,
                    )
                }
                _ => fret_ui_kit::primitives::roving_focus_group::first_enabled(&disabled_flags),
            }
        } else {
            None
        };

        let gap = MetricRef::space(spacing).resolve(&theme);
        let radius = MetricRef::radius(Radius::Md).resolve(&theme);
        let mut ring = decl_style::focus_ring(&theme, radius);
        ring.color = alpha_mul(toggle_ring_color(&theme), 0.5);
        let ring_border = toggle_ring_color(&theme);

        let item_h = toggle_group_item_h(&theme, size_token);
        let pad_x = toggle_group_item_pad_x(&theme);
        let pad_y = Px(0.0);

        let bg_hover_muted = toggle_bg_hover(&theme);
        let bg_on = toggle_bg_on(&theme);
        let border = toggle_border(&theme);

        let ToggleGroupStyle {
            item_background,
            item_border_color,
        } = style_override;

        let hover_bg = match variant {
            ToggleVariant::Default => bg_hover_muted,
            ToggleVariant::Outline => bg_on,
        };

        let default_item_background = WidgetStateProperty::new(None)
            .when(WidgetStates::HOVERED, Some(ColorRef::Color(hover_bg)))
            .when(WidgetStates::ACTIVE, Some(ColorRef::Color(hover_bg)))
            .when(WidgetStates::SELECTED, Some(ColorRef::Color(bg_on)))
            .when(WidgetStates::DISABLED, None);

        let default_item_border_color = WidgetStateProperty::new(None)
            .when(
                WidgetStates::FOCUS_VISIBLE,
                Some(ColorRef::Color(ring_border)),
            )
            .when(WidgetStates::DISABLED, None);

        let item_background_prop = item_background.unwrap_or(default_item_background);
        let item_border_color_prop = item_border_color.unwrap_or(default_item_border_color);

        let mut group_props = decl_style::container_props(&theme, chrome, layout);
        group_props.corner_radii = Corners::all(radius);
        if matches!(variant, ToggleVariant::Outline) && gap.0 > 0.0 {
            group_props.shadow = Some(decl_style::shadow_xs(&theme, radius));
        }

        let base_chrome = match variant {
            ToggleVariant::Default => ChromeRefinement {
                radius: Some(MetricRef::Px(radius)),
                border_width: Some(MetricRef::Px(Px(1.0))),
                border_color: Some(ColorRef::Color(Color::TRANSPARENT)),
                ..Default::default()
            },
            ToggleVariant::Outline => ChromeRefinement {
                radius: Some(MetricRef::Px(radius)),
                border_width: Some(MetricRef::Px(Px(1.0))),
                border_color: Some(ColorRef::Color(border)),
                ..Default::default()
            },
        };

        let roving = RovingFocusProps {
            enabled: roving_focus && !group_disabled,
            wrap: loop_navigation,
            disabled: Arc::from(disabled_flags.clone().into_boxed_slice()),
            ..Default::default()
        };

        cx.container(group_props, move |cx| {
            let item_background_prop = item_background_prop.clone();
            let item_border_color_prop = item_border_color_prop.clone();

            let flex = FlexProps {
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
            };

            let render_items = move |cx: &mut ElementContext<'_, H>| {
                if roving_focus {
                    cx.roving_nav_apg();
                }
                let n = items.len();
                let mut out = Vec::with_capacity(n);

                for (idx, item) in items.into_iter().enumerate() {
                    let item_disabled = disabled_flags.get(idx).copied().unwrap_or(true);
                    let enabled = !item_disabled;
                    let focusable = if roving_focus {
                        tab_stop.is_some_and(|i| i == idx)
                    } else {
                        enabled
                    };
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
                    let a11y = if model_single.is_some() {
                        fret_ui_kit::primitives::toggle_group::toggle_group_item_a11y_single(
                            a11y_label.clone(),
                            on,
                        )
                    } else {
                        fret_ui_kit::primitives::toggle_group::toggle_group_item_a11y_multiple(
                            a11y_label.clone(),
                            on,
                        )
                    };
                    let children = item.children;
                    let model_single = model_single.clone();
                    let model_multi = model_multi.clone();
                    let pressable_layout = decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default()
                            .min_h(MetricRef::Px(item_h))
                            .min_w_0()
                            .flex_none(),
                    );

                    let item_theme = theme.clone();
                    let item_background_prop = item_background_prop.clone();
                    let item_border_color_prop = item_border_color_prop.clone();

                    out.push(cx.keyed(value.clone(), move |cx| {
                        cx.pressable(
                            PressableProps {
                                layout: pressable_layout,
                                enabled,
                                focusable,
                                focus_ring: Some(ring),
                                a11y,
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
                                            let _ = host.models_mut().update(&model, |v| *v = next);
                                        },
                                    ));
                                }
                                if let Some(m) = model_multi.as_ref() {
                                    cx.pressable_toggle_vec_arc_str(m, value.clone());
                                }

                                let mut states = WidgetStates::from_pressable(cx, state, enabled);
                                states.set(WidgetState::Selected, on);

                                let mut props = base_props;
                                let bg = item_background_prop.resolve(states).clone();
                                if let Some(bg) = bg {
                                    props.background = Some(bg.resolve(&item_theme));
                                }

                                let border_color = item_border_color_prop.resolve(states).clone();
                                if let Some(border_color) = border_color {
                                    props.border_color = Some(border_color.resolve(&item_theme));
                                }
                                props.layout.size = pressable_layout.size;

                                vec![shadcn_layout::container_hstack_centered(
                                    cx,
                                    props,
                                    Space::N1,
                                    children,
                                )]
                            },
                        )
                    }));
                }

                out
            };

            if roving_focus {
                vec![cx.roving_flex(RovingFlexProps { flex, roving }, render_items)]
            } else {
                vec![cx.flex(flex, render_items)]
            }
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

pub fn toggle_group_single_uncontrolled<H: UiHost, T: Into<Arc<str>>>(
    cx: &mut ElementContext<'_, H>,
    default_value: Option<T>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<ToggleGroupItem>,
) -> AnyElement {
    ToggleGroup::single_uncontrolled(default_value)
        .items(f(cx))
        .into_element(cx)
}

pub fn toggle_group_multiple<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Vec<Arc<str>>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<ToggleGroupItem>,
) -> AnyElement {
    ToggleGroup::multiple(model).items(f(cx)).into_element(cx)
}

pub fn toggle_group_multiple_uncontrolled<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    default_value: Vec<Arc<str>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<ToggleGroupItem>,
) -> AnyElement {
    ToggleGroup::multiple_uncontrolled(default_value)
        .items(f(cx))
        .into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, Point, Px, Rect, SemanticsRole, Size, SvgId, SvgService,
    };
    use fret_core::{PathCommand, PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_ui::tree::UiTree;

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &fret_core::TextInput,
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

    fn render_single_without_roving_focus(
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
            "toggle-group-single-no-roving",
            |cx| {
                let items = vec![
                    ToggleGroupItem::new("alpha", vec![]),
                    ToggleGroupItem::new("beta", vec![]),
                    ToggleGroupItem::new("gamma", vec![]),
                ];
                vec![
                    ToggleGroup::single(model)
                        .roving_focus(false)
                        .items(items)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_single_uncontrolled(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        default_value: Option<Arc<str>>,
    ) -> fret_core::NodeId {
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "toggle-group-single-uncontrolled",
            |cx| {
                let items = vec![
                    ToggleGroupItem::new("alpha", vec![]),
                    ToggleGroupItem::new("beta", vec![]),
                    ToggleGroupItem::new("gamma", vec![]),
                ];
                vec![
                    ToggleGroup::single_uncontrolled(default_value.clone())
                        .items(items)
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_multiple_uncontrolled(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        default_value: Vec<Arc<str>>,
    ) -> fret_core::NodeId {
        let root = fret_ui::declarative::render_root(
            ui,
            app,
            services,
            window,
            bounds,
            "toggle-group-multiple-uncontrolled",
            |cx| {
                let items = vec![
                    ToggleGroupItem::new("alpha", vec![]),
                    ToggleGroupItem::new("beta", vec![]),
                    ToggleGroupItem::new("gamma", vec![]),
                ];
                vec![
                    ToggleGroup::multiple_uncontrolled(default_value.clone())
                        .items(items)
                        .into_element(cx),
                ]
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
        assert_eq!(focused_node.role, SemanticsRole::RadioButton);
        assert_eq!(focused_node.label.as_deref(), Some("beta"));
    }

    #[test]
    fn toggle_group_single_without_roving_focus_does_not_move_focus_on_arrow() {
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

        let root = render_single_without_roving_focus(
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

        let _ = render_single_without_roving_focus(
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
        assert_eq!(focused_node.label.as_deref(), Some("alpha"));
    }

    #[test]
    fn toggle_group_single_uncontrolled_applies_default_value_once_and_allows_deactivate() {
        fn checked(ui: &UiTree<App>, label: &str) -> Option<bool> {
            ui.semantics_snapshot()
                .expect("semantics snapshot")
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::RadioButton && n.label.as_deref() == Some(label))
                .and_then(|n| n.flags.checked)
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let root = render_single_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(Arc::from("alpha")),
        );
        assert_eq!(checked(&ui, "alpha"), Some(true));

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

        let _ = render_single_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(Arc::from("alpha")),
        );
        assert_eq!(checked(&ui, "alpha"), Some(false));

        // The internal model should not be reset by repeatedly passing the same default value.
        let _ = render_single_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(Arc::from("alpha")),
        );
        assert_eq!(checked(&ui, "alpha"), Some(false));
    }

    #[test]
    fn toggle_group_multiple_uncontrolled_applies_default_value_once_and_allows_toggle() {
        fn selected(ui: &UiTree<App>, label: &str) -> bool {
            ui.semantics_snapshot()
                .expect("semantics snapshot")
                .nodes
                .iter()
                .find(|n| n.role == SemanticsRole::Button && n.label.as_deref() == Some(label))
                .is_some_and(|n| n.flags.selected)
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices::default();

        let default_value = vec![Arc::from("alpha"), Arc::from("gamma")];
        let root = render_multiple_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            default_value.clone(),
        );
        assert!(selected(&ui, "alpha"));
        assert!(selected(&ui, "gamma"));

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

        let _ = render_multiple_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            default_value.clone(),
        );
        assert!(!selected(&ui, "alpha"));
        assert!(selected(&ui, "gamma"));

        // The internal model should not be reset by repeatedly passing the same default value.
        let _ = render_multiple_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            default_value,
        );
        assert!(!selected(&ui, "alpha"));
        assert!(selected(&ui, "gamma"));
    }
}
