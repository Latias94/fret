use std::sync::Arc;

use fret_core::{
    Color, Corners, Edges, FontId, FontWeight, Point, Px, Rect, Size, TextOverflow, TextStyle,
    TextWrap,
};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableProps, RovingFlexProps, RovingFocusProps, SizeStyle, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::model_watch::ModelWatchExt as _;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::radio_group as radio_group_prim;
use fret_ui_kit::primitives::roving_focus_group;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Space, WidgetState,
    WidgetStateProperty, WidgetStates,
};

fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

fn row_gap(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.radio_group.gap")
        .unwrap_or_else(|| MetricRef::space(Space::N3).resolve(theme))
}

fn label_gap(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.radio_group.label_gap")
        .unwrap_or_else(|| MetricRef::space(Space::N2).resolve(theme))
}

fn icon_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.radio_group.icon_size_px")
        .unwrap_or(Px(16.0))
}

fn indicator_size(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.radio_group.indicator_size_px")
        .unwrap_or(Px(8.0))
}

fn radio_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.radio_group.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.radio_group.line_height")
        .unwrap_or(px);

    TextStyle {
        font: FontId::default(),
        size: px,
        weight: FontWeight::NORMAL,
        slant: Default::default(),
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

fn radio_border(theme: &Theme) -> Color {
    theme
        .color_by_key("input")
        .or_else(|| theme.color_by_key("border"))
        .expect("missing theme token: input/border")
}

fn radio_ring(theme: &Theme) -> Color {
    theme.color_required("ring")
}

fn radio_fg(theme: &Theme) -> Color {
    theme.color_required("foreground")
}

fn radio_indicator(theme: &Theme) -> Color {
    theme.color_required("primary")
}

pub use fret_ui_kit::primitives::radio_group::RadioGroupOrientation;

#[derive(Debug, Clone)]
pub struct RadioGroupItem {
    pub value: Arc<str>,
    pub label: Arc<str>,
    pub children: Option<Vec<AnyElement>>,
    pub disabled: bool,
}

impl RadioGroupItem {
    pub fn new(value: impl Into<Arc<str>>, label: impl Into<Arc<str>>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            children: None,
            disabled: false,
        }
    }

    /// Overrides the default item contents (icon + label text) to enable composable labels.
    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }

    pub fn child(mut self, child: AnyElement) -> Self {
        self.children = Some(vec![child]);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct RadioGroupStyle {
    pub icon_border_color: Option<WidgetStateProperty<ColorRef>>,
    pub label_color: Option<WidgetStateProperty<ColorRef>>,
    pub indicator_color: Option<WidgetStateProperty<ColorRef>>,
}

impl RadioGroupStyle {
    pub fn icon_border_color(mut self, icon_border_color: WidgetStateProperty<ColorRef>) -> Self {
        self.icon_border_color = Some(icon_border_color);
        self
    }

    pub fn label_color(mut self, label_color: WidgetStateProperty<ColorRef>) -> Self {
        self.label_color = Some(label_color);
        self
    }

    pub fn indicator_color(mut self, indicator_color: WidgetStateProperty<ColorRef>) -> Self {
        self.indicator_color = Some(indicator_color);
        self
    }

    pub fn merged(mut self, other: Self) -> Self {
        if other.icon_border_color.is_some() {
            self.icon_border_color = other.icon_border_color;
        }
        if other.label_color.is_some() {
            self.label_color = other.label_color;
        }
        if other.indicator_color.is_some() {
            self.indicator_color = other.indicator_color;
        }
        self
    }
}

#[derive(Clone)]
pub struct RadioGroup {
    model: Option<Model<Option<Arc<str>>>>,
    default_value: Option<Arc<str>>,
    items: Vec<RadioGroupItem>,
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    orientation: RadioGroupOrientation,
    loop_navigation: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    style: RadioGroupStyle,
}

impl RadioGroup {
    pub fn new(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model: Some(model),
            default_value: None,
            items: Vec::new(),
            disabled: false,
            a11y_label: None,
            orientation: RadioGroupOrientation::default(),
            loop_navigation: true,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: RadioGroupStyle::default(),
        }
    }

    /// Creates an uncontrolled radio group with an optional initial value (Radix `defaultValue`).
    pub fn uncontrolled<T: Into<Arc<str>>>(default_value: Option<T>) -> Self {
        Self {
            model: None,
            default_value: default_value.map(Into::into),
            items: Vec::new(),
            disabled: false,
            a11y_label: None,
            orientation: RadioGroupOrientation::default(),
            loop_navigation: true,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            style: RadioGroupStyle::default(),
        }
    }

    pub fn item(mut self, item: RadioGroupItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    /// Sets the uncontrolled initial selection value (Radix `defaultValue`).
    ///
    /// Note: If a controlled `model` is provided, this value is ignored.
    pub fn default_value<T: Into<Arc<str>>>(mut self, default_value: Option<T>) -> Self {
        self.default_value = default_value.map(Into::into);
        self
    }

    pub fn orientation(mut self, orientation: RadioGroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// When `true` (default), roving navigation loops at the ends (Radix `loop` behavior).
    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
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

    pub fn style(mut self, style: RadioGroupStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Self {
            model,
            default_value,
            items,
            disabled,
            a11y_label,
            orientation,
            loop_navigation,
            chrome,
            layout,
            style,
        } = self;

        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();
            let gap_y = row_gap(&theme);
            let gap_x = label_gap(&theme);
            let icon = icon_size(&theme);
            let indicator = indicator_size(&theme);

            let text_style = radio_text_style(&theme);
            let fg = radio_fg(&theme);
            let border = radio_border(&theme);
            let ring = radio_ring(&theme);
            let dot = radio_indicator(&theme);

            let default_icon_border_color = WidgetStateProperty::new(ColorRef::Color(border))
                .when(WidgetStates::HOVERED, ColorRef::Color(alpha_mul(ring, 0.8)))
                .when(WidgetStates::ACTIVE, ColorRef::Color(alpha_mul(ring, 0.8)))
                .when(WidgetStates::FOCUS_VISIBLE, ColorRef::Color(ring))
                .when(WidgetStates::DISABLED, ColorRef::Color(alpha_mul(border, 0.5)));

            let default_label_color =
                WidgetStateProperty::new(ColorRef::Color(fg)).when(
                    WidgetStates::DISABLED,
                    ColorRef::Color(alpha_mul(alpha_mul(fg, 0.5), 0.8)),
                );

            let default_indicator_color = WidgetStateProperty::new(ColorRef::Color(dot))
                .when(WidgetStates::DISABLED, ColorRef::Color(alpha_mul(dot, 0.8)));

            let group_disabled = disabled;
            let group_label = a11y_label.clone();
            let items = items.clone();
            let style_override = style.clone();
            let model = radio_group_prim::radio_group_use_model(
                cx,
                model.clone(),
                || default_value.clone(),
            )
            .model();

            let selected: Option<Arc<str>> = cx.watch_model(&model).cloned().flatten();
            let values: Vec<Arc<str>> = items.iter().map(|i| i.value.clone()).collect();
            let disabled: Vec<bool> = items.iter().map(|i| group_disabled || i.disabled).collect();
            let active = roving_focus_group::active_index_from_str_keys(
                &values,
                selected.as_deref(),
                &disabled,
            );

            let values_arc: Arc<[Arc<str>]> = Arc::from(values.into_boxed_slice());
            let disabled_arc: Arc<[bool]> = Arc::from(disabled.clone().into_boxed_slice());
            let set_size = u32::try_from(items.len()).ok().and_then(|n| (n > 0).then_some(n));

            let mut radix_root = radio_group_prim::RadioGroupRoot::new(model.clone())
                .disabled(group_disabled)
                .orientation(orientation)
                .loop_navigation(loop_navigation);
            if let Some(label) = group_label.clone() {
                radix_root = radix_root.a11y_label(label);
            }

            let root_for_items = radix_root.clone();
            let list = radix_root.list(values_arc.clone(), disabled_arc.clone());

            let container_props = decl_style::container_props(&theme, chrome, layout);

            let list_element = list.into_element(
                cx,
                RovingFlexProps {
                    flex: FlexProps {
                        gap: match orientation {
                            RadioGroupOrientation::Vertical => gap_y,
                            RadioGroupOrientation::Horizontal => gap_x,
                        },
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: match orientation {
                            RadioGroupOrientation::Vertical => CrossAlign::Stretch,
                            RadioGroupOrientation::Horizontal => CrossAlign::Center,
                        },
                        wrap: false,
                        ..Default::default()
                    },
                    roving: RovingFocusProps::default(),
                },
                move |cx| {
                    let mut out = Vec::with_capacity(items.len());
                    for (idx, item) in items.iter().cloned().enumerate() {
                        let item_disabled = disabled.get(idx).copied().unwrap_or(true);
                        let item_enabled = !item_disabled;
                        let tab_stop = active.is_some_and(|a| a == idx);

                        let radius = Px((icon.0 * 0.5).max(0.0));
                        let ring_style = decl_style::focus_ring(&theme, radius);
                        let pressable_layout = decl_style::layout_style(
                            &theme,
                            fret_ui_kit::LayoutRefinement::default().w_full(),
                        );

                        let a11y_label = item.label.clone();
                        let value = item.value.clone();
                        let item_children = item.children.clone();
                        let text_style = text_style.clone();
                        let root_for_item = root_for_items.clone();
                        let style_override = style_override.clone();
                        let default_icon_border_color = default_icon_border_color.clone();
                        let default_label_color = default_label_color.clone();
                        let default_indicator_color = default_indicator_color.clone();
                        out.push(cx.keyed(value.clone(), move |cx| {
                            radio_group_prim::RadioGroupItem::new(value)
                                .label(a11y_label.clone())
                                .disabled(!item_enabled)
                                .index(idx)
                                .tab_stop(tab_stop)
                                .set_size(set_size)
                                .into_element(
                                    cx,
                                    &root_for_item,
                                    PressableProps {
                                        layout: pressable_layout,
                                        enabled: item_enabled,
                                        focusable: tab_stop,
                                        focus_ring: Some(ring_style),
                                        focus_ring_bounds: Some(Rect::new(
                                            Point::new(Px(0.0), Px(0.0)),
                                            Size::new(icon, icon),
                                        )),
                                        ..Default::default()
                                    },
                                    move |cx, st, checked| {
                                        let theme = Theme::global(&*cx.app).clone();

                                        let mut states =
                                            WidgetStates::from_pressable(cx, st, item_enabled);
                                        states.set(WidgetState::Selected, checked);

                                        let border_prop = style_override
                                            .icon_border_color
                                            .as_ref()
                                            .unwrap_or(&default_icon_border_color);
                                        let label_prop = style_override
                                            .label_color
                                            .as_ref()
                                            .unwrap_or(&default_label_color);
                                        let indicator_prop = style_override
                                            .indicator_color
                                            .as_ref()
                                            .unwrap_or(&default_indicator_color);

                                        let border_color =
                                            border_prop.resolve(states).clone().resolve(&theme);
                                        let fg = label_prop.resolve(states).clone().resolve(&theme);
                                        let dot =
                                            indicator_prop.resolve(states).clone().resolve(&theme);

                                        let icon_layout = decl_style::layout_style(
                                            &theme,
                                            fret_ui_kit::LayoutRefinement::default()
                                                .w_px(MetricRef::Px(icon))
                                                .h_px(MetricRef::Px(icon)),
                                        );
                                        let icon_props = ContainerProps {
                                            layout: icon_layout,
                                            padding: Edges::all(Px(0.0)),
                                            background: None,
                                            shadow: Some(decl_style::shadow_xs(&theme, radius)),
                                            border: Edges::all(Px(1.0)),
                                            border_color: Some(border_color),
                                            corner_radii: Corners::all(radius),
                                            ..Default::default()
                                        };

                                        let row_layout = LayoutStyle {
                                            size: SizeStyle {
                                                width: Length::Fill,
                                                height: Length::Auto,
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        };

                                        let indicator_layout = decl_style::layout_style(
                                            &theme,
                                            fret_ui_kit::LayoutRefinement::default()
                                                .w_px(MetricRef::Px(indicator))
                                                .h_px(MetricRef::Px(indicator)),
                                        );
                                        let indicator_props = ContainerProps {
                                            layout: indicator_layout,
                                            padding: Edges::all(Px(0.0)),
                                            background: Some(dot),
                                            shadow: None,
                                            border: Edges::all(Px(0.0)),
                                            border_color: None,
                                            corner_radii: Corners::all(Px(
                                                (indicator.0 * 0.5).max(0.0),
                                            )),
                                            ..Default::default()
                                        };

                                        let label = a11y_label.clone();
                                        let label_props = TextProps {
                                            layout: LayoutStyle::default(),
                                            text: label,
                                            style: Some(text_style.clone()),
                                            color: Some(fg),
                                            wrap: TextWrap::Word,
                                            overflow: TextOverflow::Clip,
                                        };

                                        vec![cx.flex(
                                            FlexProps {
                                                layout: row_layout,
                                                direction: fret_core::Axis::Horizontal,
                                                gap: gap_x,
                                                padding: Edges::all(Px(0.0)),
                                                justify: MainAlign::Start,
                                                align: CrossAlign::Center,
                                                wrap: false,
                                            },
                                            move |cx| {
                                                let mut out = Vec::new();
                                                out.push(cx.container(icon_props, move |cx| {
                                                    if !checked {
                                                        return Vec::new();
                                                    }

                                                    vec![cx.flex(
                                                        FlexProps {
                                                            layout: decl_style::layout_style(
                                                                &theme,
                                                                fret_ui_kit::LayoutRefinement::default()
                                                                    .size_full(),
                                                            ),
                                                            direction: fret_core::Axis::Horizontal,
                                                            gap: Px(0.0),
                                                            padding: Edges::all(Px(0.0)),
                                                            justify: MainAlign::Center,
                                                            align: CrossAlign::Center,
                                                            wrap: false,
                                                        },
                                                        move |cx| {
                                                            vec![cx.container(
                                                                indicator_props,
                                                                |_cx| Vec::new(),
                                                            )]
                                                        },
                                                    )]
                                                }));

                                                if let Some(children) = item_children.clone() {
                                                    out.extend(children);
                                                } else {
                                                    out.push(cx.text_props(label_props));
                                                }

                                                out
                                            },
                                        )]
                                    },
                                )
                        }));
                    }
                    out
                },
            );

            cx.container(container_props, move |_cx| vec![list_element])
        })
    }
}

pub fn radio_group<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    items: Vec<RadioGroupItem>,
) -> AnyElement {
    let mut group = RadioGroup::new(model);
    for item in items {
        group = group.item(item);
    }
    group.into_element(cx)
}

pub fn radio_group_uncontrolled<H: UiHost, T: Into<Arc<str>>>(
    cx: &mut ElementContext<'_, H>,
    default_value: Option<T>,
    items: Vec<RadioGroupItem>,
) -> AnyElement {
    let mut group = RadioGroup::uncontrolled(default_value);
    for item in items {
        group = group.item(item);
    }
    group.into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_app::App;
    use fret_core::{
        AppWindowId, Modifiers, PathCommand, SemanticsRole, SvgId, SvgService, TextBlobId,
        TextConstraints, TextMetrics, TextService,
    };
    use fret_core::{Event, KeyCode};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{Point, Px, Rect};
    use fret_ui::{Theme, ThemeConfig, UiTree};

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
                    size: fret_core::Size::new(Px(0.0), Px(0.0)),
                    baseline: Px(0.0),
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

    #[test]
    fn radio_group_emits_radio_group_and_radio_button_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let model = app.models_mut().insert(Some(Arc::from("b")));
        let items = vec![
            RadioGroupItem::new("a", "Alpha"),
            RadioGroupItem::new("b", "Beta"),
        ];

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(240.0), Px(160.0)),
        );
        let mut services = FakeServices;

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "test",
            |cx| {
                vec![
                    RadioGroup::new(model.clone())
                        .a11y_label("Options")
                        .item(items[0].clone())
                        .item(items[1].clone())
                        .into_element(cx),
                ]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let mut scene = fret_core::Scene::default();
        ui.paint_all(&mut app, &mut services, bounds, &mut scene, 1.0);

        let snap = ui.semantics_snapshot().expect("semantics snapshot");

        assert!(
            snap.nodes.iter().any(|n| {
                n.role == SemanticsRole::RadioGroup && n.label.as_deref() == Some("Options")
            }),
            "radio group should expose RadioGroup role + label"
        );

        let alpha = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::RadioButton && n.label.as_deref() == Some("Alpha"))
            .expect("Alpha radio");
        assert_eq!(alpha.flags.checked, Some(false));
        assert_eq!(alpha.pos_in_set, Some(1));
        assert_eq!(alpha.set_size, Some(2));

        let beta = snap
            .nodes
            .iter()
            .find(|n| n.role == SemanticsRole::RadioButton && n.label.as_deref() == Some("Beta"))
            .expect("Beta radio");
        assert_eq!(beta.flags.checked, Some(true));
        assert_eq!(beta.pos_in_set, Some(2));
        assert_eq!(beta.set_size, Some(2));
    }

    fn render(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        model: Model<Option<Arc<str>>>,
        orientation: RadioGroupOrientation,
        loop_navigation: bool,
    ) -> fret_core::NodeId {
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![
                    RadioGroup::new(model)
                        .a11y_label("Options")
                        .orientation(orientation)
                        .loop_navigation(loop_navigation)
                        .item(RadioGroupItem::new("alpha", "Alpha"))
                        .item(RadioGroupItem::new("beta", "Beta"))
                        .item(RadioGroupItem::new("gamma", "Gamma"))
                        .into_element(cx),
                ]
            });
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    fn render_uncontrolled(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        default_value: Option<Arc<str>>,
        orientation: RadioGroupOrientation,
        loop_navigation: bool,
    ) -> fret_core::NodeId {
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                vec![
                    RadioGroup::uncontrolled(default_value.clone())
                        .a11y_label("Options")
                        .orientation(orientation)
                        .loop_navigation(loop_navigation)
                        .item(RadioGroupItem::new("alpha", "Alpha"))
                        .item(RadioGroupItem::new("beta", "Beta"))
                        .item(RadioGroupItem::new("gamma", "Gamma"))
                        .into_element(cx),
                ]
            });
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
        root
    }

    #[test]
    fn radio_group_uncontrolled_applies_default_value_once_and_does_not_reset() {
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

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices;

        let root = render_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(Arc::from("alpha")),
            RadioGroupOrientation::Horizontal,
            true,
        );
        assert_eq!(checked(&ui, "Alpha"), Some(true));
        assert_eq!(checked(&ui, "Beta"), Some(false));

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable item");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );

        let _ = render_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(Arc::from("alpha")),
            RadioGroupOrientation::Horizontal,
            true,
        );
        assert_eq!(checked(&ui, "Alpha"), Some(false));
        assert_eq!(checked(&ui, "Beta"), Some(true));

        // The internal model should not be reset by repeatedly passing the same default value.
        let _ = render_uncontrolled(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(Arc::from("alpha")),
            RadioGroupOrientation::Horizontal,
            true,
        );
        assert_eq!(checked(&ui, "Alpha"), Some(false));
        assert_eq!(checked(&ui, "Beta"), Some(true));
    }

    #[test]
    fn radio_group_horizontal_arrow_right_moves_and_selects() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let model = app.models_mut().insert(Some(Arc::from("alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices;

        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            RadioGroupOrientation::Horizontal,
            true,
        );

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable item");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
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
            RadioGroupOrientation::Horizontal,
            true,
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("beta"));

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let focus = snap.focus.expect("focus");
        let focused_node = snap
            .nodes
            .iter()
            .find(|n| n.id == focus)
            .expect("focused node");
        assert_eq!(focused_node.role, SemanticsRole::RadioButton);
        assert_eq!(focused_node.label.as_deref(), Some("Beta"));
    }

    #[test]
    fn radio_group_loop_false_does_not_wrap_at_end() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let model = app.models_mut().insert(Some(Arc::from("gamma")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices;

        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            RadioGroupOrientation::Horizontal,
            false,
        );

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable item");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
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
            RadioGroupOrientation::Horizontal,
            false,
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("gamma"));
    }

    #[test]
    fn radio_group_does_not_select_on_enter_key() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let model = app.models_mut().insert(Some(Arc::from("alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices;

        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            RadioGroupOrientation::Horizontal,
            true,
        );

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable item");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
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
            RadioGroupOrientation::Horizontal,
            true,
        );

        let _ = app
            .models_mut()
            .update(&model, |v| *v = Some(Arc::from("alpha")));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyUp {
                key: KeyCode::Enter,
                modifiers: Modifiers::default(),
            },
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("alpha"));
    }

    #[test]
    fn radio_group_selects_on_space_key() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Test".to_string(),
                ..ThemeConfig::default()
            });
        });

        let model = app.models_mut().insert(Some(Arc::from("alpha")));

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            fret_core::Size::new(Px(400.0), Px(240.0)),
        );
        let mut services = FakeServices;

        let root = render(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            model.clone(),
            RadioGroupOrientation::Horizontal,
            true,
        );

        let focusable = ui
            .first_focusable_descendant_including_declarative(&mut app, window, root)
            .expect("focusable item");
        ui.set_focus(Some(focusable));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::ArrowRight,
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
            RadioGroupOrientation::Horizontal,
            true,
        );

        let _ = app
            .models_mut()
            .update(&model, |v| *v = Some(Arc::from("alpha")));

        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyDown {
                key: KeyCode::Space,
                modifiers: Modifiers::default(),
                repeat: false,
            },
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &Event::KeyUp {
                key: KeyCode::Space,
                modifiers: Modifiers::default(),
            },
        );

        let selected = app.models().get_cloned(&model).flatten();
        assert_eq!(selected.as_deref(), Some("beta"));
    }
}
