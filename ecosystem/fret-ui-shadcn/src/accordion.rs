use std::sync::Arc;
use std::time::Duration;

use fret_core::{
    AttributedText, Color, DecorationLineStyle, Edges, FontId, FontWeight, Point, Px, TextAlign,
    TextOverflow, TextPaintStyle, TextSpan, TextStyle, TextWrap, Transform2D, UnderlineStyle,
};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, ElementKind, HoverRegionProps,
    InteractivityGateProps, LayoutStyle, Length, MainAlign, OpacityProps, PressableProps,
    RovingFlexProps, RovingFocusProps, RowProps, StyledTextProps, VisualTransformProps,
};
use fret_ui::{ElementContext, Theme, ThemeSnapshot, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::motion::drive_tween_f32_for_element;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::declarative::transition::{
    drive_transition_with_durations_and_cubic_bezier, ticks_60hz_for_duration,
};
use fret_ui_kit::primitives::accordion as radix_accordion;
use fret_ui_kit::primitives::collapsible as radix_collapsible;
use fret_ui_kit::primitives::direction::LayoutDirection;
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Space, WidgetStates, ui,
};

use crate::overlay_motion;

fn border_color(theme: &ThemeSnapshot) -> Color {
    theme
        .color_by_key("border")
        .or_else(|| theme.color_by_key("input"))
        .expect("missing theme token: border/input")
}

fn trigger_text_style(theme: &ThemeSnapshot) -> TextStyle {
    let px = theme
        .metric_by_key("component.accordion.trigger.text_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_token("font.size"));
    let line_height = theme
        .metric_by_key("component.accordion.trigger.line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_token("font.line_height"));
    let mut style = typography::fixed_line_box_style(FontId::ui(), px, line_height);
    style.weight = FontWeight::MEDIUM;
    style
}

fn trigger_gap(theme: &ThemeSnapshot) -> Px {
    theme
        .metric_by_key("component.accordion.trigger.gap")
        .unwrap_or_else(|| MetricRef::space(Space::N4).resolve(theme))
}

fn tailwind_transition_ease_in_out(t: f32) -> f32 {
    // Tailwind default `ease-in-out`: cubic-bezier(0.4, 0, 0.2, 1)
    fret_ui_kit::headless::easing::CubicBezier::new(0.4, 0.0, 0.2, 1.0).sample(t)
}

fn apply_trigger_label_defaults(
    el: AnyElement,
    text_style: &TextStyle,
    align: TextAlign,
) -> AnyElement {
    match el.kind {
        ElementKind::Text(mut props) => {
            if props.style.is_none() {
                props.style = Some(text_style.clone());
            }
            props.layout.size.width = Length::Fill;
            props.layout.size.min_width = Some(Length::Px(Px(0.0)));
            props.wrap = TextWrap::Word;
            props.overflow = TextOverflow::Clip;
            props.align = align;
            AnyElement::new(el.id, ElementKind::Text(props), el.children)
        }
        ElementKind::StyledText(mut props) => {
            if props.style.is_none() {
                props.style = Some(text_style.clone());
            }
            props.layout.size.width = Length::Fill;
            props.layout.size.min_width = Some(Length::Px(Px(0.0)));
            props.wrap = TextWrap::Word;
            props.overflow = TextOverflow::Clip;
            props.align = align;
            AnyElement::new(el.id, ElementKind::StyledText(props), el.children)
        }
        ElementKind::SelectableText(mut props) => {
            if props.style.is_none() {
                props.style = Some(text_style.clone());
            }
            props.layout.size.width = Length::Fill;
            props.layout.size.min_width = Some(Length::Px(Px(0.0)));
            props.wrap = TextWrap::Word;
            props.overflow = TextOverflow::Clip;
            props.align = align;
            AnyElement::new(el.id, ElementKind::SelectableText(props), el.children)
        }
        _ => el,
    }
}

fn underline_rich_text(rich: AttributedText) -> AttributedText {
    let spans = rich
        .spans
        .iter()
        .map(|span| {
            let mut span = span.clone();
            if span.paint.underline.is_none() {
                span.paint.underline = Some(UnderlineStyle {
                    color: None,
                    style: DecorationLineStyle::Solid,
                });
            }
            span
        })
        .collect::<Vec<_>>();
    AttributedText::new(rich.text, Arc::from(spans.into_boxed_slice()))
}

fn apply_trigger_label_hover_underline(el: AnyElement) -> AnyElement {
    match el.kind {
        ElementKind::Text(props) => {
            let text = props.text.clone();
            let mut span = TextSpan::new(text.as_ref().len());
            span.paint = TextPaintStyle {
                underline: Some(UnderlineStyle {
                    color: None,
                    style: DecorationLineStyle::Solid,
                }),
                ..Default::default()
            };
            let rich = AttributedText::new(text, Arc::from(vec![span].into_boxed_slice()));

            let mut styled = StyledTextProps::new(rich);
            styled.layout = props.layout;
            styled.style = props.style;
            styled.color = props.color;
            styled.wrap = props.wrap;
            styled.overflow = props.overflow;
            styled.align = props.align;
            styled.ink_overflow = props.ink_overflow;
            AnyElement::new(el.id, ElementKind::StyledText(styled), el.children)
        }
        ElementKind::StyledText(mut props) => {
            props.rich = underline_rich_text(props.rich);
            AnyElement::new(el.id, ElementKind::StyledText(props), el.children)
        }
        ElementKind::SelectableText(mut props) => {
            props.rich = underline_rich_text(props.rich);
            AnyElement::new(el.id, ElementKind::SelectableText(props), el.children)
        }
        _ => el,
    }
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

    pub struct AccordionTrigger {
        disabled: bool,
        a11y_label: Option<Arc<str>>,
        test_id: Option<Arc<str>>,
        chrome: ChromeRefinement,
        layout: LayoutRefinement,
        children: Vec<AnyElement>,
    }

    impl std::fmt::Debug for AccordionTrigger {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("AccordionTrigger")
                .field("disabled", &self.disabled)
                .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
                .field("test_id", &self.test_id.as_ref().map(|s| s.as_ref()))
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
                test_id: None,
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

        pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
            self.test_id = Some(id.into());
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
            dir: LayoutDirection,
        ) -> AnyElement {
            let theme = Theme::global(&*cx.app).snapshot();

            let a11y_label = self.a11y_label.unwrap_or_else(|| value.clone());
            let test_id = self.test_id;
            let chevron_test_id: Option<Arc<str>> = test_id
                .as_ref()
                .map(|id| Arc::from(format!("{id}-chevron")));
            let label_test_id: Option<Arc<str>> =
                test_id.as_ref().map(|id| Arc::from(format!("{id}-label")));
            let text_style = trigger_text_style(&theme);
            let radius = MetricRef::radius(Radius::Md).resolve(&theme);
            let label_align = if dir == LayoutDirection::Rtl {
                TextAlign::End
            } else {
                TextAlign::Start
            };
            let is_rtl = dir == LayoutDirection::Rtl;
            let chevron_motion_key = value.clone();

            let pressable_layout = decl_style::layout_style(
                &theme,
                self.layout
                    .merge(LayoutRefinement::default().w_full().min_w_0()),
            );
            let container_layout = pressable_layout;

            let chrome = self.chrome;
            let children = self.children;

            let trigger = radix_accordion::AccordionTrigger::new(value.clone())
                .label(a11y_label.clone())
                .disabled(!enabled)
                .tab_stop(focusable)
                .into_element_with_id_props(
                    cx,
                    root,
                    PressableProps {
                        layout: pressable_layout,
                        ..Default::default()
                    },
                    move |cx, is_open, st, id, mut pressable_props| {
                        let states = WidgetStates::from_pressable(cx, st, enabled);
                        let duration = overlay_motion::shadcn_motion_duration_150(cx);
                        let ring_alpha = drive_tween_f32_for_element(
                            cx,
                            id,
                            "accordion.trigger.ring.alpha",
                            if states.contains(WidgetStates::FOCUS_VISIBLE) {
                                1.0
                            } else {
                                0.0
                            },
                            duration,
                            tailwind_transition_ease_in_out,
                        );

                        let mut focus_ring = decl_style::focus_ring(&theme, radius);
                        focus_ring.color.a =
                            (focus_ring.color.a * ring_alpha.value).clamp(0.0, 1.0);
                        if let Some(offset_color) = focus_ring.offset_color {
                            focus_ring.offset_color = Some(Color {
                                a: (offset_color.a * ring_alpha.value).clamp(0.0, 1.0),
                                ..offset_color
                            });
                        }
                        pressable_props.focus_ring = Some(focus_ring);
                        pressable_props.focus_ring_always_paint = ring_alpha.animating;

                        let chrome = ChromeRefinement::default()
                            .px(Space::N0)
                            .py(Space::N4)
                            .rounded(Radius::Md)
                            .merge(chrome.clone());
                        let mut props =
                            decl_style::container_props(&theme, chrome, Default::default());
                        props.layout.size = container_layout.size;
                        props.layout.overflow = container_layout.overflow;

                        let children = vec![cx.container(
                            ContainerProps {
                                layout: props.layout,
                                padding: props.padding.into(),
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
                                    .unwrap_or_else(|| theme.color_token("muted-foreground"));
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
                                let chevron_motion = cx.keyed(
                                    ("accordion-chevron-motion", chevron_motion_key.clone()),
                                    |cx| {
                                        let theme_full = Theme::global(&*cx.app);
                                        let duration = theme_full
                                            .duration_ms_by_key(
                                                "duration.shadcn.motion.collapsible.toggle",
                                            )
                                            .or_else(|| {
                                                theme_full.duration_ms_by_key(
                                                    "duration.motion.collapsible.toggle",
                                                )
                                            })
                                            .or_else(|| {
                                                theme_full.duration_ms_by_key(
                                                    "duration.shadcn.motion.200",
                                                )
                                            })
                                            .map(|ms| Duration::from_millis(ms as u64))
                                            .unwrap_or(Duration::from_millis(200));
                                        let ticks = ticks_60hz_for_duration(duration);
                                        let easing = theme_full
                                            .easing_by_key(
                                                "easing.shadcn.motion.collapsible.toggle",
                                            )
                                            .or_else(|| {
                                                theme_full.easing_by_key(
                                                    "easing.motion.collapsible.toggle",
                                                )
                                            })
                                            .or_else(|| {
                                                theme_full.easing_by_key("easing.shadcn.motion")
                                            })
                                            .or_else(|| {
                                                theme_full.easing_by_key("easing.motion.standard")
                                            })
                                            .unwrap_or_else(|| {
                                                overlay_motion::shadcn_motion_ease_bezier(cx)
                                            });

                                        drive_transition_with_durations_and_cubic_bezier(
                                            cx, is_open, ticks, ticks, easing,
                                        )
                                    },
                                );
                                let chevron_rotation = 180.0 * chevron_motion.progress;
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
                                let chevron = if let Some(id) = chevron_test_id.clone() {
                                    chevron.test_id(id)
                                } else {
                                    chevron
                                };

                                let left_layout = decl_style::layout_style(
                                    &theme,
                                    LayoutRefinement::default().flex_1().min_w_0(),
                                );
                                let mut row_layout = LayoutStyle::default();
                                row_layout.size.width = Length::Fill;
                                row_layout.size.min_width = Some(Length::Px(Px(0.0)));
                                let hover_layout = row_layout;
                                vec![cx.hover_region(
                                    HoverRegionProps { layout: hover_layout },
                                    move |cx, hovered| {
                                        let hover_underline = hovered && enabled;
                                        vec![cx.row(
                                            RowProps {
                                                layout: row_layout,
                                                gap: trigger_gap(&theme).into(),
                                                padding: Edges::all(Px(0.0)).into(),
                                                justify: MainAlign::Start,
                                                align: CrossAlign::Start,
                                            },
                                            move |cx| {
                                                let left_children = if children.is_empty() {
                                                    let mut label_text =
                                                        ui::text( a11y_label.clone())
                                                            .text_size_px(text_style.size)
                                                            .font_weight(text_style.weight)
                                                            .wrap(TextWrap::Word)
                                                            .overflow(TextOverflow::Clip);
                                                    if let Some(line_height) = text_style.line_height {
                                                        label_text = label_text
                                                            .line_height_px(line_height)
                                                            .line_height_policy(
                                                                fret_core::TextLineHeightPolicy::FixedFromStyle,
                                                            );
                                                    }
                                                    if let Some(letter_spacing_em) =
                                                        text_style.letter_spacing_em
                                                    {
                                                        label_text = label_text
                                                            .letter_spacing_em(letter_spacing_em);
                                                    }
                                                    vec![label_text.into_element(cx)]
                                                } else {
                                                    children
                                                };
                                                let left_children = left_children
                                                    .into_iter()
                                                    .map(|el| {
                                                        let el = apply_trigger_label_defaults(
                                                            el,
                                                            &text_style,
                                                            label_align,
                                                        );
                                                        if hover_underline {
                                                            apply_trigger_label_hover_underline(el)
                                                        } else {
                                                            el
                                                        }
                                                    })
                                                    .collect::<Vec<_>>();

                                                let label = cx.container(
                                                    ContainerProps {
                                                        layout: left_layout,
                                                        ..Default::default()
                                                    },
                                                    |_cx| left_children,
                                                );
                                                let label = if let Some(id) = label_test_id.clone()
                                                {
                                                    label.test_id(id)
                                                } else {
                                                    label
                                                };

                                                if is_rtl {
                                                    vec![chevron, label]
                                                } else {
                                                    vec![label, chevron]
                                                }
                                            },
                                        )]
                                    },
                                )]
                            },
                        )];

                        (pressable_props, children)
                    },
                );

            let trigger = if let Some(test_id) = test_id {
                trigger.test_id(test_id)
            } else {
                trigger
            };

            if enabled {
                trigger
            } else {
                cx.opacity(0.5, move |_cx| [trigger])
            }
        }
    }

    pub struct AccordionContent {
        test_id: Option<Arc<str>>,
        chrome: ChromeRefinement,
        layout: LayoutRefinement,
        gap: Option<MetricRef>,
        children: Vec<AnyElement>,
    }

    impl std::fmt::Debug for AccordionContent {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("AccordionContent")
                .field("test_id", &self.test_id.as_ref().map(|s| s.as_ref()))
                .field("chrome", &self.chrome)
                .field("layout", &self.layout)
                .field("gap", &self.gap)
                .field("children_len", &self.children.len())
                .finish()
        }
    }

    impl AccordionContent {
        pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
            let children = children.into_iter().collect();
            Self {
                test_id: None,
                chrome: ChromeRefinement::default(),
                layout: LayoutRefinement::default(),
                gap: None,
                children,
            }
        }

        pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
            self.test_id = Some(id.into());
            self
        }

        /// Controls the vertical spacing between direct children (shadcn `gap-*`).
        pub fn gap(mut self, gap: impl Into<MetricRef>) -> Self {
            self.gap = Some(gap.into());
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

        fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
            let theme = Theme::global(&*cx.app).snapshot();
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

            let gap = self
                .gap
                .unwrap_or_else(|| MetricRef::Px(Px(0.0)))
                .resolve(&theme);
            let children = self.children;

            cx.container(props, move |cx| {
                let mut column_layout = LayoutStyle::default();
                column_layout.size.width = Length::Fill;
                column_layout.size.min_width = Some(Length::Px(Px(0.0)));
                vec![cx.column(
                    ColumnProps {
                        layout: column_layout,
                        gap: gap.into(),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Start,
                    },
                    move |_cx| children,
                )]
            })
        }
    }

    pub struct AccordionItem {
        value: Arc<str>,
        trigger: Option<AccordionTrigger>,
        content: Option<AccordionContent>,
        test_id: Option<Arc<str>>,
        disabled: bool,
        layout: LayoutRefinement,
        chrome: ChromeRefinement,
    }

    impl std::fmt::Debug for AccordionItem {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("AccordionItem")
                .field("value", &self.value.as_ref())
                .field("test_id", &self.test_id.as_ref().map(|s| s.as_ref()))
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
                test_id: None,
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

        pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
            self.test_id = Some(id.into());
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

        #[track_caller]
        pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
            cx.scope(|cx| {
                let theme = Theme::global(&*cx.app).snapshot();

                let model = self.model;
                let items = self.items;
                let group_disabled = self.disabled;
                let layout = self.layout;
                let loop_navigation = self.loop_navigation;
                let orientation = self.orientation;
                let dir = fret_ui_kit::primitives::direction::use_direction_in_scope(cx, self.dir);
                let on_value_change = self.on_value_change;
                let single_non_collapsible =
                    matches!(&model, AccordionModel::Single { collapsible: false, .. });

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
                .dir(Some(dir));

                let values: Vec<Arc<str>> = items.iter().map(|i| i.value.clone()).collect();
                if let Some(on_value_change) = on_value_change.as_ref() {
                    let next = values
                        .iter()
                        .filter(|value| root.is_item_open(cx, value.as_ref()))
                        .cloned()
                        .collect::<Vec<_>>();
                    let changed =
                        cx.slot_state(AccordionValueChangeCallbackState::default, |state| {
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
                        gap: Px(0.0).into(),
                        padding: Edges::all(Px(0.0)).into(),
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
                                let aria_disabled = single_non_collapsible && is_open;

                                let trigger = trigger.into_element(
                                    cx,
                                    &root,
                                    item.value.clone(),
                                    enabled,
                                    focusable,
                                    dir,
                                );
                                let trigger_element = trigger.id;

                                let theme = theme.clone();
                                let value = item.value.clone();
                                let content_test_id = content.test_id.clone();
                                let item_test_id = item.test_id.clone();

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

                                let item_el = cx.container(props, move |cx| {
                                    let mut children = Vec::new();

                                    let motion =
                                        cx.keyed(("accordion-motion", value.clone()), |cx| {
                                            let theme_full = Theme::global(&*cx.app);
                                            let toggle_duration = theme_full
                                                .duration_ms_by_key(
                                                    "duration.shadcn.motion.collapsible.toggle",
                                                )
                                                .or_else(|| {
                                                    theme_full.duration_ms_by_key(
                                                        "duration.motion.collapsible.toggle",
                                                    )
                                                })
                                                .or_else(|| {
                                                    theme_full.duration_ms_by_key(
                                                        "duration.shadcn.motion.200",
                                                    )
                                                })
                                                .map(|ms| Duration::from_millis(ms as u64))
                                                .unwrap_or(Duration::from_millis(200));
                                            let toggle_ticks =
                                                ticks_60hz_for_duration(toggle_duration);
                                            let toggle_easing = theme_full
                                                .easing_by_key(
                                                    "easing.shadcn.motion.collapsible.toggle",
                                                )
                                                .or_else(|| {
                                                    theme_full.easing_by_key(
                                                        "easing.motion.collapsible.toggle",
                                                    )
                                                })
                                                .unwrap_or_else(|| {
                                                    overlay_motion::shadcn_motion_ease_bezier(cx)
                                                });
                                            radix_collapsible::measured_height_motion_for_root_with_cubic_bezier(
                                                cx,
                                                is_open,
                                                false,
                                                true,
                                                toggle_ticks,
                                                toggle_ticks,
                                                toggle_easing,
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
                                                move |cx| vec![content.into_element(cx)],
                                            )];

                                            let wrapper_kind =
                                                if motion_for_wrapper.wants_measurement {
                                                    ElementKind::InteractivityGate(
                                                        InteractivityGateProps {
                                                            layout: wrapper_layout,
                                                            present: true,
                                                            interactive: false,
                                                        },
                                                    )
                                                } else {
                                                    ElementKind::Container(ContainerProps {
                                                        layout: wrapper_layout,
                                                        ..Default::default()
                                                    })
                                                };

                                            let wrapper_el =
                                                AnyElement::new(content_id, wrapper_kind, children);

                                            let wrapper_el =
                                                if let Some(test_id) = content_test_id.clone() {
                                                    wrapper_el.test_id(test_id)
                                                } else {
                                                    wrapper_el
                                                };

                                            (content_id, Some(wrapper_el))
                                        });

                                    let trigger = radix_accordion::apply_accordion_trigger_controls(
                                        trigger, content_id,
                                    );
                                    let trigger = radix_accordion::apply_accordion_trigger_aria_disabled(
                                        trigger,
                                        aria_disabled,
                                    );
                                    children.push(trigger);

                                    if let Some(wrapper_el) = wrapper_el {
                                        let wrapper_el =
                                            radix_accordion::apply_accordion_content_region_labelled_by(
                                                wrapper_el,
                                                trigger_element,
                                            );
                                        let _ = radix_collapsible::update_measured_for_motion(
                                            cx,
                                            motion_for_update,
                                            wrapper_el.id,
                                        );
                                        children.push(wrapper_el);
                                    }

                                    children
                                });

                                out.push(if let Some(test_id) = item_test_id {
                                    item_el.test_id(test_id)
                                } else {
                                    item_el
                                });
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

pub struct AccordionTrigger {
    disabled: bool,
    a11y_label: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl std::fmt::Debug for AccordionTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccordionTrigger")
            .field("disabled", &self.disabled)
            .field("a11y_label", &self.a11y_label.as_ref().map(|s| s.as_ref()))
            .field("test_id", &self.test_id.as_ref().map(|s| s.as_ref()))
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
            test_id: None,
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

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
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
        dir: LayoutDirection,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();

        let a11y_label = self.a11y_label.unwrap_or_else(|| value.clone());
        let test_id = self.test_id;
        let chevron_test_id: Option<Arc<str>> = test_id
            .as_ref()
            .map(|id| Arc::from(format!("{id}-chevron")));
        let label_test_id: Option<Arc<str>> =
            test_id.as_ref().map(|id| Arc::from(format!("{id}-label")));
        let text_style = trigger_text_style(&theme);
        let radius = MetricRef::radius(Radius::Md).resolve(&theme);
        let label_align = if dir == LayoutDirection::Rtl {
            TextAlign::End
        } else {
            TextAlign::Start
        };
        let is_rtl = dir == LayoutDirection::Rtl;
        let chevron_motion_key = value.clone();

        let pressable_layout = decl_style::layout_style(
            &theme,
            self.layout
                .merge(LayoutRefinement::default().w_full().min_w_0()),
        );
        let container_layout = pressable_layout;

        let chrome = self.chrome;
        let children = self.children;

        let trigger = radix_accordion::AccordionTrigger::new(value.clone())
            .label(a11y_label.clone())
            .disabled(!enabled)
            .tab_stop(focusable)
            .into_element_with_id_props(
                cx,
                root,
                PressableProps {
                    layout: pressable_layout,
                    ..Default::default()
                },
                move |cx, is_open, st, id, mut pressable_props| {
                    let states = WidgetStates::from_pressable(cx, st, enabled);
                    let duration = overlay_motion::shadcn_motion_duration_150(cx);
                    let ring_alpha = drive_tween_f32_for_element(
                        cx,
                        id,
                        "accordion.trigger.ring.alpha",
                        if states.contains(WidgetStates::FOCUS_VISIBLE) {
                            1.0
                        } else {
                            0.0
                        },
                        duration,
                        tailwind_transition_ease_in_out,
                    );

                    let mut focus_ring = decl_style::focus_ring(&theme, radius);
                    focus_ring.color.a = (focus_ring.color.a * ring_alpha.value).clamp(0.0, 1.0);
                    if let Some(offset_color) = focus_ring.offset_color {
                        focus_ring.offset_color = Some(Color {
                            a: (offset_color.a * ring_alpha.value).clamp(0.0, 1.0),
                            ..offset_color
                        });
                    }
                    pressable_props.focus_ring = Some(focus_ring);
                    pressable_props.focus_ring_always_paint = ring_alpha.animating;

                    let chrome = ChromeRefinement::default()
                        .px(Space::N0)
                        .py(Space::N4)
                        .rounded(Radius::Md)
                        .merge(chrome.clone());
                    let mut props = decl_style::container_props(&theme, chrome, Default::default());
                    props.layout.size = container_layout.size;
                    props.layout.overflow = container_layout.overflow;

                    let children = vec![cx.container(
                        ContainerProps {
                            layout: props.layout,
                            padding: props.padding.into(),
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
                                .unwrap_or_else(|| theme.color_token("muted-foreground"));
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
                            let chevron_motion = cx.keyed(
                                ("accordion-chevron-motion", chevron_motion_key.clone()),
                                |cx| {
                                    let theme_full = Theme::global(&*cx.app);
                                    let duration = theme_full
                                        .duration_ms_by_key(
                                            "duration.shadcn.motion.collapsible.toggle",
                                        )
                                        .or_else(|| {
                                            theme_full.duration_ms_by_key(
                                                "duration.motion.collapsible.toggle",
                                            )
                                        })
                                        .or_else(|| {
                                            theme_full.duration_ms_by_key(
                                                "duration.shadcn.motion.200",
                                            )
                                        })
                                        .map(|ms| Duration::from_millis(ms as u64))
                                        .unwrap_or(Duration::from_millis(200));
                                    let ticks = ticks_60hz_for_duration(duration);
                                    let easing = theme_full
                                        .easing_by_key(
                                            "easing.shadcn.motion.collapsible.toggle",
                                        )
                                        .or_else(|| {
                                            theme_full.easing_by_key(
                                                "easing.motion.collapsible.toggle",
                                            )
                                        })
                                        .or_else(|| theme_full.easing_by_key("easing.shadcn.motion"))
                                        .or_else(|| theme_full.easing_by_key("easing.motion.standard"))
                                        .unwrap_or_else(|| {
                                            overlay_motion::shadcn_motion_ease_bezier(cx)
                                        });

                                    drive_transition_with_durations_and_cubic_bezier(
                                        cx, is_open, ticks, ticks, easing,
                                    )
                                },
                            );
                            let chevron_rotation = 180.0 * chevron_motion.progress;
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
                            let chevron = if let Some(id) = chevron_test_id.clone() {
                                chevron.test_id(id)
                            } else {
                                chevron
                            };

                            let left_layout = decl_style::layout_style(
                                &theme,
                                LayoutRefinement::default().flex_1().min_w_0(),
                            );
                            let mut row_layout = LayoutStyle::default();
                            row_layout.size.width = Length::Fill;
                            row_layout.size.min_width = Some(Length::Px(Px(0.0)));
                            let hover_layout = row_layout;
                            vec![cx.hover_region(
                                HoverRegionProps { layout: hover_layout },
                                move |cx, hovered| {
                                    let hover_underline = hovered && enabled;
                                    vec![cx.row(
                                        RowProps {
                                            layout: row_layout,
                                            gap: trigger_gap(&theme).into(),
                                            padding: Edges::all(Px(0.0)).into(),
                                            justify: MainAlign::Start,
                                            align: CrossAlign::Start,
                                        },
                                        move |cx| {
                                            let left_children = if children.is_empty() {
                                                let mut label_text =
                                                    ui::text( a11y_label.clone())
                                                        .text_size_px(text_style.size)
                                                        .font_weight(text_style.weight)
                                                        .wrap(TextWrap::Word)
                                                        .overflow(TextOverflow::Clip);
                                                if let Some(line_height) = text_style.line_height {
                                                    label_text = label_text
                                                        .line_height_px(line_height)
                                                        .line_height_policy(
                                                            fret_core::TextLineHeightPolicy::FixedFromStyle,
                                                        );
                                                }
                                                if let Some(letter_spacing_em) =
                                                    text_style.letter_spacing_em
                                                {
                                                    label_text = label_text
                                                        .letter_spacing_em(letter_spacing_em);
                                                }
                                                vec![label_text.into_element(cx)]
                                            } else {
                                                children
                                            };
                                            let left_children = left_children
                                                .into_iter()
                                                .map(|el| {
                                                    let el = apply_trigger_label_defaults(
                                                        el,
                                                        &text_style,
                                                        label_align,
                                                    );
                                                    if hover_underline {
                                                        apply_trigger_label_hover_underline(el)
                                                    } else {
                                                        el
                                                    }
                                                })
                                                .collect::<Vec<_>>();

                                            let label = cx.container(
                                                ContainerProps {
                                                    layout: left_layout,
                                                    ..Default::default()
                                                },
                                                |_cx| left_children,
                                            );
                                            let label = if let Some(id) = label_test_id.clone() {
                                                label.test_id(id)
                                            } else {
                                                label
                                            };

                                            if is_rtl {
                                                vec![chevron, label]
                                            } else {
                                                vec![label, chevron]
                                            }
                                        },
                                    )]
                                },
                            )]
                        },
                    )];

                    (pressable_props, children)
                },
            );

        let trigger = if let Some(test_id) = test_id {
            trigger.test_id(test_id)
        } else {
            trigger
        };

        if enabled {
            trigger
        } else {
            cx.opacity(0.5, move |_cx| [trigger])
        }
    }
}

pub struct AccordionContent {
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    gap: Option<MetricRef>,
    children: Vec<AnyElement>,
}

impl std::fmt::Debug for AccordionContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccordionContent")
            .field("test_id", &self.test_id.as_ref().map(|s| s.as_ref()))
            .field("chrome", &self.chrome)
            .field("layout", &self.layout)
            .field("gap", &self.gap)
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl AccordionContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            gap: None,
            children,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    /// Controls the vertical spacing between direct children (shadcn `gap-*`).
    pub fn gap(mut self, gap: impl Into<MetricRef>) -> Self {
        self.gap = Some(gap.into());
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

    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
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

        let gap = self
            .gap
            .unwrap_or_else(|| MetricRef::Px(Px(0.0)))
            .resolve(&theme);
        let children = self.children;

        cx.container(props, move |cx| {
            let mut column_layout = LayoutStyle::default();
            column_layout.size.width = Length::Fill;
            column_layout.size.min_width = Some(Length::Px(Px(0.0)));
            vec![cx.column(
                ColumnProps {
                    layout: column_layout,
                    gap: gap.into(),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Start,
                    align: CrossAlign::Start,
                },
                move |_cx| children,
            )]
        })
    }
}

pub struct AccordionItem {
    value: Arc<str>,
    trigger: AccordionTrigger,
    content: AccordionContent,
    test_id: Option<Arc<str>>,
    disabled: bool,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for AccordionItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccordionItem")
            .field("value", &self.value.as_ref())
            .field("test_id", &self.test_id.as_ref().map(|s| s.as_ref()))
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
            test_id: None,
            disabled: false,
            layout: LayoutRefinement::default(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
        let theme = Theme::global(&*cx.app).snapshot();

        let model = self.model;
        let items = self.items;
        let group_disabled = self.disabled;
        let layout = self.layout;
            let loop_navigation = self.loop_navigation;
            let orientation = self.orientation;
            let dir = fret_ui_kit::primitives::direction::use_direction_in_scope(cx, self.dir);
            let on_value_change = self.on_value_change;
            let single_non_collapsible =
                matches!(&model, AccordionModel::Single { collapsible: false, .. });

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
            .dir(Some(dir));

            let values: Vec<Arc<str>> = items.iter().map(|i| i.value.clone()).collect();
            if let Some(on_value_change) = on_value_change.as_ref() {
                let next = values
                    .iter()
                    .filter(|value| root.is_item_open(cx, value.as_ref()))
                    .cloned()
                    .collect::<Vec<_>>();
                let changed = cx.slot_state(AccordionValueChangeCallbackState::default, |state| {
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
                            gap: Px(0.0).into(),
                            padding: Edges::all(Px(0.0)).into(),
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
                            let aria_disabled = single_non_collapsible && is_open;

                            let trigger = item.trigger.into_element(
                                cx,
                                &root,
                                item.value.clone(),
                                enabled,
                                focusable,
                                dir,
                            );
                            let trigger_element = trigger.id;

                            let content = item.content;
                            let theme = theme.clone();
                            let value = item.value.clone();
                            let content_test_id = content.test_id.clone();
                            let item_test_id = item.test_id.clone();

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

                                let item_el = cx.container(props, move |cx| {
                                    let mut children = Vec::new();

                                    let motion = cx.keyed(("accordion-motion", value.clone()), |cx| {
                                        let theme_full = Theme::global(&*cx.app);
                                        let toggle_duration = theme_full
                                            .duration_ms_by_key(
                                                "duration.shadcn.motion.collapsible.toggle",
                                            )
                                            .or_else(|| {
                                                theme_full.duration_ms_by_key(
                                                    "duration.motion.collapsible.toggle",
                                                )
                                            })
                                            .or_else(|| {
                                                theme_full.duration_ms_by_key("duration.shadcn.motion.200")
                                            })
                                            .map(|ms| Duration::from_millis(ms as u64))
                                            .unwrap_or(Duration::from_millis(200));
                                        let toggle_ticks = ticks_60hz_for_duration(toggle_duration);
                                        let toggle_easing = theme_full
                                            .easing_by_key(
                                                "easing.shadcn.motion.collapsible.toggle",
                                            )
                                            .or_else(|| {
                                                theme_full.easing_by_key(
                                                    "easing.motion.collapsible.toggle",
                                                )
                                            })
                                            .unwrap_or_else(|| {
                                                overlay_motion::shadcn_motion_ease_bezier(cx)
                                            });
                                        radix_collapsible::measured_height_motion_for_root_with_cubic_bezier(
                                            cx,
                                            is_open,
                                            false,
                                            true,
                                            toggle_ticks,
                                            toggle_ticks,
                                            toggle_easing,
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
                                            move |cx| vec![content.into_element(cx)],
                                        )];

                                        let wrapper_kind = if motion_for_wrapper.wants_measurement {
                                            ElementKind::InteractivityGate(InteractivityGateProps {
                                                layout: wrapper_layout,
                                                present: true,
                                                interactive: false,
                                            })
                                        } else {
                                            ElementKind::Container(ContainerProps {
                                                layout: wrapper_layout,
                                                ..Default::default()
                                            })
                                        };

                                        let wrapper_el =
                                            AnyElement::new(content_id, wrapper_kind, children);

                                        let wrapper_el =
                                            if let Some(test_id) = content_test_id.clone() {
                                                wrapper_el.test_id(test_id)
                                            } else {
                                                wrapper_el
                                            };

                                        (content_id, Some(wrapper_el))
                                    });

                                let trigger = radix_accordion::apply_accordion_trigger_controls(
                                    trigger, content_id,
                                );
                                let trigger = radix_accordion::apply_accordion_trigger_aria_disabled(
                                    trigger,
                                    aria_disabled,
                                );
                                children.push(trigger);

                                if let Some(wrapper_el) = wrapper_el {
                                    let wrapper_el =
                                        radix_accordion::apply_accordion_content_region_labelled_by(
                                            wrapper_el,
                                            trigger_element,
                                        );
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
                                        gap: Px(0.0).into(),
                                        padding: Edges::all(Px(0.0)).into(),
                                        justify: MainAlign::Start,
                                        align: CrossAlign::Stretch,
                                    },
                                    move |_cx| children,
                                )]
                            });

                            out.push(if let Some(test_id) = item_test_id {
                                item_el.test_id(test_id)
                            } else {
                                item_el
                            });
                        }

                        out
                    },
                )]
            })
        })
    }
}

/// Builder-preserving controlled helper for the common accordion authoring path.
pub fn accordion_single<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Option<Arc<str>>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> Accordion
where
    I: IntoIterator<Item = AccordionItem>,
{
    Accordion::single(model).items(f(cx))
}

/// Builder-preserving uncontrolled helper for the common `defaultValue` accordion path.
pub fn accordion_single_uncontrolled<H: UiHost, T: Into<Arc<str>>, I>(
    cx: &mut ElementContext<'_, H>,
    default_value: Option<T>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> Accordion
where
    I: IntoIterator<Item = AccordionItem>,
{
    Accordion::single_uncontrolled(default_value).items(f(cx))
}

/// Builder-preserving controlled helper for the common multi-open accordion path.
pub fn accordion_multiple<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    model: Model<Vec<Arc<str>>>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> Accordion
where
    I: IntoIterator<Item = AccordionItem>,
{
    Accordion::multiple(model).items(f(cx))
}

/// Builder-preserving uncontrolled helper for the common multi-open `defaultValue` path.
pub fn accordion_multiple_uncontrolled<H: UiHost, V, I>(
    cx: &mut ElementContext<'_, H>,
    default_value: V,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> Accordion
where
    V: IntoIterator,
    V::Item: Into<Arc<str>>,
    I: IntoIterator<Item = AccordionItem>,
{
    Accordion::multiple_uncontrolled(default_value).items(f(cx))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::Mutex;

    use fret_app::App;
    use fret_core::{AppWindowId, NodeId, PathCommand, Point, Rect, Size, SvgId, SvgService};
    use fret_core::{PathConstraints, PathId, PathMetrics, PathService, PathStyle};
    use fret_core::{Px, TextAlign, TextBlobId, TextConstraints, TextMetrics, TextService};
    use fret_runtime::{FrameId, TickId};
    use fret_ui::Theme;
    use fret_ui::UiTree;
    use fret_ui::element::{CrossAlign, ElementKind, SpacingLength, TextProps};
    use fret_ui_kit::LayoutRefinement;

    use super::{
        Accordion, AccordionContent, AccordionItem, AccordionTrigger, apply_trigger_label_defaults,
        composable as composable_accordion, trigger_text_style,
    };

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(120.0)),
        )
    }

    #[test]
    fn accordion_trigger_label_defaults_do_not_force_foreground_color() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let theme = Theme::global(&*cx.app).snapshot();
            let text_style = trigger_text_style(&theme);

            let mut text = TextProps::new("hello");
            text.color = None;
            let el = cx.text_props(text);

            let out = apply_trigger_label_defaults(el, &text_style, TextAlign::Start);
            let ElementKind::Text(props) = &out.kind else {
                panic!("expected Text element");
            };
            assert!(
                props.color.is_none(),
                "expected AccordionTrigger label to inherit foreground color from scope (e.g. Card foreground)"
            );
        });
    }

    #[test]
    fn accordion_trigger_focus_ring_alpha_tweens_in_and_out_like_transition_all() {
        use std::cell::Cell;
        use std::rc::Rc;
        use std::time::Duration;

        use fret_runtime::FrameId;
        use fret_ui::element::AnyElement;
        use fret_ui::elements;
        use fret_ui::elements::GlobalElementId;
        use fret_ui_kit::declarative::transition::ticks_60hz_for_duration;

        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        crate::shadcn_themes::apply_shadcn_new_york(
            &mut app,
            crate::shadcn_themes::ShadcnBaseColor::Neutral,
            crate::shadcn_themes::ShadcnColorScheme::Light,
        );

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(520.0), Px(280.0)),
        );
        let mut services = FakeServices;

        let theme = Theme::global(&app).snapshot();
        let focus_radius = fret_ui_kit::MetricRef::radius(fret_ui_kit::Radius::Md).resolve(&theme);
        let base_alpha = fret_ui_kit::declarative::style::focus_ring(&theme, focus_radius)
            .color
            .a;

        let open = app.models_mut().insert::<Option<Arc<str>>>(None);

        let trigger_id_out: Rc<Cell<Option<GlobalElementId>>> = Rc::new(Cell::new(None));
        let ring_alpha_out: Rc<Cell<Option<f32>>> = Rc::new(Cell::new(None));
        let always_paint_out: Rc<Cell<Option<bool>>> = Rc::new(Cell::new(None));

        fn find_element_by_test_id<'a>(
            el: &'a AnyElement,
            test_id: &str,
        ) -> Option<&'a AnyElement> {
            if el
                .semantics_decoration
                .as_ref()
                .and_then(|s| s.test_id.as_deref())
                == Some(test_id)
            {
                return Some(el);
            }
            for child in &el.children {
                if let Some(found) = find_element_by_test_id(child, test_id) {
                    return Some(found);
                }
            }
            None
        }

        fn render_frame(
            ui: &mut UiTree<App>,
            app: &mut App,
            services: &mut dyn fret_core::UiServices,
            window: AppWindowId,
            bounds: Rect,
            open: fret_runtime::Model<Option<Arc<str>>>,
            trigger_id_out: Rc<Cell<Option<GlobalElementId>>>,
            ring_alpha_out: Rc<Cell<Option<f32>>>,
            always_paint_out: Rc<Cell<Option<bool>>>,
        ) {
            let root = fret_ui::declarative::render_root(
                ui,
                app,
                services,
                window,
                bounds,
                "accordion-trigger-focus-ring-transition",
                move |cx| {
                    let item = AccordionItem::new(
                        Arc::from("item-1"),
                        AccordionTrigger::new(vec![cx.text("Item 1")]).test_id("accordion-trigger"),
                        AccordionContent::new(vec![cx.text("Content 1")]),
                    );

                    let accordion = Accordion::single(open.clone())
                        .collapsible(true)
                        .items([item])
                        .into_element(cx);

                    let trigger = find_element_by_test_id(&accordion, "accordion-trigger")
                        .expect("missing trigger test_id");
                    trigger_id_out.set(Some(trigger.id));

                    let ElementKind::Pressable(props) = &trigger.kind else {
                        panic!("expected trigger to be a Pressable");
                    };
                    let alpha = props
                        .focus_ring
                        .as_ref()
                        .map(|ring| ring.color.a)
                        .unwrap_or(0.0);
                    ring_alpha_out.set(Some(alpha));
                    always_paint_out.set(Some(props.focus_ring_always_paint));

                    vec![accordion]
                },
            );
            ui.set_root(root);
        }

        // Frame 1: unfocused, ring should be fully hidden.
        app.set_frame_id(FrameId(1));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            trigger_id_out.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let trigger_id = trigger_id_out.get().expect("trigger element id");
        let trigger_node =
            elements::node_for_element(&mut app, window, trigger_id).expect("trigger node");

        assert!(
            ring_alpha_out.get().expect("alpha").abs() <= 1e-6,
            "expected initial ring alpha to be 0; got {:?}",
            ring_alpha_out.get()
        );
        assert_eq!(
            always_paint_out.get().expect("always paint"),
            false,
            "expected initial focus_ring_always_paint=false"
        );

        // Focus it and switch modality to keyboard (focus-visible).
        ui.set_focus(Some(trigger_node));
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::KeyDown {
                key: fret_core::KeyCode::Tab,
                modifiers: fret_core::Modifiers::default(),
                repeat: false,
            },
        );

        // Frame 2: focus-visible should start a tween (intermediate alpha) and paint-on-exit is ok.
        app.set_frame_id(FrameId(2));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            trigger_id_out.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let alpha2 = ring_alpha_out.get().expect("alpha2");
        assert!(
            alpha2 > 1e-6 && alpha2 < base_alpha - 1e-6,
            "expected ring alpha to tween in (intermediate); got alpha={alpha2} base_alpha={base_alpha}"
        );
        assert!(
            always_paint_out.get().expect("always paint2"),
            "expected focus_ring_always_paint while animating"
        );

        // Advance frames until the default 150ms transition settles.
        let settle = ticks_60hz_for_duration(Duration::from_millis(150)) + 2;
        for i in 0..settle {
            app.set_frame_id(FrameId(3 + i));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                trigger_id_out.clone(),
                ring_alpha_out.clone(),
                always_paint_out.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let alpha_final = ring_alpha_out.get().expect("alpha_final");
        assert!(
            (alpha_final - base_alpha).abs() <= 1e-4,
            "expected ring alpha to settle to base; got alpha={alpha_final} base_alpha={base_alpha}"
        );
        assert!(
            !always_paint_out.get().expect("always paint final"),
            "expected focus_ring_always_paint=false once settled"
        );

        // Blur: should animate out and keep painting while alpha decreases.
        ui.set_focus(None);
        app.set_frame_id(FrameId(3 + settle));
        render_frame(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            trigger_id_out.clone(),
            ring_alpha_out.clone(),
            always_paint_out.clone(),
        );
        ui.layout_all(&mut app, &mut services, bounds, 1.0);
        let alpha_out = ring_alpha_out.get().expect("alpha_out");
        assert!(
            alpha_out > 1e-6 && alpha_out < base_alpha - 1e-6,
            "expected ring alpha to tween out (intermediate); got alpha={alpha_out} base_alpha={base_alpha}"
        );
        assert!(
            always_paint_out.get().expect("always paint out"),
            "expected focus_ring_always_paint while animating out"
        );

        for i in 0..settle {
            app.set_frame_id(FrameId(4 + settle + i));
            render_frame(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                trigger_id_out.clone(),
                ring_alpha_out.clone(),
                always_paint_out.clone(),
            );
            ui.layout_all(&mut app, &mut services, bounds, 1.0);
        }

        let alpha_zero = ring_alpha_out.get().expect("alpha_zero");
        assert!(
            alpha_zero.abs() <= 1e-4,
            "expected ring alpha to settle to 0; got alpha={alpha_zero}"
        );
        assert!(
            !always_paint_out.get().expect("always paint after out"),
            "expected focus_ring_always_paint=false once ring alpha is 0"
        );
    }

    #[test]
    fn accordion_content_defaults_use_zero_gap_and_do_not_stretch_children() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let b = bounds();

        fret_ui::elements::with_element_cx(&mut app, window, b, "test", |cx| {
            let el = AccordionContent::new([cx.text("a"), cx.text("b")]).into_element(cx);
            let child = el.children.first().expect("content child");

            let ElementKind::Column(props) = &child.kind else {
                panic!("expected Column child");
            };
            assert_eq!(props.align, CrossAlign::Start);
            assert_eq!(props.gap, SpacingLength::Px(Px(0.0)));
        });
    }

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

    impl fret_core::MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Ok(fret_core::MaterialId::default())
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
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

    fn render_accordion_frame_with_measured_content_test_id(
        ui: &mut UiTree<App>,
        app: &mut App,
        services: &mut dyn fret_core::UiServices,
        window: AppWindowId,
        bounds: Rect,
        open: fret_runtime::Model<Option<Arc<str>>>,
        content_test_id: &str,
    ) {
        app.set_tick_id(TickId(app.tick_id().0.saturating_add(1)));
        app.set_frame_id(FrameId(app.frame_id().0.saturating_add(1)));

        let content_id: Arc<str> = Arc::from(content_test_id);
        let root =
            fret_ui::declarative::render_root(ui, app, services, window, bounds, "test", |cx| {
                let content_lines = vec![
                    cx.text("Line 1"),
                    cx.text("Line 2"),
                    cx.text("Line 3"),
                    cx.text("Line 4"),
                    cx.text("Line 5"),
                    cx.text("Line 6"),
                ];

                let item_1 = AccordionItem::new(
                    Arc::from("item-1"),
                    AccordionTrigger::new(vec![cx.text("Item 1")])
                        .refine_layout(LayoutRefinement::default().h_px(Px(40.0))),
                    AccordionContent::new(content_lines).test_id(content_id.clone()),
                );
                let item_2 = AccordionItem::new(
                    Arc::from("item-2"),
                    AccordionTrigger::new(vec![cx.text("Item 2")])
                        .refine_layout(LayoutRefinement::default().h_px(Px(40.0))),
                    AccordionContent::new(vec![cx.text("Content 2")]),
                );

                let accordion = Accordion::single(open)
                    .collapsible(true)
                    .items([item_1, item_2])
                    .into_element(cx);

                vec![accordion]
            });

        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(app, services, bounds, 1.0);
    }

    fn node_id_by_test_id(ui: &UiTree<App>, test_id: &str) -> NodeId {
        ui.semantics_snapshot()
            .expect("semantics snapshot")
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some(test_id))
            .unwrap_or_else(|| panic!("missing semantics test_id={test_id}"))
            .id
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

    impl fret_core::MaterialService for MeasuredServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Ok(fret_core::MaterialId::default())
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
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
    fn accordion_content_measured_height_animates_between_open_and_closed_over_time() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let open = app
            .models_mut()
            .insert::<Option<Arc<str>>>(Some(Arc::from("item-1")));
        let mut services = MeasuredServices::default();

        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(600.0)),
        );

        // Let the open state settle and ensure a stable measured height.
        for _ in 0..12 {
            render_accordion_frame_with_measured_content_test_id(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                "accordion-content",
            );
        }
        let content_node = node_id_by_test_id(&ui, "accordion-content");
        let open_bounds = ui
            .debug_node_bounds(content_node)
            .expect("open content bounds");
        let open_h = open_bounds.size.height;
        assert!(
            open_h.0 > 1.0,
            "expected measured open height > 1px, got {open_h:?}"
        );

        let _ = app.models_mut().update(&open, |v| *v = None);

        let mut heights: Vec<Px> = Vec::new();
        for _ in 0..24 {
            render_accordion_frame_with_measured_content_test_id(
                &mut ui,
                &mut app,
                &mut services,
                window,
                bounds,
                open.clone(),
                "accordion-content",
            );

            let node = ui
                .semantics_snapshot()
                .expect("semantics snapshot")
                .nodes
                .iter()
                .find(|n| n.test_id.as_deref() == Some("accordion-content"))
                .map(|n| n.id);

            let Some(node) = node else {
                break;
            };

            let b = ui.debug_node_bounds(node).expect("content bounds");
            heights.push(b.size.height);
        }

        assert!(
            heights.len() >= 3,
            "expected at least 3 mounted frames during close animation, got {}",
            heights.len()
        );

        let start_h = heights[0];
        let min_h = heights
            .iter()
            .copied()
            .fold(Px(f32::INFINITY), |a, b| Px(a.0.min(b.0)));

        assert!(
            start_h.0 <= open_h.0 + 0.5 && start_h.0 >= open_h.0 * 0.75,
            "expected close transition to start near the open height; open={open_h:?} start={start_h:?}"
        );

        let saw_intermediate = heights.iter().any(|h| h.0 > 1.0 && h.0 < open_h.0 - 1.0);
        assert!(
            saw_intermediate,
            "expected at least one intermediate measured height during close transition; open={open_h:?} heights={heights:?}"
        );

        assert!(
            min_h.0 < open_h.0 * 0.5,
            "expected close transition to significantly reduce height before unmount; open={open_h:?} min={min_h:?} heights={heights:?}"
        );
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
    fn accordion_content_is_region_and_labelled_by_trigger_when_open() {
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

        let content_id = *trigger_node
            .controls
            .first()
            .expect("trigger controls node");
        let content_node = snap
            .nodes
            .iter()
            .find(|n| n.id == content_id)
            .expect("content semantics node");

        assert_eq!(
            content_node.role,
            fret_core::SemanticsRole::Region,
            "expected accordion content to surface as role=region"
        );
        assert!(
            content_node.labelled_by.contains(&trigger_node.id),
            "expected accordion content to be labelled_by its trigger"
        );
    }

    #[test]
    fn accordion_trigger_open_non_collapsible_is_aria_disabled() {
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
                false,
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
            trigger_node.flags.disabled,
            "expected open non-collapsible trigger to be disabled for assistive tech (aria-disabled)"
        );
        assert!(
            !trigger_node.actions.invoke,
            "expected open non-collapsible trigger to suppress the click/invoke action"
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
    fn accordion_content_is_region_and_labelled_by_trigger_when_open_composable() {
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

        let content_id = *trigger_node
            .controls
            .first()
            .expect("trigger controls node");
        let content_node = snap
            .nodes
            .iter()
            .find(|n| n.id == content_id)
            .expect("content semantics node");

        assert_eq!(
            content_node.role,
            fret_core::SemanticsRole::Region,
            "expected accordion content to surface as role=region"
        );
        assert!(
            content_node.labelled_by.contains(&trigger_node.id),
            "expected accordion content to be labelled_by its trigger"
        );
    }

    #[test]
    fn accordion_trigger_open_non_collapsible_is_aria_disabled_composable() {
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
                false,
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
            trigger_node.flags.disabled,
            "expected open non-collapsible trigger to be disabled for assistive tech (aria-disabled)"
        );
        assert!(
            !trigger_node.actions.invoke,
            "expected open non-collapsible trigger to suppress the click/invoke action"
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

        render_accordion_frame_with_on_value_change(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            open.clone(),
            Some(on_value_change.clone()),
        );
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
