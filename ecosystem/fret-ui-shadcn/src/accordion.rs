use std::sync::Arc;

use fret_core::{Color, Edges, FontId, FontWeight, Point, Px, TextStyle, Transform2D};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, ElementKind, LayoutStyle, MainAlign,
    OpacityProps, PressableProps, RovingFlexProps, RovingFocusProps, RowProps,
    VisualTransformProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::accordion as radix_accordion;
use fret_ui_kit::primitives::collapsible as radix_collapsible;
use fret_ui_kit::primitives::direction::LayoutDirection;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, ui};

use crate::overlay_motion;

fn border_color(theme: &Theme) -> Color {
    theme
        .color_by_key("border")
        .or_else(|| theme.color_by_key("input"))
        .expect("missing theme token: border/input")
}

fn trigger_text_style(theme: &Theme) -> TextStyle {
    let px = theme
        .metric_by_key("component.accordion.trigger.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.accordion.trigger.line_height")
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

fn trigger_gap(theme: &Theme) -> Px {
    theme
        .metric_by_key("component.accordion.trigger.gap")
        .unwrap_or_else(|| MetricRef::space(Space::N4).resolve(theme))
}

pub use fret_ui_kit::primitives::accordion::{AccordionKind, AccordionOrientation};

type OnValueChange = Arc<dyn Fn(Vec<Arc<str>>) + Send + Sync + 'static>;

#[derive(Default)]
struct AccordionValueChangeCallbackState {
    initialized: bool,
    last_value: Vec<Arc<str>>,
}

fn accordion_value_change_event(
    state: &mut AccordionValueChangeCallbackState,
    next: &[Arc<str>],
) -> Option<Vec<Arc<str>>> {
    if !state.initialized {
        state.initialized = true;
        state.last_value = next.to_vec();
        return None;
    }

    if state.last_value == next {
        return None;
    }

    state.last_value = next.to_vec();
    Some(next.to_vec())
}

/// A Radix-shaped, shadcn-skinned accordion surface (`AccordionRoot` / `AccordionItem` /
/// `AccordionTrigger` / `AccordionContent`).
///
/// This lives alongside the legacy `Accordion` builder so we can offer both:
/// - a JSX-like component family (this module), and
/// - a compact builder-style API (`super::Accordion`) used in some internal recipes/tests.
pub mod composable {
    use super::*;

    #[derive(Clone)]
    enum AccordionModel {
        Single {
            model: Option<Model<Option<Arc<str>>>>,
            default_value: Option<Arc<str>>,
            collapsible: bool,
        },
        Multiple {
            model: Option<Model<Vec<Arc<str>>>>,
            default_value: Vec<Arc<str>>,
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
        pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
            let children = children.into_iter().collect();
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
            cx: &mut ElementContext<'_, H>,
            root: &radix_accordion::AccordionRoot,
            value: Arc<str>,
            enabled: bool,
            focusable: bool,
        ) -> AnyElement {
            let theme = Theme::global(&*cx.app).clone();

            let a11y_label = self.a11y_label.unwrap_or_else(|| value.clone());
            let text_style = trigger_text_style(&theme);
            let fg = theme.color_required("foreground");
            let radius = MetricRef::radius(Radius::Md).resolve(&theme);

            let pressable_layout = decl_style::layout_style(
                &theme,
                self.layout
                    .merge(LayoutRefinement::default().w_full().min_w_0()),
            );
            let container_layout = pressable_layout;

            let chrome = self.chrome;
            let children = self.children;

            radix_accordion::AccordionTrigger::new(value.clone())
                .label(a11y_label.clone())
                .disabled(!enabled)
                .tab_stop(focusable)
                .into_element(
                    cx,
                    root,
                    PressableProps {
                        layout: pressable_layout,
                        focus_ring: Some(decl_style::focus_ring(&theme, radius)),
                        ..Default::default()
                    },
                    move |cx, is_open| {
                        let chrome = ChromeRefinement::default()
                            .px(Space::N0)
                            .py(Space::N4)
                            .rounded(Radius::Md)
                            .merge(chrome.clone());
                        let mut props =
                            decl_style::container_props(&theme, chrome, Default::default());
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
                                ..Default::default()
                            },
                            move |cx| {
                                let chevron_fg = theme
                                    .color_by_key("muted-foreground")
                                    .unwrap_or_else(|| theme.color_required("muted-foreground"));
                                let chevron_layout = decl_style::layout_style(
                                    &theme,
                                    LayoutRefinement::default()
                                        .w_px(Px(16.0))
                                        .h_px(Px(16.0))
                                        .flex_shrink_0(),
                                );
                                let mut chevron_center = Point::new(Px(8.0), Px(8.0));
                                if let (
                                    fret_ui::element::Length::Px(w),
                                    fret_ui::element::Length::Px(h),
                                ) = (chevron_layout.size.width, chevron_layout.size.height)
                                {
                                    chevron_center = Point::new(Px(w.0 * 0.5), Px(h.0 * 0.5));
                                }
                                let chevron_rotation = if is_open { 180.0 } else { 0.0 };
                                let chevron_offset_y =
                                    MetricRef::space(Space::N0p5).resolve(&theme);
                                let chevron_transform =
                                    Transform2D::translation(Point::new(Px(0.0), chevron_offset_y))
                                        * Transform2D::rotation_about_degrees(
                                            chevron_rotation,
                                            chevron_center,
                                        );
                                let chevron = cx.visual_transform_props(
                                    VisualTransformProps {
                                        layout: chevron_layout,
                                        transform: chevron_transform,
                                    },
                                    move |cx| {
                                        vec![decl_icon::icon_with(
                                            cx,
                                            ids::ui::CHEVRON_DOWN,
                                            Some(Px(16.0)),
                                            Some(ColorRef::Color(chevron_fg)),
                                        )]
                                    },
                                );

                                let left_layout = decl_style::layout_style(
                                    &theme,
                                    LayoutRefinement::default().flex_1().min_w_0(),
                                );
                                vec![cx.row(
                                    RowProps {
                                        layout: LayoutStyle::default(),
                                        gap: trigger_gap(&theme),
                                        padding: Edges::all(Px(0.0)),
                                        justify: MainAlign::SpaceBetween,
                                        align: CrossAlign::Start,
                                    },
                                    move |cx| {
                                        let left_children = if children.is_empty() {
                                            let mut label_text = ui::text(cx, a11y_label.clone())
                                                .text_size_px(text_style.size)
                                                .font_weight(text_style.weight)
                                                .text_color(ColorRef::Color(fg))
                                                .nowrap();
                                            if let Some(line_height) = text_style.line_height {
                                                label_text = label_text.line_height_px(line_height);
                                            }
                                            if let Some(letter_spacing_em) =
                                                text_style.letter_spacing_em
                                            {
                                                label_text =
                                                    label_text.letter_spacing_em(letter_spacing_em);
                                            }
                                            vec![label_text.into_element(cx)]
                                        } else {
                                            children
                                        };

                                        vec![
                                            cx.container(
                                                ContainerProps {
                                                    layout: left_layout,
                                                    ..Default::default()
                                                },
                                                |_cx| left_children,
                                            ),
                                            chevron,
                                        ]
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
        pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
            let children = children.into_iter().collect();
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

        fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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
                        gap: MetricRef::space(Space::N4).resolve(&theme),
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
        trigger: Option<AccordionTrigger>,
        content: Option<AccordionContent>,
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
        pub fn new(value: impl Into<Arc<str>>) -> Self {
            Self {
                value: value.into(),
                trigger: None,
                content: None,
                disabled: false,
                layout: LayoutRefinement::default(),
                chrome: ChromeRefinement::default(),
            }
        }

        pub fn trigger(mut self, trigger: AccordionTrigger) -> Self {
            self.trigger = Some(trigger);
            self
        }

        pub fn content(mut self, content: AccordionContent) -> Self {
            self.content = Some(content);
            self
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
    pub struct AccordionRoot {
        model: AccordionModel,
        items: Vec<AccordionItem>,
        disabled: bool,
        layout: LayoutRefinement,
        loop_navigation: bool,
        orientation: AccordionOrientation,
        dir: Option<LayoutDirection>,
        on_value_change: Option<OnValueChange>,
    }

    impl std::fmt::Debug for AccordionRoot {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let kind = match &self.model {
                AccordionModel::Single { .. } => AccordionKind::Single,
                AccordionModel::Multiple { .. } => AccordionKind::Multiple,
            };
            f.debug_struct("AccordionRoot")
                .field("kind", &kind)
                .field("items_len", &self.items.len())
                .field("disabled", &self.disabled)
                .field("layout", &self.layout)
                .field("loop_navigation", &self.loop_navigation)
                .field("on_value_change", &self.on_value_change.is_some())
                .finish()
        }
    }

    impl AccordionRoot {
        pub fn single(model: Model<Option<Arc<str>>>) -> Self {
            Self {
                model: AccordionModel::Single {
                    model: Some(model),
                    default_value: None,
                    collapsible: false,
                },
                items: Vec::new(),
                disabled: false,
                layout: LayoutRefinement::default(),
                loop_navigation: true,
                orientation: AccordionOrientation::default(),
                dir: None,
                on_value_change: None,
            }
        }

        #[cfg(test)]
        pub(super) fn has_on_value_change_handler(&self) -> bool {
            self.on_value_change.is_some()
        }

        /// Creates an uncontrolled accordion with an optional initial value (Radix `defaultValue`).
        pub fn single_uncontrolled<T: Into<Arc<str>>>(default_value: Option<T>) -> Self {
            Self {
                model: AccordionModel::Single {
                    model: None,
                    default_value: default_value.map(Into::into),
                    collapsible: false,
                },
                items: Vec::new(),
                disabled: false,
                layout: LayoutRefinement::default(),
                loop_navigation: true,
                orientation: AccordionOrientation::default(),
                dir: None,
                on_value_change: None,
            }
        }

        pub fn multiple(model: Model<Vec<Arc<str>>>) -> Self {
            Self {
                model: AccordionModel::Multiple {
                    model: Some(model),
                    default_value: Vec::new(),
                },
                items: Vec::new(),
                disabled: false,
                layout: LayoutRefinement::default(),
                loop_navigation: true,
                orientation: AccordionOrientation::default(),
                dir: None,
                on_value_change: None,
            }
        }

        /// Creates an uncontrolled accordion with an initial set of values (Radix `defaultValue`).
        pub fn multiple_uncontrolled<I, T>(default_value: I) -> Self
        where
            I: IntoIterator<Item = T>,
            T: Into<Arc<str>>,
        {
            let default_value = default_value.into_iter().map(Into::into).collect();
            Self {
                model: AccordionModel::Multiple {
                    model: None,
                    default_value,
                },
                items: Vec::new(),
                disabled: false,
                layout: LayoutRefinement::default(),
                loop_navigation: true,
                orientation: AccordionOrientation::default(),
                dir: None,
                on_value_change: None,
            }
        }

        pub fn collapsible(mut self, collapsible: bool) -> Self {
            if let AccordionModel::Single {
                model,
                default_value,
                collapsible: _,
            } = self.model
            {
                self.model = AccordionModel::Single {
                    model,
                    default_value,
                    collapsible,
                };
            }
            self
        }

        pub fn disabled(mut self, disabled: bool) -> Self {
            self.disabled = disabled;
            self
        }

        /// When `true` (default), arrow key navigation loops at the ends (Radix `loop` behavior).
        pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
            self.loop_navigation = loop_navigation;
            self
        }

        /// Controls the keyboard navigation axis for the accordion triggers.
        pub fn orientation(mut self, orientation: AccordionOrientation) -> Self {
            self.orientation = orientation;
            self
        }

        /// Overrides the local direction for horizontal keyboard navigation.
        pub fn dir(mut self, dir: Option<LayoutDirection>) -> Self {
            self.dir = dir;
            self
        }

        /// Called when expanded values change (Base UI `onValueChange`).
        pub fn on_value_change(mut self, on_value_change: Option<OnValueChange>) -> Self {
            self.on_value_change = on_value_change;
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

        pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
            cx.scope(|cx| {
                let theme = Theme::global(&*cx.app).clone();

                let model = self.model;
                let items = self.items;
                let group_disabled = self.disabled;
                let layout = self.layout;
                let loop_navigation = self.loop_navigation;
                let orientation = self.orientation;
                let dir = self.dir;
                let on_value_change = self.on_value_change;

                let root = match &model {
                    AccordionModel::Single {
                        model,
                        default_value,
                        collapsible,
                    } => radix_accordion::AccordionRoot::single_controllable(
                        cx,
                        model.clone(),
                        || default_value.clone(),
                    )
                    .collapsible(*collapsible),
                    AccordionModel::Multiple {
                        model,
                        default_value,
                    } => radix_accordion::AccordionRoot::multiple_controllable(
                        cx,
                        model.clone(),
                        || default_value.clone(),
                    ),
                }
                .disabled(group_disabled)
                .loop_navigation(loop_navigation)
                .orientation(orientation)
                .dir(dir);

                let values: Vec<Arc<str>> = items.iter().map(|i| i.value.clone()).collect();
                if let Some(on_value_change) = on_value_change.as_ref() {
                    let next = values
                        .iter()
                        .filter(|value| root.is_item_open(cx, value.as_ref()))
                        .cloned()
                        .collect::<Vec<_>>();
                    let changed = cx
                        .with_state(AccordionValueChangeCallbackState::default, |state| {
                            accordion_value_change_event(state, &next)
                        });
                    if let Some(next) = changed {
                        on_value_change(next);
                    }
                }
                let disabled_flags: Vec<bool> =
                    items.iter().map(|i| group_disabled || i.disabled).collect();

                let values_arc: Arc<[Arc<str>]> = Arc::from(values.clone().into_boxed_slice());
                let disabled_arc: Arc<[bool]> =
                    Arc::from(disabled_flags.clone().into_boxed_slice());
                let mut list_layout = LayoutStyle::default();
                list_layout.size.width = fret_ui::element::Length::Fill;
                let list = root
                    .clone()
                    .list(values_arc, disabled_arc.clone())
                    .layout(list_layout);
                let tab_stop = list.tab_stop_index(cx).or_else(|| {
                    fret_ui_kit::primitives::roving_focus_group::first_enabled(&disabled_flags)
                });

                let roving = RovingFocusProps {
                    enabled: !group_disabled,
                    wrap: loop_navigation,
                    disabled: disabled_arc.clone(),
                    ..Default::default()
                };

                let border = border_color(&theme);
                let base_item_chrome = ChromeRefinement::default()
                    .border_width(Px(1.0))
                    .border_color(ColorRef::Color(border))
                    .radius(Px(0.0));

                let wrapper =
                    decl_style::container_props(&theme, ChromeRefinement::default(), layout);

                cx.container(wrapper, move |cx| {
                    let mut flex = fret_ui::element::FlexProps {
                        direction: fret_core::Axis::Vertical,
                        gap: Px(0.0),
                        padding: Edges::all(Px(0.0)),
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                        wrap: false,
                        ..Default::default()
                    };
                    flex.layout.size.width = fret_ui::element::Length::Fill;

                    vec![
                        list.into_element(cx, RovingFlexProps { flex, roving }, move |cx| {
                            let mut out = Vec::with_capacity(items.len());
                            let item_len = items.len();

                            for (idx, item) in items.into_iter().enumerate() {
                                let trigger =
                                    item.trigger.expect("AccordionItem must provide a trigger");
                                let content =
                                    item.content.expect("AccordionItem must provide content");

                                let item_disabled =
                                    disabled_flags.get(idx).copied().unwrap_or(true)
                                        || trigger.disabled;
                                let enabled = !item_disabled;
                                let focusable = tab_stop.is_some_and(|i| i == idx);
                                let is_open = root.is_item_open(cx, item.value.as_ref());

                                let trigger = trigger.into_element(
                                    cx,
                                    &root,
                                    item.value.clone(),
                                    enabled,
                                    focusable,
                                );

                                let theme = theme.clone();
                                let value = item.value.clone();
                                let content = content.clone();

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
                                if idx + 1 == item_len {
                                    props.border.bottom = Px(0.0);
                                }

                                out.push(cx.container(props, move |cx| {
                                    let mut children = Vec::new();

                                    let motion =
                                        cx.keyed(("accordion-motion", value.clone()), |cx| {
                                            radix_collapsible::measured_height_motion_for_root(
                                                cx,
                                                is_open,
                                                false,
                                                true,
                                                8,
                                                8,
                                                overlay_motion::shadcn_ease,
                                            )
                                        });

                                    let motion_for_wrapper = motion.clone();
                                    let motion_for_update = motion.clone();
                                    let theme_for_wrapper = theme.clone();

                                    let (content_id, wrapper_el) =
                                        cx.keyed(("accordion-content", value.clone()), move |cx| {
                                            let content_id = cx.root_id();
                                            if !motion_for_wrapper.should_render {
                                                return (content_id, None);
                                            }

                                            let wrapper_refinement =
                                                motion_for_wrapper.wrapper_refinement.clone();
                                            let wrapper_layout = decl_style::layout_style(
                                                &theme_for_wrapper,
                                                wrapper_refinement,
                                            );

                                            let children = vec![cx.opacity_props(
                                                OpacityProps {
                                                    layout: LayoutStyle::default(),
                                                    opacity: motion_for_wrapper.wrapper_opacity,
                                                },
                                                move |cx| vec![content.clone().into_element(cx)],
                                            )];

                                            let wrapper_el = AnyElement::new(
                                                content_id,
                                                ElementKind::Container(ContainerProps {
                                                    layout: wrapper_layout,
                                                    ..Default::default()
                                                }),
                                                children,
                                            );

                                            (content_id, Some(wrapper_el))
                                        });

                                    let trigger = radix_accordion::apply_accordion_trigger_controls(
                                        trigger, content_id,
                                    );
                                    children.push(trigger);

                                    if let Some(wrapper_el) = wrapper_el {
                                        let _ = radix_collapsible::update_measured_for_motion(
                                            cx,
                                            motion_for_update,
                                            wrapper_el.id,
                                        );
                                        children.push(wrapper_el);
                                    }

                                    children
                                }));
                            }

                            out
                        }),
                    ]
                })
            })
        }
    }
}

#[derive(Clone)]
enum AccordionModel {
    Single {
        model: Option<Model<Option<Arc<str>>>>,
        default_value: Option<Arc<str>>,
        collapsible: bool,
    },
    Multiple {
        model: Option<Model<Vec<Arc<str>>>>,
        default_value: Vec<Arc<str>>,
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
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
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
        cx: &mut ElementContext<'_, H>,
        root: &radix_accordion::AccordionRoot,
        value: Arc<str>,
        enabled: bool,
        focusable: bool,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let a11y_label = self.a11y_label.unwrap_or_else(|| value.clone());
        let text_style = trigger_text_style(&theme);
        let fg = theme.color_required("foreground");
        let radius = MetricRef::radius(Radius::Md).resolve(&theme);

        let pressable_layout = decl_style::layout_style(
            &theme,
            self.layout
                .merge(LayoutRefinement::default().w_full().min_w_0()),
        );
        let container_layout = pressable_layout;

        let chrome = self.chrome;
        let children = self.children;

        radix_accordion::AccordionTrigger::new(value.clone())
            .label(a11y_label.clone())
            .disabled(!enabled)
            .tab_stop(focusable)
            .into_element(
                cx,
                root,
                PressableProps {
                    layout: pressable_layout,
                    focus_ring: Some(decl_style::focus_ring(&theme, radius)),
                    ..Default::default()
                },
                move |cx, is_open| {
                    let chrome = ChromeRefinement::default()
                        .px(Space::N0)
                        .py(Space::N4)
                        .rounded(Radius::Md)
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
                            ..Default::default()
                        },
                        move |cx| {
                            let chevron_fg = theme
                                .color_by_key("muted-foreground")
                                .unwrap_or_else(|| theme.color_required("muted-foreground"));
                            let chevron_layout = decl_style::layout_style(
                                &theme,
                                LayoutRefinement::default()
                                    .w_px(Px(16.0))
                                    .h_px(Px(16.0))
                                    .flex_shrink_0(),
                            );
                            let mut chevron_center = Point::new(Px(8.0), Px(8.0));
                            if let (
                                fret_ui::element::Length::Px(w),
                                fret_ui::element::Length::Px(h),
                            ) = (chevron_layout.size.width, chevron_layout.size.height)
                            {
                                chevron_center = Point::new(Px(w.0 * 0.5), Px(h.0 * 0.5));
                            }
                            let chevron_rotation = if is_open { 180.0 } else { 0.0 };
                            let chevron_offset_y = MetricRef::space(Space::N0p5).resolve(&theme);
                            let chevron_transform =
                                Transform2D::translation(Point::new(Px(0.0), chevron_offset_y))
                                    * Transform2D::rotation_about_degrees(
                                        chevron_rotation,
                                        chevron_center,
                                    );
                            let chevron = cx.visual_transform_props(
                                VisualTransformProps {
                                    layout: chevron_layout,
                                    transform: chevron_transform,
                                },
                                move |cx| {
                                    vec![decl_icon::icon_with(
                                        cx,
                                        ids::ui::CHEVRON_DOWN,
                                        Some(Px(16.0)),
                                        Some(ColorRef::Color(chevron_fg)),
                                    )]
                                },
                            );

                            let left_layout = decl_style::layout_style(
                                &theme,
                                LayoutRefinement::default().flex_1().min_w_0(),
                            );
                            vec![cx.row(
                                RowProps {
                                    layout: LayoutStyle::default(),
                                    gap: trigger_gap(&theme),
                                    padding: Edges::all(Px(0.0)),
                                    justify: MainAlign::SpaceBetween,
                                    align: CrossAlign::Start,
                                },
                                move |cx| {
                                    let left_children = if children.is_empty() {
                                        let mut label_text = ui::text(cx, a11y_label.clone())
                                            .text_size_px(text_style.size)
                                            .font_weight(text_style.weight)
                                            .text_color(ColorRef::Color(fg))
                                            .nowrap();
                                        if let Some(line_height) = text_style.line_height {
                                            label_text = label_text.line_height_px(line_height);
                                        }
                                        if let Some(letter_spacing_em) =
                                            text_style.letter_spacing_em
                                        {
                                            label_text =
                                                label_text.letter_spacing_em(letter_spacing_em);
                                        }
                                        vec![label_text.into_element(cx)]
                                    } else {
                                        children
                                    };

                                    vec![
                                        cx.container(
                                            ContainerProps {
                                                layout: left_layout,
                                                ..Default::default()
                                            },
                                            |_cx| left_children,
                                        ),
                                        chevron,
                                    ]
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
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
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

    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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
                    gap: MetricRef::space(Space::N4).resolve(&theme),
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
    loop_navigation: bool,
    orientation: AccordionOrientation,
    dir: Option<LayoutDirection>,
    on_value_change: Option<OnValueChange>,
}

impl std::fmt::Debug for Accordion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match &self.model {
            AccordionModel::Single { .. } => AccordionKind::Single,
            AccordionModel::Multiple { .. } => AccordionKind::Multiple,
        };
        f.debug_struct("Accordion")
            .field("kind", &kind)
            .field("items_len", &self.items.len())
            .field("disabled", &self.disabled)
            .field("layout", &self.layout)
            .field("loop_navigation", &self.loop_navigation)
            .field("orientation", &self.orientation)
            .field("dir", &self.dir)
            .field("on_value_change", &self.on_value_change.is_some())
            .finish()
    }
}

impl Accordion {
    pub fn single(model: Model<Option<Arc<str>>>) -> Self {
        Self {
            model: AccordionModel::Single {
                model: Some(model),
                default_value: None,
                collapsible: false,
            },
            items: Vec::new(),
            disabled: false,
            layout: LayoutRefinement::default(),
            loop_navigation: true,
            orientation: AccordionOrientation::default(),
            dir: None,
            on_value_change: None,
        }
    }

    /// Creates an uncontrolled accordion with an optional initial value (Radix `defaultValue`).
    pub fn single_uncontrolled<T: Into<Arc<str>>>(default_value: Option<T>) -> Self {
        Self {
            model: AccordionModel::Single {
                model: None,
                default_value: default_value.map(Into::into),
                collapsible: false,
            },
            items: Vec::new(),
            disabled: false,
            layout: LayoutRefinement::default(),
            loop_navigation: true,
            orientation: AccordionOrientation::default(),
            dir: None,
            on_value_change: None,
        }
    }

    pub fn multiple(model: Model<Vec<Arc<str>>>) -> Self {
        Self {
            model: AccordionModel::Multiple {
                model: Some(model),
                default_value: Vec::new(),
            },
            items: Vec::new(),
            disabled: false,
            layout: LayoutRefinement::default(),
            loop_navigation: true,
            orientation: AccordionOrientation::default(),
            dir: None,
            on_value_change: None,
        }
    }

    /// Creates an uncontrolled accordion with an initial set of values (Radix `defaultValue`).
    pub fn multiple_uncontrolled<I, T>(default_value: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<Arc<str>>,
    {
        let default_value = default_value.into_iter().map(Into::into).collect();
        Self {
            model: AccordionModel::Multiple {
                model: None,
                default_value,
            },
            items: Vec::new(),
            disabled: false,
            layout: LayoutRefinement::default(),
            loop_navigation: true,
            orientation: AccordionOrientation::default(),
            dir: None,
            on_value_change: None,
        }
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        if let AccordionModel::Single {
            model,
            default_value,
            collapsible: _,
        } = self.model
        {
            self.model = AccordionModel::Single {
                model,
                default_value,
                collapsible,
            };
        }
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// When `true` (default), arrow key navigation loops at the ends (Radix `loop` behavior).
    pub fn loop_navigation(mut self, loop_navigation: bool) -> Self {
        self.loop_navigation = loop_navigation;
        self
    }

    /// Controls the keyboard navigation axis for the accordion triggers.
    pub fn orientation(mut self, orientation: AccordionOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Overrides the local direction for horizontal keyboard navigation.
    pub fn dir(mut self, dir: Option<LayoutDirection>) -> Self {
        self.dir = dir;
        self
    }

    /// Called when expanded values change (Base UI `onValueChange`).
    pub fn on_value_change(mut self, on_value_change: Option<OnValueChange>) -> Self {
        self.on_value_change = on_value_change;
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let theme = Theme::global(&*cx.app).clone();

            let model = self.model;
            let items = self.items;
            let group_disabled = self.disabled;
            let layout = self.layout;
            let loop_navigation = self.loop_navigation;
            let orientation = self.orientation;
            let dir = self.dir;
            let on_value_change = self.on_value_change;

            let root = match &model {
                AccordionModel::Single {
                    model,
                    default_value,
                    collapsible,
                } => radix_accordion::AccordionRoot::single_controllable(cx, model.clone(), || {
                    default_value.clone()
                })
                .collapsible(*collapsible),
                AccordionModel::Multiple {
                    model,
                    default_value,
                } => {
                    radix_accordion::AccordionRoot::multiple_controllable(cx, model.clone(), || {
                        default_value.clone()
                    })
                }
            }
            .disabled(group_disabled)
            .loop_navigation(loop_navigation)
            .orientation(orientation)
            .dir(dir);

            let values: Vec<Arc<str>> = items.iter().map(|i| i.value.clone()).collect();
            if let Some(on_value_change) = on_value_change.as_ref() {
                let next = values
                    .iter()
                    .filter(|value| root.is_item_open(cx, value.as_ref()))
                    .cloned()
                    .collect::<Vec<_>>();
                let changed = cx.with_state(AccordionValueChangeCallbackState::default, |state| {
                    accordion_value_change_event(state, &next)
                });
                if let Some(next) = changed {
                    on_value_change(next);
                }
            }
            let disabled_flags: Vec<bool> =
                items.iter().map(|i| group_disabled || i.disabled).collect();

            let values_arc: Arc<[Arc<str>]> = Arc::from(values.clone().into_boxed_slice());
            let disabled_arc: Arc<[bool]> = Arc::from(disabled_flags.clone().into_boxed_slice());
            let mut list_layout = LayoutStyle::default();
            list_layout.size.width = fret_ui::element::Length::Fill;
            let list = root
                .clone()
                .list(values_arc, disabled_arc.clone())
                .layout(list_layout);
            let tab_stop = list.tab_stop_index(cx).or_else(|| {
                fret_ui_kit::primitives::roving_focus_group::first_enabled(&disabled_flags)
            });

            let roving = RovingFocusProps {
                enabled: !group_disabled,
                wrap: loop_navigation,
                disabled: disabled_arc.clone(),
                ..Default::default()
            };

            let border = border_color(&theme);
            let base_item_chrome = ChromeRefinement::default()
                .border_width(Px(1.0))
                .border_color(ColorRef::Color(border))
                .radius(Px(0.0));

            let wrapper = decl_style::container_props(&theme, ChromeRefinement::default(), layout);

            cx.container(wrapper, move |cx| {
                vec![list.into_element(
                    cx,
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
                        let mut out = Vec::with_capacity(items.len());
                        let item_len = items.len();

                        for (idx, item) in items.into_iter().enumerate() {
                            let item_disabled = disabled_flags.get(idx).copied().unwrap_or(true)
                                || item.trigger.disabled;
                            let enabled = !item_disabled;
                            let focusable = tab_stop.is_some_and(|i| i == idx);
                            let is_open = root.is_item_open(cx, item.value.as_ref());

                            let trigger = item.trigger.into_element(
                                cx,
                                &root,
                                item.value.clone(),
                                enabled,
                                focusable,
                            );

                            let content = item.content;
                            let theme = theme.clone();
                            let value = item.value.clone();

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
                            if idx + 1 == item_len {
                                props.border.bottom = Px(0.0);
                            }

                            out.push(cx.container(props, move |cx| {
                                let mut children = Vec::new();

                                let motion = cx.keyed(("accordion-motion", value.clone()), |cx| {
                                    radix_collapsible::measured_height_motion_for_root(
                                        cx,
                                        is_open,
                                        false,
                                        true,
                                        8,
                                        8,
                                        overlay_motion::shadcn_ease,
                                    )
                                });

                                let motion_for_wrapper = motion.clone();
                                let motion_for_update = motion.clone();
                                let theme_for_wrapper = theme.clone();
                                let content = content.clone();

                                let (content_id, wrapper_el) =
                                    cx.keyed(("accordion-content", value.clone()), move |cx| {
                                        let content_id = cx.root_id();
                                        if !motion_for_wrapper.should_render {
                                            return (content_id, None);
                                        }

                                        let wrapper_refinement =
                                            motion_for_wrapper.wrapper_refinement.clone();
                                        let wrapper_layout = decl_style::layout_style(
                                            &theme_for_wrapper,
                                            wrapper_refinement,
                                        );

                                        let children = vec![cx.opacity_props(
                                            OpacityProps {
                                                layout: LayoutStyle::default(),
                                                opacity: motion_for_wrapper.wrapper_opacity,
                                            },
                                            move |cx| vec![content.clone().into_element(cx)],
                                        )];

                                        let wrapper_el = AnyElement::new(
                                            content_id,
                                            ElementKind::Container(ContainerProps {
                                                layout: wrapper_layout,
                                                ..Default::default()
                                            }),
                                            children,
                                        );

                                        (content_id, Some(wrapper_el))
                                    });

                                let trigger = radix_accordion::apply_accordion_trigger_controls(
                                    trigger, content_id,
                                );
                                children.push(trigger);

                                if let Some(wrapper_el) = wrapper_el {
                                    let _ = radix_collapsible::update_measured_for_motion(
                                        cx,
                                        motion_for_update,
                                        wrapper_el.id,
                                    );
                                    children.push(wrapper_el);
                                }

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
                            }));
                        }

                        out
                    },
                )]
            })
        })
    }
}

pub fn accordion_single<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AccordionItem>,
{
    Accordion::single(model).items(f(cx)).into_element(cx)
}

pub fn accordion_single_uncontrolled<H: UiHost, T: Into<Arc<str>>, I>(
    cx: &mut ElementContext<'_, H>,
    default_value: Option<T>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AccordionItem>,
{
    Accordion::single_uncontrolled(default_value)
        .items(f(cx))
        .into_element(cx)
}

pub fn accordion_multiple<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Vec<Arc<str>>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AccordionItem>,
{
    Accordion::multiple(model).items(f(cx)).into_element(cx)
}

pub fn accordion_multiple_uncontrolled<H: UiHost, V, I>(
    cx: &mut ElementContext<'_, H>,
    default_value: V,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    V: IntoIterator,
    V::Item: Into<Arc<str>>,
    I: IntoIterator<Item = AccordionItem>,
{
    Accordion::multiple_uncontrolled(default_value)
        .items(f(cx))
        .into_element(cx)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::Mutex;

    use fret_app::App;
    use fret_core::{AppWindowId, PathCommand, Point, Rect, Size, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{Px, TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_runtime::{FrameId, TickId};
    use fret_ui::UiTree;
    use fret_ui_kit::LayoutRefinement;

    use super::{
        Accordion, AccordionContent, AccordionItem, AccordionTrigger,
        composable as composable_accordion,
    };

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
                        .refine_layout(LayoutRefinement::default().h_px(Px(40.0))),
                    AccordionContent::new(vec![cx.text("Content 1")]),
                );
                let item_2 = AccordionItem::new(
                    Arc::from("item-2"),
                    AccordionTrigger::new(vec![cx.text("Item 2")])
                        .refine_layout(LayoutRefinement::default().h_px(Px(40.0))),
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

    fn render_accordion_frame_with_on_value_change(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: fret_runtime::Model<Option<Arc<str>>>,
        on_value_change: Option<Arc<dyn Fn(Vec<Arc<str>>) + Send + Sync + 'static>>,
    ) {
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let item_1 = AccordionItem::new(
                    Arc::from("item-1"),
                    AccordionTrigger::new(vec![cx.text("Item 1")])
                        .refine_layout(LayoutRefinement::default().h_px(Px(40.0))),
                    AccordionContent::new(vec![cx.text("Content 1")]),
                );
                let item_2 = AccordionItem::new(
                    Arc::from("item-2"),
                    AccordionTrigger::new(vec![cx.text("Item 2")])
                        .refine_layout(LayoutRefinement::default().h_px(Px(40.0))),
                    AccordionContent::new(vec![cx.text("Content 2")]),
                );

                let accordion = Accordion::single(open)
                    .on_value_change(on_value_change.clone())
                    .items([item_1, item_2])
                    .into_element(cx);

                vec![accordion]
            });

        ui.set_root(root);
    }

    fn render_accordion_frame_composable(
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
                let item_1 = composable_accordion::AccordionItem::new(Arc::from("item-1"))
                    .trigger(
                        composable_accordion::AccordionTrigger::new(vec![cx.text("Item 1")])
                            .refine_layout(LayoutRefinement::default().h_px(Px(40.0))),
                    )
                    .content(composable_accordion::AccordionContent::new(vec![
                        cx.text("Content 1"),
                    ]));

                let item_2 = composable_accordion::AccordionItem::new(Arc::from("item-2"))
                    .trigger(
                        composable_accordion::AccordionTrigger::new(vec![cx.text("Item 2")])
                            .refine_layout(LayoutRefinement::default().h_px(Px(40.0))),
                    )
                    .content(composable_accordion::AccordionContent::new(vec![
                        cx.text("Content 2"),
                    ]));

                let accordion = composable_accordion::AccordionRoot::single(open)
                    .collapsible(collapsible)
                    .items([item_1, item_2])
                    .into_element(cx);

                vec![accordion]
            });

        ui.set_root(root);
    }

    fn render_accordion_frame_composable_with_semantics(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: fret_runtime::Model<Option<Arc<str>>>,
        collapsible: bool,
    ) {
        app.set_tick_id(TickId(app.tick_id().0.saturating_add(1)));
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));

        render_accordion_frame_composable(ui, app, services, window, bounds, open, collapsible);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
    }

    fn render_accordion_frame_uncontrolled(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        default_value: Option<Arc<str>>,
        collapsible: bool,
    ) {
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let item_1 = AccordionItem::new(
                    Arc::from("item-1"),
                    AccordionTrigger::new(vec![cx.text("Item 1")])
                        .refine_layout(LayoutRefinement::default().h_px(Px(40.0))),
                    AccordionContent::new(vec![cx.text("Content 1")]),
                );
                let item_2 = AccordionItem::new(
                    Arc::from("item-2"),
                    AccordionTrigger::new(vec![cx.text("Item 2")])
                        .refine_layout(LayoutRefinement::default().h_px(Px(40.0))),
                    AccordionContent::new(vec![cx.text("Content 2")]),
                );

                let accordion = Accordion::single_uncontrolled(default_value.clone())
                    .collapsible(collapsible)
                    .items([item_1, item_2])
                    .into_element(cx);

                vec![accordion]
            });

        ui.set_root(root);
    }

    fn render_accordion_frame_with_semantics(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: fret_runtime::Model<Option<Arc<str>>>,
        collapsible: bool,
    ) {
        app.set_tick_id(TickId(app.tick_id().0.saturating_add(1)));
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));

        render_accordion_frame(ui, app, services, window, bounds, open, collapsible);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
    }

    fn render_accordion_frame_uncontrolled_with_semantics(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        default_value: Option<Arc<str>>,
        collapsible: bool,
    ) {
        app.set_tick_id(TickId(app.tick_id().0.saturating_add(1)));
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));

        render_accordion_frame_uncontrolled(
            ui,
            app,
            services,
            window,
            bounds,
            default_value,
            collapsible,
        );
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
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
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
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
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_cloned(&open).flatten().as_deref(), None);

        // Click second trigger should open item-2.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(60.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(60.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(
            app.models().get_cloned(&open).flatten().as_deref(),
            Some("item-2")
        );
    }

    #[test]
    fn accordion_single_collapsible_toggles_active_item_composable() {
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

        render_accordion_frame_composable(
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
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
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
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(app.models().get_cloned(&open).flatten().as_deref(), None);

        // Click second trigger should open item-2.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(60.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(60.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
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
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
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
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(10.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        assert_eq!(
            app.models().get_cloned(&open).flatten().as_deref(),
            Some("item-1")
        );
    }

    #[derive(Default)]
    struct MeasuredServices;

    impl TextService for MeasuredServices {
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

    impl PathService for MeasuredServices {
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

    impl SvgService for MeasuredServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    #[test]
    fn accordion_content_remains_mounted_for_close_animation_when_measured() {
        fn snapshot_has_label(ui: &UiTree<App>, label: &str) -> bool {
            ui.semantics_snapshot()
                .expect("semantics snapshot")
                .nodes
                .iter()
                .any(|n| n.label.as_deref() == Some(label))
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app.models_mut().insert::<Option<Arc<str>>>(None);
        let mut services = MeasuredServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        render_accordion_frame_with_semantics(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
        );
        assert!(!snapshot_has_label(&ui, "Content 1"));

        let _ = app
            .models_mut()
            .update(&open, |v| *v = Some(Arc::from("item-1")));

        // Render enough frames for presence to settle and for height to be measured.
        for _ in 0..12 {
            render_accordion_frame_with_semantics(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                true,
            );
        }
        assert!(snapshot_has_label(&ui, "Content 1"));

        let _ = app.models_mut().update(&open, |v| *v = None);

        // First close frame: content should still be mounted (present=true) for the transition.
        render_accordion_frame_with_semantics(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            true,
        );
        assert!(snapshot_has_label(&ui, "Content 1"));

        // After enough frames, presence completes and content unmounts.
        for _ in 0..16 {
            render_accordion_frame_with_semantics(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                true,
            );
        }
        assert!(!snapshot_has_label(&ui, "Content 1"));
    }

    #[test]
    fn accordion_single_uncontrolled_applies_default_value_once_and_does_not_reset() {
        fn snapshot_has_label(ui: &UiTree<App>, label: &str) -> bool {
            ui.semantics_snapshot()
                .expect("semantics snapshot")
                .nodes
                .iter()
                .any(|n| n.label.as_deref() == Some(label))
        }

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        render_accordion_frame_uncontrolled_with_semantics(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            Some(Arc::from("item-1")),
            false,
        );
        assert!(snapshot_has_label(&ui, "Content 1"));

        // Click second trigger to open item-2.
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(60.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(60.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        // Render enough frames for presence to settle and for the previous content to unmount.
        for _ in 0..24 {
            render_accordion_frame_uncontrolled_with_semantics(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                Some(Arc::from("item-1")),
                false,
            );
        }

        assert!(snapshot_has_label(&ui, "Content 2"));
        assert!(!snapshot_has_label(&ui, "Content 1"));

        // The internal model should not reset back to default_value on subsequent renders.
        for _ in 0..8 {
            render_accordion_frame_uncontrolled_with_semantics(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                Some(Arc::from("item-1")),
                false,
            );
        }
        assert!(snapshot_has_label(&ui, "Content 2"));
        assert!(!snapshot_has_label(&ui, "Content 1"));
    }

    #[test]
    fn accordion_trigger_controls_resolves_to_content_when_open() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app
            .models_mut()
            .insert::<Option<Arc<str>>>(Some(Arc::from("item-1")));
        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        for _ in 0..4 {
            render_accordion_frame_with_semantics(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                true,
            );
        }

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_node = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == fret_core::SemanticsRole::Button && n.label.as_deref() == Some("item-1")
            })
            .expect("trigger semantics node");

        assert!(
            trigger_node
                .controls
                .iter()
                .any(|id| snap.nodes.iter().any(|n| n.id == *id)),
            "expected trigger controls relationship to resolve when content is mounted"
        );
    }

    #[test]
    fn accordion_trigger_controls_resolves_to_content_when_open_composable() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app
            .models_mut()
            .insert::<Option<Arc<str>>>(Some(Arc::from("item-1")));
        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        for _ in 0..4 {
            render_accordion_frame_composable_with_semantics(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                true,
            );
        }

        let snap = ui.semantics_snapshot().expect("semantics snapshot");
        let trigger_node = snap
            .nodes
            .iter()
            .find(|n| {
                n.role == fret_core::SemanticsRole::Button && n.label.as_deref() == Some("item-1")
            })
            .expect("trigger semantics node");

        assert!(
            trigger_node
                .controls
                .iter()
                .any(|id| snap.nodes.iter().any(|n| n.id == *id)),
            "expected trigger controls relationship to resolve when content is mounted"
        );
    }

    #[test]
    fn accordion_on_value_change_builder_sets_handler() {
        let mut app = App::new();
        let open = app
            .models_mut()
            .insert::<Option<Arc<str>>>(Some(Arc::from("item-1")));

        let accordion = Accordion::single(open).on_value_change(Some(Arc::new(|_value| {})));
        assert!(accordion.on_value_change.is_some());
    }

    #[test]
    fn accordion_root_on_value_change_builder_sets_handler() {
        let mut app = App::new();
        let open = app
            .models_mut()
            .insert::<Option<Arc<str>>>(Some(Arc::from("item-1")));

        let accordion = composable_accordion::AccordionRoot::single(open)
            .on_value_change(Some(Arc::new(|_value| {})));
        assert!(accordion.has_on_value_change_handler());
    }

    #[test]
    fn accordion_on_value_change_fires_once_when_selection_changes() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app
            .models_mut()
            .insert::<Option<Arc<str>>>(Some(Arc::from("item-1")));
        let changed_values: Arc<Mutex<Vec<Vec<Arc<str>>>>> = Arc::new(Mutex::new(Vec::new()));
        let on_value_change: Arc<dyn Fn(Vec<Arc<str>>) + Send + Sync + 'static> = Arc::new({
            let changed_values = changed_values.clone();
            move |value| {
                changed_values
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(value);
            }
        });

        let mut services = FakeServices;
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "test",
            |cx| {
                let item_1 = AccordionItem::new(
                    Arc::from("item-1"),
                    AccordionTrigger::new(vec![cx.text("Item 1")])
                        .refine_layout(LayoutRefinement::default().h_px(Px(40.0))),
                    AccordionContent::new(vec![cx.text("Content 1")]),
                );
                let item_2 = AccordionItem::new(
                    Arc::from("item-2"),
                    AccordionTrigger::new(vec![cx.text("Item 2")])
                        .refine_layout(LayoutRefinement::default().h_px(Px(40.0))),
                    AccordionContent::new(vec![cx.text("Content 2")]),
                );

                vec![
                    Accordion::single(open.clone())
                        .on_value_change(Some(on_value_change.clone()))
                        .items([item_1, item_2])
                        .into_element(cx),
                ]
            },
        );

        ui.set_root(root);
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(60.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: Point::new(Px(10.0), Px(60.0)),
                button: fret_core::MouseButton::Left,
                modifiers: fret_core::Modifiers::default(),
                is_click: true,
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );

        render_accordion_frame_with_on_value_change(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            Some(on_value_change),
        );

        let selected = app.models().get_cloned(&open).flatten();
        assert_eq!(selected.as_deref(), Some("item-2"));

        let values = changed_values.lock().unwrap_or_else(|e| e.into_inner());
        assert_eq!(values.len(), 1);
        assert_eq!(values[0].len(), 1);
        assert_eq!(values[0][0].as_ref(), "item-2");
    }
}
