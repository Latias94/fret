use fret_core::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

use fret_core::{Point, Px, SemanticsRole, Transform2D};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, PressableA11y, PressableProps, SemanticsDecoration, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::ModelWatchExt;
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::transition::{
    drive_transition_with_durations_and_cubic_bezier, ticks_60hz_for_duration,
};
use fret_ui_kit::ui;
use fret_ui_kit::{ColorRef, Items, LayoutRefinement, Space, typography};

use crate::elements::Shimmer;

const AUTO_CLOSE_DELAY: Duration = Duration::from_millis(1000);
const MS_IN_S: u128 = 1000;

#[derive(Debug, Clone)]
pub struct ReasoningController {
    pub open: Model<bool>,
    pub is_open: bool,
    pub is_streaming: bool,
    pub duration_secs: Option<u32>,
}

pub fn use_reasoning_controller<H: UiHost>(
    cx: &ElementContext<'_, H>,
) -> Option<ReasoningController> {
    cx.provided::<ReasoningController>().cloned()
}

#[derive(Debug, Default)]
struct ReasoningLogicState {
    has_ever_streamed: bool,
    has_auto_closed: bool,
    started_at: Option<Instant>,
    computed_duration_secs: Option<u32>,
    auto_close_deadline: Option<Instant>,
}

#[derive(Clone, Default)]
struct ReasoningLogicRef(Arc<Mutex<ReasoningLogicState>>);

impl ReasoningLogicRef {
    fn lock(&self) -> std::sync::MutexGuard<'_, ReasoningLogicState> {
        self.0.lock().expect("reasoning state lock poisoned")
    }
}

/// AI Elements-aligned "reasoning" disclosure container.
///
/// Upstream reference: `repo-ref/ai-elements/packages/elements/src/reasoning.tsx`.
#[derive(Clone)]
pub struct Reasoning {
    is_streaming: bool,
    open: Option<Model<bool>>,
    default_open: Option<bool>,
    duration_secs: Option<u32>,
    test_id_root: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Reasoning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reasoning")
            .field("is_streaming", &self.is_streaming)
            .field("open", &"<model>")
            .field("default_open", &self.default_open)
            .field("duration_secs", &self.duration_secs)
            .field("test_id_root", &self.test_id_root.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl Reasoning {
    pub fn new(is_streaming: bool) -> Self {
        Self {
            is_streaming,
            open: None,
            default_open: None,
            duration_secs: None,
            test_id_root: None,
            // Upstream default: `mb-4` via `cn("not-prose mb-4", className)`.
            layout: LayoutRefinement::default().mb(Space::N4),
        }
    }

    /// Docs-style compound children composition (Trigger + Content).
    ///
    /// This mirrors the upstream JSX shape:
    ///
    /// ```tsx
    /// <Reasoning>
    ///   <ReasoningTrigger />
    ///   <ReasoningContent>...</ReasoningContent>
    /// </Reasoning>
    /// ```
    pub fn children<I, C>(self, children: I) -> ReasoningWithChildren
    where
        I: IntoIterator<Item = C>,
        C: Into<ReasoningChild>,
    {
        ReasoningWithChildren {
            root: self,
            children: children.into_iter().map(Into::into).collect(),
        }
    }

    pub fn trigger(self, trigger: ReasoningTrigger) -> ReasoningWithChildren {
        self.children([ReasoningChild::Trigger(trigger)])
    }

    pub fn content(self, content: ReasoningContent) -> ReasoningWithChildren {
        self.children([ReasoningChild::Content(content)])
    }

    /// Provide a controlled open model (Radix `open`).
    pub fn open(mut self, open: Model<bool>) -> Self {
        self.open = Some(open);
        self
    }

    /// Uncontrolled initial open value (Radix `defaultOpen`).
    ///
    /// Notes:
    /// - If `Some(false)`, it also suppresses auto-open when streaming begins (AI Elements parity).
    /// - If `None`, the default is `is_streaming` (AI Elements parity).
    pub fn default_open(mut self, default_open: Option<bool>) -> Self {
        self.default_open = default_open;
        self
    }

    /// Override the displayed duration in seconds (AI Elements `duration` prop).
    pub fn duration_secs(mut self, duration_secs: Option<u32>) -> Self {
        self.duration_secs = duration_secs;
        self
    }

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        use fret_ui_kit::primitives::collapsible::CollapsibleRoot;
        use fret_ui_shadcn::facade::Collapsible;

        let is_streaming = self.is_streaming;
        let resolved_default_open = self.default_open.unwrap_or(is_streaming);
        let is_explicitly_closed = self.default_open == Some(false);
        let open_root = CollapsibleRoot::new()
            .open(self.open)
            .default_open(resolved_default_open);

        let duration_prop = self.duration_secs;
        let test_id_root = self.test_id_root;
        let layout = self.layout;

        cx.scope(move |cx| {
            let logic = cx.root_state(ReasoningLogicRef::default, |st| st.clone());
            let open = open_root.use_open_model(cx).model();
            let is_open = cx.watch_model(&open).layout().copied().unwrap_or(false);

            let theme = Theme::global(&*cx.app).clone();
            let wrapper = cx.container(
                ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(&theme, layout),
                    ..Default::default()
                },
                move |cx| {
                    let now_duration = {
                        let mut st = logic.lock();
                        if !st.has_ever_streamed {
                            st.has_ever_streamed = is_streaming;
                        }

                        if is_streaming {
                            st.has_ever_streamed = true;
                            if st.started_at.is_none() {
                                st.started_at = Some(Instant::now());
                            }
                        } else if let Some(started_at) = st.started_at.take() {
                            let elapsed_ms = started_at.elapsed().as_millis();
                            let secs = (elapsed_ms + (MS_IN_S - 1)) / MS_IN_S;
                            st.computed_duration_secs = Some(u32::try_from(secs).unwrap_or(0));
                        }

                        duration_prop.or(st.computed_duration_secs)
                    };

                    // Auto-open when streaming starts (unless defaultOpen was explicitly false).
                    if is_streaming && !is_open && !is_explicitly_closed {
                        let _ = cx.app.models_mut().update(&open, |v| *v = true);
                        cx.request_frame();
                    }

                    // Auto-close once when streaming ends.
                    {
                        let wants_auto_close = {
                            let st = logic.lock();
                            st.has_ever_streamed && !is_streaming && is_open && !st.has_auto_closed
                        };

                        let mut st = logic.lock();
                        if wants_auto_close && st.auto_close_deadline.is_none() {
                            st.auto_close_deadline = Some(Instant::now() + AUTO_CLOSE_DELAY);
                        } else if !wants_auto_close {
                            st.auto_close_deadline = None;
                        }

                        if let Some(deadline) = st.auto_close_deadline {
                            // Drive time-based progression without relying on runner timer
                            // routing (which requires explicit token -> element mapping).
                            cx.request_animation_frame();
                            if Instant::now() >= deadline {
                                st.auto_close_deadline = None;
                                st.has_auto_closed = true;
                                let _ = cx.app.models_mut().update(&open, |v| *v = false);
                                cx.request_frame();
                            }
                        }
                    }

                    let controller = ReasoningController {
                        open: open.clone(),
                        is_open,
                        is_streaming,
                        duration_secs: now_duration,
                    };
                    let collapsible = Collapsible::new(open.clone());

                    vec![cx.provide(controller, move |cx| {
                        collapsible.into_element_with_open_model(
                            cx,
                            move |cx, _open, _is_open| trigger(cx),
                            move |cx| content(cx),
                        )
                    })]
                },
            );

            match test_id_root {
                Some(test_id) => wrapper.attach_semantics(
                    SemanticsDecoration::default()
                        .role(SemanticsRole::Group)
                        .test_id(test_id),
                ),
                None => wrapper,
            }
        })
    }
}

pub enum ReasoningChild {
    Trigger(ReasoningTrigger),
    Content(ReasoningContent),
}

impl std::fmt::Debug for ReasoningChild {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trigger(_) => f.write_str("ReasoningChild::Trigger(..)"),
            Self::Content(_) => f.write_str("ReasoningChild::Content(..)"),
        }
    }
}

impl From<ReasoningTrigger> for ReasoningChild {
    fn from(value: ReasoningTrigger) -> Self {
        Self::Trigger(value)
    }
}

impl From<ReasoningContent> for ReasoningChild {
    fn from(value: ReasoningContent) -> Self {
        Self::Content(value)
    }
}

#[derive(Debug)]
pub struct ReasoningWithChildren {
    root: Reasoning,
    children: Vec<ReasoningChild>,
}

impl ReasoningWithChildren {
    pub fn children<I, C>(mut self, children: I) -> Self
    where
        I: IntoIterator<Item = C>,
        C: Into<ReasoningChild>,
    {
        self.children.extend(children.into_iter().map(Into::into));
        self
    }

    pub fn trigger(self, trigger: ReasoningTrigger) -> Self {
        self.children([ReasoningChild::Trigger(trigger)])
    }

    pub fn content(self, content: ReasoningContent) -> Self {
        self.children([ReasoningChild::Content(content)])
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Self { root, children } = self;

        let mut trigger: Option<ReasoningTrigger> = None;
        let mut content: Option<ReasoningContent> = None;

        for child in children {
            match child {
                ReasoningChild::Trigger(value) => {
                    if trigger.is_some() {
                        debug_assert!(false, "Reasoning expects a single ReasoningTrigger");
                    }
                    trigger = Some(value);
                }
                ReasoningChild::Content(value) => {
                    if content.is_some() {
                        debug_assert!(false, "Reasoning expects a single ReasoningContent");
                    }
                    content = Some(value);
                }
            }
        }

        let trigger = trigger.unwrap_or_else(ReasoningTrigger::new);
        let content = content.unwrap_or_else(|| ReasoningContent::new(""));

        root.into_element(
            cx,
            move |cx| trigger.into_element(cx),
            move |cx| content.into_element(cx),
        )
    }
}

pub struct ReasoningTrigger {
    children: Option<Vec<AnyElement>>,
    thinking_children: Option<Vec<AnyElement>>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for ReasoningTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReasoningTrigger")
            .field("children_len", &self.children.as_ref().map(|v| v.len()))
            .field(
                "thinking_children_len",
                &self.thinking_children.as_ref().map(|v| v.len()),
            )
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl ReasoningTrigger {
    pub fn new() -> Self {
        Self {
            children: None,
            thinking_children: None,
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    /// Overrides the full Trigger body (similar to JSX `children`).
    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }

    /// Fret convenience API: override only the "thinking message" slot while retaining the
    /// default Brain/Chevron affordances.
    pub fn thinking_children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.thinking_children = Some(children.into_iter().collect());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(controller) = use_reasoning_controller(cx) else {
            debug_assert!(
                false,
                "ReasoningTrigger must be rendered within a Reasoning scope"
            );
            return cx.container(Default::default(), |_| Vec::new());
        };

        let theme = Theme::global(&*cx.app).clone();
        let muted_fg = theme.color_token("muted-foreground");
        let fg_hover = theme.color_token("foreground");

        let open = controller.open.clone();
        let is_open = controller.is_open;
        let is_streaming = controller.is_streaming;
        let duration_secs = controller.duration_secs;

        let children = self.children;
        let thinking_children = self.thinking_children;
        let test_id = self.test_id;
        let layout = self.layout;

        cx.pressable(
            PressableProps {
                layout: fret_ui_kit::declarative::style::layout_style(&theme, layout),
                enabled: true,
                focusable: true,
                a11y: PressableA11y {
                    role: Some(SemanticsRole::Button),
                    label: Some(Arc::<str>::from("Toggle reasoning")),
                    test_id,
                    ..Default::default()
                },
                ..Default::default()
            },
            move |cx, st| {
                cx.pressable_toggle_bool(&open);

                let fg = if st.hovered { fg_hover } else { muted_fg };
                let icon_size = Px(16.0);

                if let Some(children) = children {
                    let row = ui::h_row(move |_cx| children)
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .items(Items::Center)
                        .gap(Space::N2)
                        .into_element(cx);

                    return vec![reasoning_message_scope(row, &theme, fg)];
                }

                let brain = decl_icon::icon_with(
                    cx,
                    fret_icons::IconId::new_static("lucide.brain"),
                    Some(icon_size),
                    Some(ColorRef::Color(fg)),
                );

                let thinking = if let Some(children) = thinking_children {
                    reasoning_message_scope(
                        cx.stack_props(
                            fret_ui::element::StackProps {
                                layout: fret_ui_kit::declarative::style::layout_style(
                                    &theme,
                                    LayoutRefinement::default().min_w_0().flex_1(),
                                ),
                            },
                            move |_cx| children,
                        ),
                        &theme,
                        fg,
                    )
                } else {
                    default_thinking_message(cx, &theme, fg, is_streaming, duration_secs)
                };

                let (toggle_ticks, toggle_easing) = {
                    let theme_full = Theme::global(&*cx.app);
                    let toggle_duration = {
                        theme_full
                            .duration_ms_by_key("duration.shadcn.motion.collapsible.toggle")
                            .or_else(|| {
                                theme_full.duration_ms_by_key("duration.motion.collapsible.toggle")
                            })
                            .or_else(|| theme_full.duration_ms_by_key("duration.shadcn.motion.200"))
                    }
                    .map(|ms| Duration::from_millis(ms as u64))
                    .unwrap_or(Duration::from_millis(200));
                    let ticks = ticks_60hz_for_duration(toggle_duration);
                    let easing = theme_full
                        .easing_by_key("easing.shadcn.motion.collapsible.toggle")
                        .or_else(|| theme_full.easing_by_key("easing.motion.collapsible.toggle"))
                        .or_else(|| theme_full.easing_by_key("easing.shadcn.motion"))
                        .or_else(|| theme_full.easing_by_key("easing.motion.standard"))
                        .unwrap_or(fret_ui::theme::CubicBezier {
                            x1: 0.4,
                            y1: 0.0,
                            x2: 0.2,
                            y2: 1.0,
                        });
                    (ticks, easing)
                };

                let chevron_motion = drive_transition_with_durations_and_cubic_bezier(
                    cx,
                    is_open,
                    toggle_ticks,
                    toggle_ticks,
                    toggle_easing,
                );
                let chevron_rotation = 180.0 * chevron_motion.progress;
                let center = Point::new(Px(8.0), Px(8.0));
                let chevron_transform =
                    Transform2D::rotation_about_degrees(chevron_rotation, center);
                let chevron = cx.visual_transform_props(
                    fret_ui::element::VisualTransformProps {
                        layout: fret_ui_kit::declarative::style::layout_style(
                            &theme,
                            LayoutRefinement::default()
                                .w_px(fret_ui_kit::MetricRef::Px(icon_size))
                                .h_px(fret_ui_kit::MetricRef::Px(icon_size))
                                .flex_shrink_0(),
                        ),
                        transform: chevron_transform,
                    },
                    move |cx| {
                        vec![decl_icon::icon_with(
                            cx,
                            fret_icons::ids::ui::CHEVRON_DOWN,
                            Some(icon_size),
                            Some(ColorRef::Color(fg)),
                        )]
                    },
                );

                let row = ui::h_row(move |_cx| vec![brain, thinking, chevron])
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .items(Items::Center)
                    .gap(Space::N2)
                    .into_element(cx);

                vec![row]
            },
        )
    }
}

fn reasoning_message_scope(element: AnyElement, theme: &Theme, fg: fret_core::Color) -> AnyElement {
    fret_ui_kit::typography::scope_text_style_with_color(
        element,
        typography::preset_text_refinement(
            theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
        ),
        fg,
    )
}

fn default_thinking_message<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    fg: fret_core::Color,
    is_streaming: bool,
    duration_secs: Option<u32>,
) -> AnyElement {
    if is_streaming || duration_secs == Some(0) {
        return reasoning_message_scope(
            Shimmer::new("Thinking...")
                .duration_secs(1.0)
                .use_resolved_passive_text()
                .role(SemanticsRole::Text)
                .into_element(cx),
            theme,
            fg,
        );
    }

    let text: Arc<str> = if let Some(duration) = duration_secs {
        Arc::<str>::from(format!("Thought for {duration} seconds"))
    } else {
        Arc::<str>::from("Thought for a few seconds")
    };

    reasoning_message_scope(
        cx.text_props(TextProps {
            layout: Default::default(),
            text,
            style: None,
            color: None,
            wrap: fret_core::TextWrap::Word,
            overflow: fret_core::TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: fret_ui::element::TextInkOverflow::None,
        }),
        theme,
        fg,
    )
}

#[derive(Clone)]
pub struct ReasoningContent {
    markdown: Arc<str>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for ReasoningContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReasoningContent")
            .field("markdown_len", &self.markdown.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl ReasoningContent {
    pub fn new(markdown: impl Into<Arc<str>>) -> Self {
        Self {
            markdown: markdown.into(),
            test_id: None,
            layout: LayoutRefinement::default().mt(Space::N4).w_full().min_w_0(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let muted_fg = theme.color_token("muted-foreground");

        let mut components = fret_markdown::MarkdownComponents::<H>::default();
        // Reasoning content is usually non-interactive; keep links inert by default.
        components.on_link_activate = None;

        let content = fret_markdown::Markdown::new(self.markdown)
            .into_element_with_non_windowed(cx, &components)
            .inherit_foreground(muted_fg);

        let inner = cx.container(
            fret_ui::element::ContainerProps {
                layout: fret_ui_kit::declarative::style::layout_style(&theme, self.layout),
                ..Default::default()
            },
            move |_cx| vec![content],
        );

        match self.test_id {
            Some(test_id) => inner.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .test_id(test_id),
            ),
            None => inner,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::{AnyElement, ElementKind};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        )
    }

    fn find_text_by_content<'a>(element: &'a AnyElement, content: &str) -> Option<&'a AnyElement> {
        if let ElementKind::Text(props) = &element.kind
            && props.text.as_ref() == content
        {
            return Some(element);
        }

        element
            .children
            .iter()
            .find_map(|child| find_text_by_content(child, content))
    }

    fn has_scoped_text_style(
        element: &AnyElement,
        refinement: &fret_core::TextStyleRefinement,
        foreground: fret_core::Color,
    ) -> bool {
        if element.inherited_text_style.as_ref() == Some(refinement)
            && element.inherited_foreground == Some(foreground)
        {
            return true;
        }

        element
            .children
            .iter()
            .any(|child| has_scoped_text_style(child, refinement, foreground))
    }

    fn find_element_by_test_id<'a>(
        element: &'a AnyElement,
        test_id: &str,
    ) -> Option<&'a AnyElement> {
        if element
            .semantics_decoration
            .as_ref()
            .and_then(|dec| dec.test_id.as_deref())
            == Some(test_id)
        {
            return Some(element);
        }

        match &element.kind {
            ElementKind::Semantics(props) if props.test_id.as_deref() == Some(test_id) => {
                return Some(element);
            }
            _ => {}
        }

        element
            .children
            .iter()
            .find_map(|child| find_element_by_test_id(child, test_id))
    }

    #[test]
    fn reasoning_trigger_default_streaming_message_scopes_inherited_typography_for_shimmer() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(true);

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                cx.provide(
                    ReasoningController {
                        open: open.clone(),
                        is_open: true,
                        is_streaming: true,
                        duration_secs: None,
                    },
                    |cx| vec![ReasoningTrigger::new().into_element(cx)],
                )
                .into_iter()
                .next()
                .expect("reasoning trigger provide should yield one element")
            });

        let theme = fret_ui::Theme::global(&app).clone();
        let expected_refinement = typography::preset_text_refinement(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
        );
        let expected_fg = theme.color_token("muted-foreground");

        assert!(has_scoped_text_style(
            &element,
            &expected_refinement,
            expected_fg
        ));

        let text = find_text_by_content(&element, "Thinking...")
            .expect("expected default streaming reasoning copy");
        let ElementKind::Text(props) = &text.kind else {
            panic!("expected text leaf under the scoped reasoning message");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
    }

    #[test]
    fn reasoning_trigger_default_settled_message_scopes_inherited_typography() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(true);

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                cx.provide(
                    ReasoningController {
                        open: open.clone(),
                        is_open: true,
                        is_streaming: false,
                        duration_secs: Some(3),
                    },
                    |cx| vec![ReasoningTrigger::new().into_element(cx)],
                )
                .into_iter()
                .next()
                .expect("reasoning trigger provide should yield one element")
            });

        let theme = fret_ui::Theme::global(&app).clone();
        let expected_refinement = typography::preset_text_refinement(
            &theme,
            typography::TypographyPreset::control_ui(typography::UiTextSize::Sm),
        );
        let expected_fg = theme.color_token("muted-foreground");

        assert!(has_scoped_text_style(
            &element,
            &expected_refinement,
            expected_fg
        ));

        let text = find_text_by_content(&element, "Thought for 3 seconds")
            .expect("expected settled reasoning copy");
        let ElementKind::Text(props) = &text.kind else {
            panic!("expected text leaf under the scoped reasoning message");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
    }

    #[test]
    fn reasoning_trigger_children_overrides_default_thinking_copy() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(true);

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                let custom = cx.text("Custom Trigger");
                cx.provide(
                    ReasoningController {
                        open: open.clone(),
                        is_open: true,
                        is_streaming: true,
                        duration_secs: None,
                    },
                    |cx| vec![ReasoningTrigger::new().children([custom]).into_element(cx)],
                )
                .into_iter()
                .next()
                .expect("reasoning trigger provide should yield one element")
            });

        assert!(find_text_by_content(&element, "Custom Trigger").is_some());
        assert!(find_text_by_content(&element, "Thinking...").is_none());
    }

    #[test]
    fn reasoning_trigger_thinking_children_overrides_default_thinking_copy() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let open = app.models_mut().insert(true);

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                let custom = cx.text("Custom Thinking");
                cx.provide(
                    ReasoningController {
                        open: open.clone(),
                        is_open: true,
                        is_streaming: true,
                        duration_secs: None,
                    },
                    |cx| {
                        vec![
                            ReasoningTrigger::new()
                                .thinking_children([custom])
                                .into_element(cx),
                        ]
                    },
                )
                .into_iter()
                .next()
                .expect("reasoning trigger provide should yield one element")
            });

        assert!(find_text_by_content(&element, "Custom Thinking").is_some());
        assert!(find_text_by_content(&element, "Thinking...").is_none());
    }

    #[test]
    fn reasoning_children_composition_renders_content_by_default_when_streaming() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Reasoning::new(true)
                    .trigger(ReasoningTrigger::new())
                    .content(ReasoningContent::new("Hello").test_id("content"))
                    .into_element(cx)
            });

        assert!(find_element_by_test_id(&element, "content").is_some());
    }

    #[test]
    fn reasoning_controller_is_available_inside_custom_parts() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                Reasoning::new(false).default_open(Some(true)).into_element(
                    cx,
                    |cx| {
                        let controller = use_reasoning_controller(cx)
                            .expect("reasoning controller should be provided to the trigger");
                        cx.text(format!(
                            "trigger open={} streaming={}",
                            controller.is_open, controller.is_streaming
                        ))
                    },
                    |cx| {
                        let controller = use_reasoning_controller(cx)
                            .expect("reasoning controller should be provided to the content");
                        cx.text(format!("content duration={:?}", controller.duration_secs))
                    },
                )
            });

        assert!(find_text_by_content(&element, "trigger open=true streaming=false").is_some());
        assert!(find_text_by_content(&element, "content duration=None").is_some());
    }
}
