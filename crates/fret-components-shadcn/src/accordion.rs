use std::sync::Arc;

use fret_components_ui::declarative::action_hooks::ActionHooksExt as _;
use fret_components_ui::declarative::model_watch::ModelWatchExt as _;
use fret_components_ui::declarative::style as decl_style;
use fret_components_ui::headless::roving_focus;
use fret_components_ui::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space};
use fret_core::{
    Color, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap,
};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, LayoutStyle, MainAlign, PressableA11y,
    PressableProps, RovingFlexProps, RovingFocusProps, RowProps, SemanticsProps, TextProps,
};
use fret_ui::{ElementCx, Theme, UiHost};

fn border_color(theme: &Theme) -> Color {
    theme
        .color_by_key("border")
        .or_else(|| theme.color_by_key("input"))
        .unwrap_or(theme.colors.panel_border)
}

fn trigger_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.accordion.trigger.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or(theme.metrics.font_size);
    let line_height = theme
        .metric_by_key("component.accordion.trigger.line_height")
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccordionKind {
    Single,
    Multiple,
}

#[derive(Clone)]
enum AccordionModel {
    Single {
        model: Model<Option<Arc<str>>>,
        collapsible: bool,
    },
    Multiple {
        model: Model<Vec<Arc<str>>>,
    },
}

#[derive(Clone)]
pub struct AccordionTrigger {
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl std::fmt::Debug for AccordionTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccordionTrigger")
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl AccordionTrigger {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            disabled: false,
            a11y_label: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            children,
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

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    fn into_element<H: UiHost>(
        self,
        cx: &mut ElementCx<'_, H>,
        model: AccordionModel,
        value: Arc<str>,
        enabled: bool,
        focusable: bool,
        is_open: bool,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let a11y_label = self.a11y_label.unwrap_or_else(|| value.clone());
        let text_style = trigger_text_style(&theme);
        let fg = theme
            .color_by_key("foreground")
            .unwrap_or(theme.colors.text_primary);
        let radius = MetricRef::radius(Radius::Md).resolve(&theme);

        let pressable_layout = decl_style::layout_style(
            &theme,
            self.layout
                .merge(LayoutRefinement::default().w_full().min_w_0()),
        );
        let container_layout = pressable_layout;

        let chrome = self.chrome;
        let children = self.children;

        cx.pressable(
            PressableProps {
                layout: pressable_layout,
                enabled,
                focusable,
                focus_ring: Some(decl_style::focus_ring(&theme, radius)),
                a11y: PressableA11y {
                    role: Some(SemanticsRole::Button),
                    label: Some(a11y_label.clone()),
                    expanded: Some(is_open),
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, _state| {
                match model.clone() {
                    AccordionModel::Single { model, collapsible } => {
                        let value = value.clone();
                        cx.pressable_add_on_activate(Arc::new(move |host, _cx, _reason| {
                            let value = value.clone();
                            let _ = host.models_mut().update(&model, |v| {
                                let is_same = v.as_deref().is_some_and(|cur| cur == value.as_ref());
                                if is_same {
                                    if collapsible {
                                        *v = None;
                                    }
                                } else {
                                    *v = Some(value);
                                }
                            });
                        }));
                    }
                    AccordionModel::Multiple { model } => {
                        cx.pressable_toggle_vec_arc_str(&model, value.clone());
                    }
                }

                let chrome = ChromeRefinement::default()
                    .px(Space::N0)
                    .py(Space::N4)
                    .merge(chrome.clone());
                let mut props = decl_style::container_props(&theme, chrome, Default::default());
                props.layout.size = container_layout.size;
                props.layout.overflow = container_layout.overflow;

                vec![cx.container(
                    ContainerProps {
                        layout: props.layout,
                        padding: props.padding,
                        background: props.background,
                        shadow: props.shadow,
                        border: props.border,
                        border_color: props.border_color,
                        corner_radii: props.corner_radii,
                    },
                    move |cx| {
                        vec![cx.row(
                            RowProps {
                                layout: LayoutStyle::default(),
                                gap: MetricRef::space(Space::N2).resolve(&theme),
                                padding: Edges::all(Px(0.0)),
                                justify: MainAlign::SpaceBetween,
                                align: CrossAlign::Center,
                            },
                            move |cx| {
                                if children.is_empty() {
                                    vec![cx.text_props(TextProps {
                                        layout: LayoutStyle::default(),
                                        text: a11y_label.clone(),
                                        style: Some(text_style),
                                        color: Some(fg),
                                        wrap: TextWrap::None,
                                        overflow: TextOverflow::Clip,
                                    })]
                                } else {
                                    children
                                }
                            },
                        )]
                    },
                )]
            },
        )
    }
}

#[derive(Clone)]
pub struct AccordionContent {
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl std::fmt::Debug for AccordionContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccordionContent")
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl AccordionContent {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            children,
        }
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let chrome = ChromeRefinement::default()
            .pt(Space::N0)
            .pb(Space::N4)
            .merge(self.chrome);

        let mut props = decl_style::container_props(
            &theme,
            chrome,
            self.layout.merge(LayoutRefinement::default().w_full()),
        );
        props.layout.overflow = fret_ui::element::Overflow::Clip;

        let children = self.children;

        cx.container(props, move |cx| {
            vec![cx.column(
                ColumnProps {
                    layout: LayoutStyle::default(),
                    gap: Px(0.0),
                    padding: Edges::all(Px(0.0)),
                    justify: MainAlign::Start,
                    align: CrossAlign::Stretch,
                },
                move |_cx| children,
            )]
        })
    }
}

#[derive(Clone)]
pub struct AccordionItem {
    value: Arc<str>,
    trigger: AccordionTrigger,
    content: AccordionContent,
    disabled: bool,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for AccordionItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccordionItem")
            .field("value", &self.value.as_ref())
            .field("disabled", &self.disabled)
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl AccordionItem {
    pub fn new(
        value: impl Into<Arc<str>>,
        trigger: AccordionTrigger,
        content: AccordionContent,
    ) -> Self {
        Self {
            value: value.into(),
            trigger,
            content,
            disabled: false,
            layout: LayoutRefinement::default(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
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
}

#[derive(Clone)]
pub struct Accordion {
    model: AccordionModel,
    items: Vec<AccordionItem>,
    disabled: bool,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Accordion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self.model {
            AccordionModel::Single { .. } => AccordionKind::Single,
            AccordionModel::Multiple { .. } => AccordionKind::Multiple,
        };
        f.debug_struct("Accordion")
            .field("kind", &kind)
            .field("items_len", &self.items.len())
            .field("disabled", &self.disabled)
            .field("layout", &self.layout)
            .finish()
    }
}

impl Accordion {
    pub fn single(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model: AccordionModel::Single {
                model,
                collapsible: false,
            },
            items: Vec::new(),
            disabled: false,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn multiple(model: Model<Vec<Arc<str>>>) -> Self {
        Self {
            model: AccordionModel::Multiple { model },
            items: Vec::new(),
            disabled: false,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        if let AccordionModel::Single {
            model,
            collapsible: _,
        } = self.model
        {
            self.model = AccordionModel::Single { model, collapsible };
        }
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn item(mut self, item: AccordionItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = AccordionItem>) -> Self {
        self.items.extend(items);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementCx<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

            let model = self.model;
            let items = self.items;
            let group_disabled = self.disabled;
            let layout = self.layout;

            let (open_single, open_multi) = match &model {
                AccordionModel::Single { model, .. } => {
                    (cx.watch_model(model).layout().cloned().flatten(), None)
                }
                AccordionModel::Multiple { model } => {
                    (None, cx.watch_model(model).layout().cloned())
                }
            };

            let values: Vec<Arc<str>> = items.iter().map(|i| i.value.clone()).collect();
            let disabled_flags: Vec<bool> =
                items.iter().map(|i| group_disabled || i.disabled).collect();

            let tab_stop = match (open_single.as_deref(), open_multi.as_ref()) {
                (Some(selected), _) => roving_focus::active_index_from_str_keys(
                    &values,
                    Some(selected),
                    &disabled_flags,
                ),
                (_, Some(selected)) => {
                    let first_open_enabled = values.iter().enumerate().find_map(|(idx, v)| {
                        let enabled = !disabled_flags.get(idx).copied().unwrap_or(true);
                        let open = selected.iter().any(|s| s.as_ref() == v.as_ref());
                        (enabled && open).then_some(idx)
                    });
                    first_open_enabled.or_else(|| roving_focus::first_enabled(&disabled_flags))
                }
                _ => roving_focus::first_enabled(&disabled_flags),
            };

            let roving = RovingFocusProps {
                enabled: !group_disabled,
                wrap: true,
                disabled: Arc::from(disabled_flags.clone().into_boxed_slice()),
                ..Default::default()
            };

            let border = border_color(&theme);
            let base_item_chrome = ChromeRefinement {
                border_width: Some(MetricRef::Px(Px(1.0))),
                border_color: Some(ColorRef::Color(border)),
                radius: Some(MetricRef::Px(Px(0.0))),
                ..Default::default()
            };

            let wrapper = decl_style::container_props(&theme, ChromeRefinement::default(), layout);

            cx.container(wrapper, move |cx| {
                vec![cx.semantics(
                    SemanticsProps {
                        role: SemanticsRole::List,
                        disabled: group_disabled,
                        ..Default::default()
                    },
                    move |cx| {
                        vec![cx.roving_flex(
                            RovingFlexProps {
                                flex: fret_ui::element::FlexProps {
                                    direction: fret_core::Axis::Vertical,
                                    gap: Px(0.0),
                                    padding: Edges::all(Px(0.0)),
                                    justify: MainAlign::Start,
                                    align: CrossAlign::Stretch,
                                    wrap: false,
                                    ..Default::default()
                                },
                                roving,
                            },
                            move |cx| {
                                cx.roving_nav_apg();
                                let mut out = Vec::with_capacity(items.len());

                                for (idx, item) in items.into_iter().enumerate() {
                                    let item_disabled =
                                        disabled_flags.get(idx).copied().unwrap_or(true)
                                            || item.trigger.disabled;
                                    let enabled = !item_disabled;
                                    let focusable = tab_stop.is_some_and(|i| i == idx);
                                    let is_open = open_single
                                        .as_deref()
                                        .is_some_and(|v| v == item.value.as_ref())
                                        || open_multi.as_ref().is_some_and(|selected| {
                                            selected
                                                .iter()
                                                .any(|v| v.as_ref() == item.value.as_ref())
                                        });

                                    let trigger = item.trigger.into_element(
                                        cx,
                                        model.clone(),
                                        item.value.clone(),
                                        enabled,
                                        focusable,
                                        is_open,
                                    );

                                    let content = is_open.then(|| item.content.into_element(cx));

                                    let mut props = decl_style::container_props(
                                        &theme,
                                        base_item_chrome.clone().merge(item.chrome),
                                        item.layout.merge(LayoutRefinement::default().w_full()),
                                    );
                                    props.border = Edges {
                                        top: Px(0.0),
                                        right: Px(0.0),
                                        bottom: props.border.bottom,
                                        left: Px(0.0),
                                    };

                                    out.push(cx.container(props, move |_cx| {
                                        let mut children = Vec::new();
                                        children.push(trigger);
                                        if let Some(content) = content {
                                            children.push(content);
                                        }
                                        children
                                    }));
                                }

                                out
                            },
                        )]
                    },
                )]
            })
        })
    }
}

pub fn accordion_single<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    model: Model<Option<Arc<str>>>,
    f: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<AccordionItem>,
) -> AnyElement {
    Accordion::single(model).items(f(cx)).into_element(cx)
}

pub fn accordion_multiple<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    model: Model<Vec<Arc<str>>>,
    f: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<AccordionItem>,
) -> AnyElement {
    Accordion::multiple(model).items(f(cx)).into_element(cx)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use fret_app::App;
    use fret_components_ui::LayoutRefinement;
    use fret_components_ui::MetricRef;
    use fret_core::{AppWindowId, PathCommand, Point, Rect, Size, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{
        Px, TextBlobId, TextConstraints, TextMetrics, TextService, TextStyle as CoreTextStyle,
    };
    use fret_ui::UiTree;

    use super::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};

    #[derive(Default)]
    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _text: &str,
            _style: &CoreTextStyle,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(0.0), Px(0.0)),
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

    fn render_accordion_frame(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: fret_runtime::Model<Option<Arc<str>>>,
        collapsible: bool,
    ) {
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let item_1 = AccordionItem::new(
                    Arc::from("item-1"),
                    AccordionTrigger::new(vec![cx.text("Item 1")])
                        .refine_layout(LayoutRefinement::default().h_px(MetricRef::Px(Px(40.0)))),
                    AccordionContent::new(vec![cx.text("Content 1")]),
                );
                let item_2 = AccordionItem::new(
                    Arc::from("item-2"),
                    AccordionTrigger::new(vec![cx.text("Item 2")])
                        .refine_layout(LayoutRefinement::default().h_px(MetricRef::Px(Px(40.0)))),
                    AccordionContent::new(vec![cx.text("Content 2")]),
                );

                let accordion = Accordion::single(open)
                    .collapsible(collapsible)
                    .items([item_1, item_2])
                    .into_element(cx);

                vec![accordion]
            });

        ui.set_root(root);
    }

    #[test]
    fn accordion_single_collapsible_toggles_active_item() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert::<Option<Arc<str>>>(None);
        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        render_accordion_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Click first trigger.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(
            app.models().get_cloned(&open).flatten().as_deref(),
            Some("item-1")
        );

        // Click first trigger again should collapse (collapsible=true).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(app.models().get_cloned(&open).flatten().as_deref(), None);

        // Click second trigger should open item-2.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(10.0), Px(60.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(10.0), Px(60.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(
            app.models().get_cloned(&open).flatten().as_deref(),
            Some("item-2")
        );
    }

    #[test]
    fn accordion_single_non_collapsible_does_not_close_active_item() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert::<Option<Arc<str>>>(None);
        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        render_accordion_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            false,
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        // Open item-1.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(
            app.models().get_cloned(&open).flatten().as_deref(),
            Some("item-1")
        );

        // Click item-1 again should remain open (collapsible=false).
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
            }),
        );
        assert_eq!(
            app.models().get_cloned(&open).flatten().as_deref(),
            Some("item-1")
        );
    }
}
