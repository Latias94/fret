//! AI Elements-aligned `Context` surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/context.tsx`.

use std::sync::Arc;

use fret_core::{FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, LayoutStyle, SemanticsDecoration, SemanticsProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::{ChromeRefinement, ColorRef, Items, Justify, LayoutRefinement, Space};
use fret_ui_shadcn::{Button, ButtonVariant, HoverCard, HoverCardContent, Progress};

fn percent_used(used_tokens: u64, max_tokens: u64) -> f32 {
    if max_tokens == 0 {
        return 0.0;
    }
    (used_tokens as f64 / max_tokens as f64).clamp(0.0, 1.0) as f32
}

fn format_percent(p: f32) -> Arc<str> {
    Arc::<str>::from(format!("{:.1}%", (p.clamp(0.0, 1.0) * 100.0) as f64))
}

fn format_compact_u64(v: u64) -> Arc<str> {
    // Upstream uses Intl compact formatting; we approximate with a small suffix set.
    const K: u64 = 1_000;
    const M: u64 = 1_000_000;
    const B: u64 = 1_000_000_000;

    let s = if v >= B {
        format!("{:.1}B", v as f64 / B as f64)
    } else if v >= M {
        format!("{:.1}M", v as f64 / M as f64)
    } else if v >= K {
        format!("{:.1}K", v as f64 / K as f64)
    } else {
        v.to_string()
    };
    Arc::<str>::from(s)
}

#[derive(Debug, Clone)]
pub struct ContextUsage {
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
}

impl Default for ContextUsage {
    fn default() -> Self {
        Self {
            prompt_tokens: None,
            completion_tokens: None,
            total_tokens: None,
        }
    }
}

/// AI Elements-aligned context usage hovercard (`context.tsx`).
#[derive(Clone)]
pub struct Context {
    used_tokens: u64,
    max_tokens: u64,
    model_id: Option<Arc<str>>,
    usage: Option<ContextUsage>,
    trigger: Option<AnyElement>,
    content: Option<AnyElement>,
    test_id_trigger: Option<Arc<str>>,
    test_id_content: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("used_tokens", &self.used_tokens)
            .field("max_tokens", &self.max_tokens)
            .field("model_id", &self.model_id.as_deref())
            .field("has_usage", &self.usage.is_some())
            .field("has_trigger", &self.trigger.is_some())
            .field("has_content", &self.content.is_some())
            .field("test_id_trigger", &self.test_id_trigger.as_deref())
            .field("test_id_content", &self.test_id_content.as_deref())
            .field("layout", &self.layout)
            .finish()
    }
}

impl Context {
    pub fn new(used_tokens: u64, max_tokens: u64) -> Self {
        Self {
            used_tokens,
            max_tokens,
            model_id: None,
            usage: None,
            trigger: None,
            content: None,
            test_id_trigger: None,
            test_id_content: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn model_id(mut self, model_id: impl Into<Arc<str>>) -> Self {
        self.model_id = Some(model_id.into());
        self
    }

    pub fn usage(mut self, usage: ContextUsage) -> Self {
        self.usage = Some(usage);
        self
    }

    pub fn trigger(mut self, trigger: AnyElement) -> Self {
        self.trigger = Some(trigger);
        self
    }

    pub fn content(mut self, content: AnyElement) -> Self {
        self.content = Some(content);
        self
    }

    pub fn test_id_trigger(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_trigger = Some(id.into());
        self
    }

    pub fn test_id_content(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_content = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let trigger = self.trigger.unwrap_or_else(|| {
            ContextTrigger::new(self.used_tokens, self.max_tokens)
                .test_id_opt(self.test_id_trigger.clone())
                .into_element(cx)
        });

        let content = self.content.unwrap_or_else(|| {
            ContextContent::new(self.used_tokens, self.max_tokens)
                .model_id_opt(self.model_id.clone())
                .usage_opt(self.usage.clone())
                .test_id_opt(self.test_id_content.clone())
                .into_element(cx)
        });

        HoverCard::new(trigger, content)
            .open_delay_frames(0)
            .close_delay_frames(0)
            .refine_layout(self.layout)
            .into_element(cx)
    }
}

#[derive(Clone, Default)]
pub struct ContextTrigger {
    used_tokens: u64,
    max_tokens: u64,
    test_id: Option<Arc<str>>,
}

impl ContextTrigger {
    pub fn new(used_tokens: u64, max_tokens: u64) -> Self {
        Self {
            used_tokens,
            max_tokens,
            test_id: None,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    fn test_id_opt(mut self, id: Option<Arc<str>>) -> Self {
        self.test_id = id;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let pct = percent_used(self.used_tokens, self.max_tokens);
        let pct_text = cx.text(format_percent(pct));

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_center()
                .justify(Justify::Between),
            move |_cx| vec![pct_text],
        );

        let btn = Button::new("Context")
            .children([row])
            .variant(ButtonVariant::Ghost)
            .refine_style(
                ChromeRefinement::default()
                    .text_color(ColorRef::Color(theme.color_required("muted-foreground"))),
            )
            .into_element(cx);

        if let Some(test_id) = self.test_id {
            btn.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Button)
                    .test_id(test_id),
            )
        } else {
            btn
        }
    }
}

#[derive(Clone, Default)]
pub struct ContextContent {
    used_tokens: u64,
    max_tokens: u64,
    model_id: Option<Arc<str>>,
    usage: Option<ContextUsage>,
    test_id: Option<Arc<str>>,
}

impl ContextContent {
    pub fn new(used_tokens: u64, max_tokens: u64) -> Self {
        Self {
            used_tokens,
            max_tokens,
            model_id: None,
            usage: None,
            test_id: None,
        }
    }

    pub fn model_id(mut self, model_id: impl Into<Arc<str>>) -> Self {
        self.model_id = Some(model_id.into());
        self
    }

    fn model_id_opt(mut self, model_id: Option<Arc<str>>) -> Self {
        self.model_id = model_id;
        self
    }

    pub fn usage(mut self, usage: ContextUsage) -> Self {
        self.usage = Some(usage);
        self
    }

    fn usage_opt(mut self, usage: Option<ContextUsage>) -> Self {
        self.usage = usage;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    fn test_id_opt(mut self, id: Option<Arc<str>>) -> Self {
        self.test_id = id;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let pct = percent_used(self.used_tokens, self.max_tokens);

        let display_pct = cx.text(format_percent(pct));
        let used = format_compact_u64(self.used_tokens);
        let total = format_compact_u64(self.max_tokens);
        let used_total = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: Arc::<str>::from(format!("{used} / {total}")),
            style: Some(TextStyle {
                font: FontId::monospace(),
                size: theme.metric_required("component.text.xs_px"),
                weight: FontWeight::NORMAL,
                slant: Default::default(),
                line_height: Some(theme.metric_required("component.text.xs_line_height")),
                letter_spacing_em: None,
            }),
            color: Some(theme.color_required("muted-foreground")),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        });

        let header_row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .items_center()
                .justify(Justify::Between)
                .gap(Space::N2),
            move |_cx| vec![display_pct, used_total],
        );

        let progress_model = cx.with_state(|| None::<Model<f32>>, |st| st.clone());
        let progress_model = if let Some(m) = progress_model {
            m
        } else {
            let m = cx.app.models_mut().insert(pct * 100.0);
            cx.with_state(|| None::<Model<f32>>, |st| *st = Some(m.clone()));
            m
        };

        let next = pct * 100.0;
        let _ = cx.app.models_mut().update(&progress_model, |v| {
            if (*v - next).abs() > f32::EPSILON {
                *v = next;
            }
        });

        let progress = Progress::new(progress_model).into_element(cx);

        let header = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N2),
            move |_cx| vec![header_row, progress],
        );

        let mut body_rows: Vec<AnyElement> = Vec::new();
        if let Some(model_id) = self.model_id {
            body_rows.push(
                cx.text_props(TextProps {
                    layout: LayoutStyle::default(),
                    text: Arc::<str>::from(format!("model: {model_id}")),
                    style: Some(TextStyle {
                        font: FontId::monospace(),
                        size: theme.metric_required("component.text.xs_px"),
                        weight: FontWeight::NORMAL,
                        slant: Default::default(),
                        line_height: Some(theme.metric_required("component.text.xs_line_height")),
                        letter_spacing_em: None,
                    }),
                    color: Some(theme.color_required("muted-foreground")),
                    wrap: TextWrap::None,
                    overflow: TextOverflow::Clip,
                })
                .attach_semantics(SemanticsDecoration::default().role(SemanticsRole::Text)),
            );
        }

        if let Some(usage) = self.usage {
            let prompt = usage.prompt_tokens.unwrap_or(0);
            let completion = usage.completion_tokens.unwrap_or(0);
            let total = usage
                .total_tokens
                .unwrap_or(prompt.saturating_add(completion));
            body_rows.push(cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: Arc::<str>::from(format!(
                    "usage: prompt={prompt} completion={completion} total={total}"
                )),
                style: Some(TextStyle {
                    font: FontId::monospace(),
                    size: theme.metric_required("component.text.xs_px"),
                    weight: FontWeight::NORMAL,
                    slant: Default::default(),
                    line_height: Some(theme.metric_required("component.text.xs_line_height")),
                    letter_spacing_em: None,
                }),
                color: Some(theme.color_required("muted-foreground")),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            }));
        }

        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N2)
                .items(Items::Start),
            move |_cx| body_rows,
        );

        let content = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N3),
            move |_cx| vec![header, body],
        );

        let card = HoverCardContent::new(vec![content])
            .refine_style(ChromeRefinement::default().p(Space::N3))
            .refine_layout(LayoutRefinement::default().w_px(Px(240.0)).min_w_0())
            .into_element(cx);

        if let Some(test_id) = self.test_id {
            cx.semantics(
                SemanticsProps {
                    role: SemanticsRole::Group,
                    test_id: Some(test_id),
                    ..Default::default()
                },
                move |_cx| vec![card],
            )
        } else {
            card
        }
    }
}
