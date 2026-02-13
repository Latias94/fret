use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use fret_core::{Point, Px, SemanticsRole, Transform2D};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, PressableA11y, PressableProps, SemanticsDecoration, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{ColorRef, LayoutRefinement, Space};

use crate::elements::Shimmer;

const AUTO_CLOSE_DELAY: Duration = Duration::from_millis(1000);
const MS_IN_S: u128 = 1000;

#[derive(Debug, Clone, Default)]
struct ReasoningContextState {
    open: Option<Model<bool>>,
    is_open: bool,
    is_streaming: bool,
    duration_secs: Option<u32>,
}

#[derive(Debug, Default)]
struct ReasoningLogicState {
    has_ever_streamed: bool,
    has_auto_closed: bool,
    started_at: Option<Instant>,
    computed_duration_secs: Option<u32>,
    auto_close_deadline: Option<Instant>,
    open: Option<Model<bool>>,
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
            layout: LayoutRefinement::default(),
        }
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

    pub fn into_element<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        trigger: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
        content: impl FnOnce(&mut ElementContext<'_, H>) -> AnyElement,
    ) -> AnyElement {
        use fret_ui_shadcn::Collapsible;

        let is_streaming = self.is_streaming;
        let resolved_default_open = self.default_open.unwrap_or(is_streaming);
        let is_explicitly_closed = self.default_open == Some(false);

        let collapsible = if let Some(open) = self.open {
            Collapsible::new(open)
        } else {
            Collapsible::uncontrolled(resolved_default_open)
        };

        let duration_prop = self.duration_secs;
        let test_id_root = self.test_id_root;
        let layout = self.layout;

        cx.scope(move |cx| {
            let logic = cx.with_state(ReasoningLogicRef::default, |st| st.clone());

            let theme = Theme::global(&*cx.app).clone();
            let wrapper = cx.container(
                ContainerProps {
                    layout: fret_ui_kit::declarative::style::layout_style(&theme, layout),
                    ..Default::default()
                },
                move |cx| {
                    let container_id = cx.root_id();
                    let _ = container_id;

                    vec![collapsible.into_element_with_open_model(
                        cx,
                        |cx, open, is_open| {
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
                                    st.computed_duration_secs =
                                        Some(u32::try_from(secs).unwrap_or(0));
                                }

                                st.open = Some(open.clone());

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
                                    st.has_ever_streamed
                                        && !is_streaming
                                        && is_open
                                        && !st.has_auto_closed
                                };

                                let mut st = logic.lock();
                                if wants_auto_close && st.auto_close_deadline.is_none() {
                                    st.auto_close_deadline =
                                        Some(Instant::now() + AUTO_CLOSE_DELAY);
                                } else if !wants_auto_close {
                                    st.auto_close_deadline = None;
                                }

                                if let Some(deadline) = st.auto_close_deadline {
                                    // Drive time-based progression without relying on runner timer
                                    // routing (which requires explicit token → element mapping).
                                    cx.request_animation_frame();
                                    if Instant::now() >= deadline {
                                        st.auto_close_deadline = None;
                                        st.has_auto_closed = true;
                                        let _ = cx.app.models_mut().update(&open, |v| *v = false);
                                        cx.request_frame();
                                    }
                                }
                            }

                            cx.with_state(ReasoningContextState::default, |ctx_state| {
                                ctx_state.open = Some(open.clone());
                                ctx_state.is_open = is_open;
                                ctx_state.is_streaming = is_streaming;
                                ctx_state.duration_secs = now_duration;
                            });

                            trigger(cx)
                        },
                        content,
                    )]
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

#[derive(Clone)]
pub struct ReasoningTrigger {
    children: Option<Vec<AnyElement>>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for ReasoningTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReasoningTrigger")
            .field("children_len", &self.children.as_ref().map(|v| v.len()))
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl ReasoningTrigger {
    pub fn new() -> Self {
        Self {
            children: None,
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(ctx_state) = cx.inherited_state::<ReasoningContextState>() else {
            debug_assert!(
                false,
                "ReasoningTrigger must be rendered within a Reasoning scope"
            );
            return cx.container(Default::default(), |_| Vec::new());
        };

        let Some(open) = ctx_state.open.clone() else {
            return cx.container(Default::default(), |_| Vec::new());
        };

        let theme = Theme::global(&*cx.app).clone();
        let muted_fg = theme.color_required("muted-foreground");
        let fg_hover = theme.color_required("foreground");

        let is_open = ctx_state.is_open;
        let is_streaming = ctx_state.is_streaming;
        let duration_secs = ctx_state.duration_secs;

        let children = self.children;
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
                let brain = decl_icon::icon_with(
                    cx,
                    fret_icons::IconId::new_static("lucide.brain"),
                    Some(icon_size),
                    Some(ColorRef::Color(fg)),
                );

                let thinking = if let Some(children) = children.clone() {
                    cx.stack_props(
                        fret_ui::element::StackProps {
                            layout: fret_ui_kit::declarative::style::layout_style(
                                &theme,
                                LayoutRefinement::default().min_w_0(),
                            ),
                        },
                        move |_cx| children,
                    )
                } else {
                    default_thinking_message(cx, &theme, fg, is_streaming, duration_secs)
                };

                let chevron_rotation = if is_open { 180.0 } else { 0.0 };
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

                let row = stack::hstack(
                    cx,
                    stack::HStackProps::default()
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .items_center()
                        .gap(Space::N2),
                    move |_cx| vec![brain, thinking, chevron],
                );

                vec![row]
            },
        )
    }
}

fn default_thinking_message<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    _theme: &Theme,
    fg: fret_core::Color,
    is_streaming: bool,
    duration_secs: Option<u32>,
) -> AnyElement {
    if is_streaming || duration_secs == Some(0) {
        return Shimmer::new("Thinking...")
            .duration_secs(1.0)
            .role(SemanticsRole::Text)
            .into_element(cx);
    }

    let text: Arc<str> = if let Some(duration) = duration_secs {
        Arc::<str>::from(format!("Thought for {duration} seconds"))
    } else {
        Arc::<str>::from("Thought for a few seconds")
    };

    cx.text_props(TextProps {
        layout: Default::default(),
        text,
        style: None,
        color: Some(fg),
        wrap: fret_core::TextWrap::Word,
        overflow: fret_core::TextOverflow::Clip,
    })
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let mut components = fret_markdown::MarkdownComponents::<H>::default();
        // Reasoning content is usually non-interactive; keep links inert by default.
        components.on_link_activate = None;

        let content =
            fret_markdown::Markdown::new(self.markdown).into_element_with(cx, &components);

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
