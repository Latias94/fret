//! AI Elements-aligned `Context` surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/context.tsx`.

use std::sync::Arc;

use fret_core::{FontWeight, Px, SemanticsRole, TextOverflow, TextWrap};
use fret_ui::SvgSource;
use fret_ui::element::{
    AnyElement, InteractivityGateProps, LayoutStyle, Length, SemanticsDecoration, SvgIconProps,
    TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::ui;
use fret_ui_kit::{ChromeRefinement, ColorRef, Items, Justify, LayoutRefinement, Space};
use fret_ui_shadcn::facade::{
    Button, ButtonVariant, HoverCard, HoverCardContent, Progress, Separator,
};

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

    fn with_suffix(value: f64, suffix: &str) -> String {
        let rounded = (value * 10.0).round() / 10.0;
        if rounded.fract().abs() < f64::EPSILON {
            format!("{rounded:.0}{suffix}")
        } else {
            format!("{rounded:.1}{suffix}")
        }
    }

    let s = if v >= B {
        with_suffix(v as f64 / B as f64, "B")
    } else if v >= M {
        with_suffix(v as f64 / M as f64, "M")
    } else if v >= K {
        with_suffix(v as f64 / K as f64, "K")
    } else {
        v.to_string()
    };
    Arc::<str>::from(s)
}

fn format_usd(v: f64) -> Arc<str> {
    // Upstream uses Intl currency formatting; we keep it deterministic and simple.
    Arc::<str>::from(format!("${:.2}", v.max(0.0)))
}

fn context_icon_svg_bytes(used_tokens: u64, max_tokens: u64) -> Arc<[u8]> {
    const ICON_RADIUS: f64 = 10.0;
    const ICON_VIEWBOX: i32 = 24;
    const ICON_CENTER: i32 = 12;
    const ICON_STROKE_WIDTH: i32 = 2;

    let used_percent = if max_tokens == 0 {
        0.0
    } else {
        (used_tokens as f64 / max_tokens as f64).clamp(0.0, 1.0)
    };

    let circumference = 2.0 * std::f64::consts::PI * ICON_RADIUS;
    let dash_offset = circumference * (1.0 - used_percent);

    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {ICON_VIEWBOX} {ICON_VIEWBOX}" width="20" height="20" role="img" aria-label="Model context usage">
  <circle cx="{ICON_CENTER}" cy="{ICON_CENTER}" fill="none" opacity="0.25" r="{ICON_RADIUS}" stroke="currentColor" stroke-width="{ICON_STROKE_WIDTH}" />
  <circle cx="{ICON_CENTER}" cy="{ICON_CENTER}" fill="none" opacity="0.7" r="{ICON_RADIUS}" stroke="currentColor" stroke-dasharray="{circumference:.4} {circumference:.4}" stroke-dashoffset="{dash_offset:.4}" stroke-linecap="round" stroke-width="{ICON_STROKE_WIDTH}" transform="rotate(-90 {ICON_CENTER} {ICON_CENTER})" />
</svg>"#
    );

    Arc::<[u8]>::from(svg.into_bytes())
}

#[derive(Debug, Clone)]
pub struct ContextUsage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,
    pub cached_input_tokens: Option<u64>,
    /// Optional precomputed USD costs (app-owned). When omitted, Fret falls back to built-in
    /// model pricing for known `model_id` aliases. Explicit values override computed estimates.
    pub input_cost_usd: Option<f64>,
    pub output_cost_usd: Option<f64>,
    pub reasoning_cost_usd: Option<f64>,
    pub cached_cost_usd: Option<f64>,
    pub total_cost_usd: Option<f64>,
}

impl Default for ContextUsage {
    fn default() -> Self {
        Self {
            input_tokens: None,
            output_tokens: None,
            reasoning_tokens: None,
            cached_input_tokens: None,
            input_cost_usd: None,
            output_cost_usd: None,
            reasoning_cost_usd: None,
            cached_cost_usd: None,
            total_cost_usd: None,
        }
    }
}

#[derive(Debug, Clone)]
struct ContextSchema {
    used_tokens: u64,
    max_tokens: u64,
    usage: Option<ContextUsage>,
    model_id: Option<Arc<str>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct ContextUsageCosts {
    input_cost_usd: Option<f64>,
    output_cost_usd: Option<f64>,
    reasoning_cost_usd: Option<f64>,
    cached_cost_usd: Option<f64>,
    total_cost_usd: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct ContextModelPricing {
    input_per_million_usd: f64,
    cached_input_per_million_usd: Option<f64>,
    output_per_million_usd: f64,
    reasoning_per_million_usd: Option<f64>,
}

fn normalize_context_model_id(model_id: &str) -> Arc<str> {
    let lower = model_id.trim().to_ascii_lowercase();

    if let Some(rest) = lower.strip_prefix("openai-") {
        return Arc::<str>::from(format!("openai:{rest}"));
    }
    if let Some(rest) = lower.strip_prefix("anthropic-") {
        return Arc::<str>::from(format!("anthropic:{rest}"));
    }
    if let Some(rest) = lower.strip_prefix("google-") {
        return Arc::<str>::from(format!("google:{rest}"));
    }
    if let Some(rest) = lower.strip_prefix("google-vertex-") {
        return Arc::<str>::from(format!("google:{rest}"));
    }
    if let Some((provider, rest)) = lower.split_once('/') {
        return Arc::<str>::from(format!("{provider}:{rest}"));
    }

    Arc::<str>::from(lower)
}

fn built_in_context_model_pricing(model_id: &str) -> Option<ContextModelPricing> {
    let normalized = normalize_context_model_id(model_id);
    match normalized.as_ref() {
        "openai:gpt-5" => Some(ContextModelPricing {
            input_per_million_usd: 1.25,
            cached_input_per_million_usd: Some(0.125),
            output_per_million_usd: 10.0,
            reasoning_per_million_usd: Some(10.0),
        }),
        "openai:gpt-4.1-mini" => Some(ContextModelPricing {
            input_per_million_usd: 0.40,
            cached_input_per_million_usd: Some(0.10),
            output_per_million_usd: 1.60,
            reasoning_per_million_usd: Some(1.60),
        }),
        "openai:gpt-4o" => Some(ContextModelPricing {
            input_per_million_usd: 2.50,
            cached_input_per_million_usd: Some(1.25),
            output_per_million_usd: 10.0,
            reasoning_per_million_usd: Some(10.0),
        }),
        "openai:gpt-4o-mini" => Some(ContextModelPricing {
            input_per_million_usd: 0.15,
            cached_input_per_million_usd: Some(0.075),
            output_per_million_usd: 0.60,
            reasoning_per_million_usd: Some(0.60),
        }),
        "anthropic:claude-3-5-sonnet"
        | "anthropic:claude-3-5-sonnet-20241022"
        | "anthropic:claude-sonnet-4-5"
        | "anthropic:claude-sonnet-4"
        | "anthropic:claude-sonnet-4-20250514" => Some(ContextModelPricing {
            input_per_million_usd: 3.0,
            cached_input_per_million_usd: Some(0.30),
            output_per_million_usd: 15.0,
            reasoning_per_million_usd: Some(15.0),
        }),
        "anthropic:claude-opus-4-20250514"
        | "anthropic:claude-opus-4"
        | "anthropic:claude-opus-4-1-20250805" => Some(ContextModelPricing {
            input_per_million_usd: 15.0,
            cached_input_per_million_usd: Some(1.50),
            output_per_million_usd: 75.0,
            reasoning_per_million_usd: Some(75.0),
        }),
        "google:gemini-2-0-flash" => Some(ContextModelPricing {
            input_per_million_usd: 0.10,
            cached_input_per_million_usd: Some(0.025),
            output_per_million_usd: 0.40,
            reasoning_per_million_usd: Some(0.40),
        }),
        _ => None,
    }
}

fn cost_from_million_rate(tokens: Option<u64>, rate_per_million_usd: Option<f64>) -> Option<f64> {
    let tokens = tokens.unwrap_or(0);
    let rate_per_million_usd = rate_per_million_usd?;
    Some(tokens as f64 / 1_000_000.0 * rate_per_million_usd)
}

fn estimated_usage_costs(schema: &ContextSchema) -> Option<ContextUsageCosts> {
    let usage = schema.usage.as_ref()?;
    let pricing = built_in_context_model_pricing(schema.model_id.as_deref()?)?;

    let input_cost_usd =
        cost_from_million_rate(usage.input_tokens, Some(pricing.input_per_million_usd));
    let output_cost_usd =
        cost_from_million_rate(usage.output_tokens, Some(pricing.output_per_million_usd));
    let reasoning_cost_usd = cost_from_million_rate(
        usage.reasoning_tokens,
        pricing
            .reasoning_per_million_usd
            .or(Some(pricing.output_per_million_usd)),
    );
    let cached_cost_usd = cost_from_million_rate(
        usage.cached_input_tokens,
        pricing.cached_input_per_million_usd,
    );
    let total_cost_usd = Some(
        input_cost_usd.unwrap_or(0.0)
            + output_cost_usd.unwrap_or(0.0)
            + reasoning_cost_usd.unwrap_or(0.0)
            + cached_cost_usd.unwrap_or(0.0),
    );

    Some(ContextUsageCosts {
        input_cost_usd,
        output_cost_usd,
        reasoning_cost_usd,
        cached_cost_usd,
        total_cost_usd,
    })
}

fn resolved_usage_costs(schema: &ContextSchema) -> Option<ContextUsageCosts> {
    let usage = schema.usage.as_ref()?;
    let estimated = estimated_usage_costs(schema).unwrap_or_default();

    let input_cost_usd = usage.input_cost_usd.or(estimated.input_cost_usd);
    let output_cost_usd = usage.output_cost_usd.or(estimated.output_cost_usd);
    let reasoning_cost_usd = usage.reasoning_cost_usd.or(estimated.reasoning_cost_usd);
    let cached_cost_usd = usage.cached_cost_usd.or(estimated.cached_cost_usd);
    let total_cost_usd = usage.total_cost_usd.or_else(|| {
        Some(
            input_cost_usd.unwrap_or(0.0)
                + output_cost_usd.unwrap_or(0.0)
                + reasoning_cost_usd.unwrap_or(0.0)
                + cached_cost_usd.unwrap_or(0.0),
        )
    });

    Some(ContextUsageCosts {
        input_cost_usd,
        output_cost_usd,
        reasoning_cost_usd,
        cached_cost_usd,
        total_cost_usd,
    })
}

fn use_context_schema<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<ContextSchema> {
    cx.provided::<ContextSchema>().cloned()
}

fn hidden<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    cx.interactivity_gate_props(
        InteractivityGateProps {
            layout: LayoutStyle::default(),
            present: false,
            interactive: false,
        },
        |_cx| Vec::new(),
    )
}

const CONTEXT_TRIGGER_SLOT_KEY: &str = "__fret_ui_ai.context.trigger";
const CONTEXT_CONTENT_SLOT_KEY: &str = "__fret_ui_ai.context.content";
const CONTEXT_HEADER_SLOT_KEY: &str = "__fret_ui_ai.context.header";
const CONTEXT_FOOTER_SLOT_KEY: &str = "__fret_ui_ai.context.footer";
const CONTEXT_INPUT_SLOT_KEY: &str = "__fret_ui_ai.context.input";
const CONTEXT_OUTPUT_SLOT_KEY: &str = "__fret_ui_ai.context.output";
const CONTEXT_REASONING_SLOT_KEY: &str = "__fret_ui_ai.context.reasoning";
const CONTEXT_CACHE_SLOT_KEY: &str = "__fret_ui_ai.context.cache";

fn context_slot_placeholder<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    slot_key: &'static str,
    semantics: Option<SemanticsDecoration>,
    children: Vec<AnyElement>,
) -> AnyElement {
    let mut slot = cx.interactivity_gate_props(
        InteractivityGateProps {
            layout: LayoutStyle::default(),
            present: false,
            interactive: false,
        },
        move |_cx| children,
    );
    slot.key_context = Some(Arc::<str>::from(slot_key));
    if let Some(decoration) = semantics {
        slot = slot.attach_semantics(decoration);
    }
    slot
}

fn context_slot_test_id(element: &AnyElement) -> Option<Arc<str>> {
    element
        .semantics_decoration
        .as_ref()
        .and_then(|decoration| decoration.test_id.clone())
}

fn render_context_header_shell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let wrapper = ui::v_stack(move |_cx| children)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N2)
        .into_element(cx);

    let wrapper = cx.container(
        decl_style::container_props(
            &theme,
            ChromeRefinement::default().p(Space::N3),
            LayoutRefinement::default().w_full().min_w_0(),
        ),
        move |_cx| vec![wrapper],
    );

    if let Some(test_id) = test_id {
        wrapper.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    } else {
        wrapper
    }
}

fn render_context_footer_shell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let wrapper = cx.container(
        decl_style::container_props(
            &theme,
            ChromeRefinement::default()
                .p(Space::N3)
                .bg(ColorRef::Color(theme.color_token("secondary"))),
            LayoutRefinement::default().w_full().min_w_0(),
        ),
        move |_cx| children,
    );

    if let Some(test_id) = test_id {
        wrapper.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    } else {
        wrapper
    }
}

fn resolve_context_trigger_slot<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    mut slot: AnyElement,
) -> AnyElement {
    let test_id = context_slot_test_id(&slot);
    if let Some(child) = slot.children.drain(..).next() {
        return if let Some(test_id) = test_id {
            child.attach_semantics(SemanticsDecoration::default().test_id(test_id))
        } else {
            child
        };
    }

    let trigger = ContextTrigger::default();
    if let Some(test_id) = test_id {
        trigger.test_id(test_id).into_element(cx)
    } else {
        trigger.into_element(cx)
    }
}

fn resolve_context_content_subtree<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    mut element: AnyElement,
) -> Option<AnyElement> {
    match element.key_context.as_deref() {
        Some(CONTEXT_HEADER_SLOT_KEY) => {
            let header = ContextContentHeader::default();
            return Some(if let Some(test_id) = context_slot_test_id(&element) {
                header.test_id(test_id).into_element(cx)
            } else {
                header.into_element(cx)
            });
        }
        Some(CONTEXT_FOOTER_SLOT_KEY) => {
            let footer = ContextContentFooter::default();
            return Some(if let Some(test_id) = context_slot_test_id(&element) {
                footer.test_id(test_id).into_element(cx)
            } else {
                footer.into_element(cx)
            });
        }
        Some(CONTEXT_INPUT_SLOT_KEY) => {
            let row = ContextInputUsage::default();
            return Some(if let Some(test_id) = context_slot_test_id(&element) {
                row.test_id(test_id).into_element(cx)
            } else {
                row.into_element(cx)
            });
        }
        Some(CONTEXT_OUTPUT_SLOT_KEY) => {
            let row = ContextOutputUsage::default();
            return Some(if let Some(test_id) = context_slot_test_id(&element) {
                row.test_id(test_id).into_element(cx)
            } else {
                row.into_element(cx)
            });
        }
        Some(CONTEXT_REASONING_SLOT_KEY) => {
            let row = ContextReasoningUsage::default();
            return Some(if let Some(test_id) = context_slot_test_id(&element) {
                row.test_id(test_id).into_element(cx)
            } else {
                row.into_element(cx)
            });
        }
        Some(CONTEXT_CACHE_SLOT_KEY) => {
            let row = ContextCacheUsage::default();
            return Some(if let Some(test_id) = context_slot_test_id(&element) {
                row.test_id(test_id).into_element(cx)
            } else {
                row.into_element(cx)
            });
        }
        _ => {}
    }

    element.children = element
        .children
        .into_iter()
        .filter_map(|child| resolve_context_content_subtree(cx, child))
        .collect();
    Some(element)
}

fn resolve_context_root_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    children: Vec<AnyElement>,
) -> (Option<AnyElement>, Option<AnyElement>) {
    let mut trigger = None;
    let mut content = None;

    for child in children {
        match child.key_context.as_deref() {
            Some(CONTEXT_TRIGGER_SLOT_KEY) if trigger.is_none() => {
                trigger = Some(resolve_context_trigger_slot(cx, child));
            }
            Some(CONTEXT_CONTENT_SLOT_KEY) if content.is_none() => {
                content = resolve_context_content_subtree(cx, child);
            }
            _ => {}
        }
    }

    (trigger, content)
}

/// AI Elements-aligned context usage hovercard (`context.tsx`).
pub struct Context {
    used_tokens: u64,
    max_tokens: u64,
    model_id: Option<Arc<str>>,
    usage: Option<ContextUsage>,
    open_delay_frames: u32,
    close_delay_frames: u32,
    children: Vec<AnyElement>,
    trigger: Option<AnyElement>,
    content: Option<AnyElement>,
    test_id_root: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("used_tokens", &self.used_tokens)
            .field("max_tokens", &self.max_tokens)
            .field("model_id", &self.model_id.as_deref())
            .field("has_usage", &self.usage.is_some())
            .field("open_delay_frames", &self.open_delay_frames)
            .field("close_delay_frames", &self.close_delay_frames)
            .field("has_trigger", &self.trigger.is_some())
            .field("has_content", &self.content.is_some())
            .field("test_id_root", &self.test_id_root.as_deref())
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
            open_delay_frames: 0,
            close_delay_frames: 0,
            children: Vec::new(),
            trigger: None,
            content: None,
            test_id_root: None,
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

    pub fn open_delay_frames(mut self, frames: u32) -> Self {
        self.open_delay_frames = frames;
        self
    }

    pub fn close_delay_frames(mut self, frames: u32) -> Self {
        self.close_delay_frames = frames;
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = children.into_iter().collect();
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

    pub fn test_id_root(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element_with_children<H: UiHost>(
        self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> (AnyElement, AnyElement),
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let schema = ContextSchema {
            used_tokens: self.used_tokens,
            max_tokens: self.max_tokens,
            usage: self.usage.clone(),
            model_id: self.model_id.clone(),
        };

        let layout = self.layout;
        let open_delay_frames = self.open_delay_frames;
        let close_delay_frames = self.close_delay_frames;
        let trigger_override = self.trigger;
        let content_override = self.content;
        let test_id_root = self.test_id_root;

        let root = cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default(),
                LayoutRefinement::default(),
            ),
            move |cx| {
                cx.provide(schema.clone(), |cx| {
                    let (default_trigger, default_content) = children(cx);
                    let trigger = trigger_override.unwrap_or(default_trigger);
                    let content = content_override.unwrap_or(default_content);

                    let hover = HoverCard::new(cx, trigger, content)
                        .open_delay_frames(open_delay_frames)
                        .close_delay_frames(close_delay_frames)
                        .refine_layout(layout)
                        .into_element(cx);

                    let hover = match test_id_root {
                        Some(id) => hover.attach_semantics(
                            SemanticsDecoration::default()
                                .role(SemanticsRole::Group)
                                .test_id(id),
                        ),
                        None => hover,
                    };

                    vec![hover]
                })
            },
        );

        root
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        if !self.children.is_empty() {
            let theme = Theme::global(&*cx.app).clone();

            let schema = ContextSchema {
                used_tokens: self.used_tokens,
                max_tokens: self.max_tokens,
                usage: self.usage.clone(),
                model_id: self.model_id.clone(),
            };

            let layout = self.layout;
            let open_delay_frames = self.open_delay_frames;
            let close_delay_frames = self.close_delay_frames;
            let direct_children = self.children;
            let trigger_override = self.trigger;
            let content_override = self.content;
            let test_id_root = self.test_id_root;

            return cx.container(
                decl_style::container_props(
                    &theme,
                    ChromeRefinement::default(),
                    LayoutRefinement::default(),
                ),
                move |cx| {
                    cx.provide(schema.clone(), |cx| {
                        let (slot_trigger, slot_content) =
                            resolve_context_root_children(cx, direct_children);
                        let trigger = trigger_override
                            .or(slot_trigger)
                            .unwrap_or_else(|| ContextTrigger::default().into_element(cx));
                        let content = content_override.or(slot_content).unwrap_or_else(|| {
                            ContextContent::new([
                                ContextContentHeader::default().into_element(cx),
                                ContextContentBody::new([
                                    ContextInputUsage::default().into_element(cx),
                                    ContextOutputUsage::default().into_element(cx),
                                    ContextReasoningUsage::default().into_element(cx),
                                    ContextCacheUsage::default().into_element(cx),
                                ])
                                .into_element(cx),
                                ContextContentFooter::default().into_element(cx),
                            ])
                            .into_element(cx)
                        });

                        let hover = HoverCard::new(cx, trigger, content)
                            .open_delay_frames(open_delay_frames)
                            .close_delay_frames(close_delay_frames)
                            .refine_layout(layout)
                            .into_element(cx);

                        let hover = match test_id_root {
                            Some(id) => hover.attach_semantics(
                                SemanticsDecoration::default()
                                    .role(SemanticsRole::Group)
                                    .test_id(id),
                            ),
                            None => hover,
                        };

                        vec![hover]
                    })
                },
            );
        }

        self.into_element_with_children(cx, |cx| {
            let trigger = ContextTrigger::default().into_element(cx);
            let content = ContextContent::new([
                ContextContentHeader::default().into_element(cx),
                ContextContentBody::new([
                    ContextInputUsage::default().into_element(cx),
                    ContextOutputUsage::default().into_element(cx),
                    ContextReasoningUsage::default().into_element(cx),
                    ContextCacheUsage::default().into_element(cx),
                ])
                .into_element(cx),
                ContextContentFooter::default().into_element(cx),
            ])
            .into_element(cx);
            (trigger, content)
        })
    }
}

/// Trigger aligned with AI Elements `ContextTrigger`.
#[derive(Debug, Default)]
pub struct ContextTrigger {
    children: Option<AnyElement>,
    test_id: Option<Arc<str>>,
}

impl ContextTrigger {
    pub fn children(mut self, el: AnyElement) -> Self {
        self.children = Some(el);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let ContextTrigger { children, test_id } = self;

        if use_context_schema(cx).is_none() {
            let semantics = test_id.map(|id| SemanticsDecoration::default().test_id(id));
            return context_slot_placeholder(
                cx,
                CONTEXT_TRIGGER_SLOT_KEY,
                semantics,
                children.into_iter().collect(),
            );
        }

        if let Some(children) = children {
            return if let Some(test_id) = test_id {
                children.attach_semantics(SemanticsDecoration::default().test_id(test_id))
            } else {
                children
            };
        }

        let Some(schema) = use_context_schema(cx) else {
            return hidden(cx);
        };

        let theme = Theme::global(&*cx.app).clone();
        let pct = percent_used(schema.used_tokens, schema.max_tokens);

        let mut pct_style =
            typography::TypographyPreset::control_ui(typography::UiTextSize::Sm).resolve(&theme);
        pct_style.weight = FontWeight::MEDIUM;

        let pct_text = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: format_percent(pct),
            style: Some(pct_style),
            color: Some(theme.color_token("muted-foreground")),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: Default::default(),
        });

        let svg_bytes = context_icon_svg_bytes(schema.used_tokens, schema.max_tokens);
        let mut svg_props = SvgIconProps::new(SvgSource::Bytes(svg_bytes));
        svg_props.inherit_color = true;
        svg_props.layout.size.width = Length::Px(Px(20.0));
        svg_props.layout.size.height = Length::Px(Px(20.0));
        let icon = cx.svg_icon_props(svg_props);

        let btn = Button::new("Context")
            .children([pct_text, icon])
            .variant(ButtonVariant::Ghost)
            .into_element(cx);

        if let Some(test_id) = test_id {
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

/// Hover card content wrapper aligned with AI Elements `ContextContent`.
#[derive(Debug)]
pub struct ContextContent {
    children: Vec<AnyElement>,
    divide_y: bool,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl ContextContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            divide_y: true,
            test_id: None,
            layout: LayoutRefinement::default(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn divide_y(mut self, enabled: bool) -> Self {
        self.divide_y = enabled;
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

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let mut out: Vec<AnyElement> = Vec::new();
        if self.divide_y {
            for (idx, child) in self.children.into_iter().enumerate() {
                if idx > 0 {
                    out.push(Separator::new().into_element(cx));
                }
                out.push(child);
            }
        } else {
            out = self.children;
        }

        let content = HoverCardContent::new(out)
            .refine_style(ChromeRefinement::default().p(Space::N0).merge(self.chrome))
            .refine_layout(
                LayoutRefinement::default()
                    .min_w(Px(240.0))
                    .overflow_hidden()
                    .merge(self.layout),
            );

        let content = match self.test_id {
            Some(id) => content.test_id(id).into_element(cx),
            None => content.into_element(cx),
        };

        if use_context_schema(cx).is_some() {
            content
        } else {
            content.key_context(CONTEXT_CONTENT_SLOT_KEY)
        }
    }
}

/// Header aligned with AI Elements `ContextContentHeader`.
#[derive(Debug, Default)]
pub struct ContextContentHeader {
    children: Option<Vec<AnyElement>>,
    test_id: Option<Arc<str>>,
}

impl ContextContentHeader {
    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let ContextContentHeader { children, test_id } = self;

        let Some(schema) = use_context_schema(cx) else {
            return match children {
                Some(children) => render_context_header_shell(cx, children, test_id),
                None => context_slot_placeholder(
                    cx,
                    CONTEXT_HEADER_SLOT_KEY,
                    test_id.map(|id| SemanticsDecoration::default().test_id(id)),
                    Vec::new(),
                ),
            };
        };

        let theme = Theme::global(&*cx.app).clone();

        let children = children.unwrap_or_else(|| {
            let pct = percent_used(schema.used_tokens, schema.max_tokens);
            let display_pct = cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: format_percent(pct),
                style: Some(
                    typography::TypographyPreset::control_ui(typography::UiTextSize::Xs)
                        .resolve(&theme),
                ),
                color: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
            });

            let used = format_compact_u64(schema.used_tokens);
            let total = format_compact_u64(schema.max_tokens);
            let used_total = cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: Arc::<str>::from(format!("{used} / {total}")),
                style: Some(
                    typography::TypographyPreset::control_monospace(typography::UiTextSize::Xs)
                        .resolve(&theme),
                ),
                color: Some(theme.color_token("muted-foreground")),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
            });

            let header_row = ui::h_row(move |_cx| vec![display_pct, used_total])
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .items(Items::Center)
                .justify(Justify::Between)
                .gap(Space::N3)
                .into_element(cx);

            let progress_model = cx.local_model(|| pct * 100.0);

            let next = pct * 100.0;
            let _ = cx.app.models_mut().update(&progress_model, |v| {
                if (*v - next).abs() > f32::EPSILON {
                    *v = next;
                }
            });

            let progress = Progress::new(progress_model)
                .a11y_label("Context usage")
                .refine_style(
                    ChromeRefinement::default().bg(ColorRef::Color(theme.color_token("muted"))),
                )
                .into_element(cx);

            vec![header_row, progress]
        });

        let _ = theme;
        render_context_header_shell(cx, children, test_id)
    }
}

/// Body wrapper aligned with AI Elements `ContextContentBody`.
#[derive(Debug)]
pub struct ContextContentBody {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
}

impl ContextContentBody {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let body = ui::v_stack(move |_cx| self.children)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N2)
            .items(Items::Start)
            .into_element(cx);

        let wrapper = cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default().p(Space::N3),
                LayoutRefinement::default().w_full().min_w_0(),
            ),
            move |_cx| vec![body],
        );

        if let Some(test_id) = self.test_id {
            wrapper.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Group)
                    .test_id(test_id),
            )
        } else {
            wrapper
        }
    }
}

/// Footer aligned with AI Elements `ContextContentFooter`.
#[derive(Debug, Default)]
pub struct ContextContentFooter {
    children: Option<Vec<AnyElement>>,
    test_id: Option<Arc<str>>,
}

impl ContextContentFooter {
    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let ContextContentFooter { children, test_id } = self;

        let Some(schema) = use_context_schema(cx) else {
            return match children {
                Some(children) => render_context_footer_shell(cx, children, test_id),
                None => context_slot_placeholder(
                    cx,
                    CONTEXT_FOOTER_SLOT_KEY,
                    test_id.map(|id| SemanticsDecoration::default().test_id(id)),
                    Vec::new(),
                ),
            };
        };

        let theme = Theme::global(&*cx.app).clone();
        let resolved_costs = resolved_usage_costs(&schema);

        let children = children.unwrap_or_else(|| {
            let Some(total_cost) = resolved_costs.and_then(|costs| costs.total_cost_usd) else {
                return Vec::new();
            };

            let mut label_style =
                typography::TypographyPreset::control_ui(typography::UiTextSize::Xs)
                    .resolve(&theme);
            label_style.weight = FontWeight::NORMAL;

            let label = cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: Arc::<str>::from("Total cost"),
                style: Some(label_style),
                color: Some(theme.color_token("muted-foreground")),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                ink_overflow: Default::default(),
            });

            let value = cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: format_usd(total_cost),
                style: Some(
                    typography::TypographyPreset::control_ui(typography::UiTextSize::Xs)
                        .resolve(&theme),
                ),
                color: None,
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::End,
                ink_overflow: Default::default(),
            });

            let row = ui::h_row(move |_cx| vec![label, value])
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .items(Items::Center)
                .justify(Justify::Between)
                .gap(Space::N3)
                .into_element(cx);

            vec![row]
        });

        if children.is_empty() {
            return hidden(cx);
        }

        let _ = theme;
        render_context_footer_shell(cx, children, test_id)
    }
}

fn usage_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    tokens: u64,
    cost_usd: Option<f64>,
    show_zero_cost: bool,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let label_text = cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: Arc::<str>::from(label),
        style: Some(
            typography::TypographyPreset::control_ui(typography::UiTextSize::Xs).resolve(&theme),
        ),
        color: Some(theme.color_token("muted-foreground")),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,
        ink_overflow: Default::default(),
    });

    let token_text = cx.text_props(TextProps {
        layout: LayoutStyle::default(),
        text: format_compact_u64(tokens),
        style: Some(
            typography::TypographyPreset::control_ui(typography::UiTextSize::Xs).resolve(&theme),
        ),
        color: None,
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::End,
        ink_overflow: Default::default(),
    });

    let trailing = if cost_usd.is_some() || show_zero_cost {
        let cost = cost_usd.unwrap_or(0.0);
        let cost_text = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: Arc::<str>::from(format!("• {}", format_usd(cost))),
            style: Some(
                typography::TypographyPreset::control_ui(typography::UiTextSize::Xs)
                    .resolve(&theme),
            ),
            color: Some(theme.color_token("muted-foreground")),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
            align: fret_core::TextAlign::End,
            ink_overflow: Default::default(),
        });

        ui::h_row(move |_cx| vec![token_text, cost_text])
            .layout(LayoutRefinement::default().min_w_0())
            .items(Items::Center)
            .gap(Space::N2)
            .into_element(cx)
    } else {
        token_text
    };

    ui::h_row(move |_cx| vec![label_text, trailing])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .items(Items::Center)
        .justify(Justify::Between)
        .gap(Space::N3)
        .into_element(cx)
}

/// Usage row aligned with AI Elements `ContextInputUsage`.
#[derive(Debug, Default)]
pub struct ContextInputUsage {
    children: Option<AnyElement>,
    test_id: Option<Arc<str>>,
}

impl ContextInputUsage {
    pub fn children(mut self, el: AnyElement) -> Self {
        self.children = Some(el);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let ContextInputUsage { children, test_id } = self;

        if let Some(children) = children {
            return children;
        }

        let Some(schema) = use_context_schema(cx) else {
            return context_slot_placeholder(
                cx,
                CONTEXT_INPUT_SLOT_KEY,
                test_id.map(|id| SemanticsDecoration::default().test_id(id)),
                Vec::new(),
            );
        };

        let Some(ref usage) = schema.usage else {
            return hidden(cx);
        };
        let resolved_costs = resolved_usage_costs(&schema);

        let tokens = usage.input_tokens.unwrap_or(0);
        if tokens == 0 {
            return hidden(cx);
        }

        let el = usage_row(
            cx,
            "Input",
            tokens,
            resolved_costs.and_then(|costs| costs.input_cost_usd),
            resolved_costs.is_some(),
        );
        if let Some(test_id) = test_id {
            el.attach_semantics(SemanticsDecoration::default().test_id(test_id))
        } else {
            el
        }
    }
}

/// Usage row aligned with AI Elements `ContextOutputUsage`.
#[derive(Debug, Default)]
pub struct ContextOutputUsage {
    children: Option<AnyElement>,
    test_id: Option<Arc<str>>,
}

impl ContextOutputUsage {
    pub fn children(mut self, el: AnyElement) -> Self {
        self.children = Some(el);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let ContextOutputUsage { children, test_id } = self;

        if let Some(children) = children {
            return children;
        }

        let Some(schema) = use_context_schema(cx) else {
            return context_slot_placeholder(
                cx,
                CONTEXT_OUTPUT_SLOT_KEY,
                test_id.map(|id| SemanticsDecoration::default().test_id(id)),
                Vec::new(),
            );
        };
        let Some(ref usage) = schema.usage else {
            return hidden(cx);
        };
        let resolved_costs = resolved_usage_costs(&schema);

        let tokens = usage.output_tokens.unwrap_or(0);
        if tokens == 0 {
            return hidden(cx);
        }

        let el = usage_row(
            cx,
            "Output",
            tokens,
            resolved_costs.and_then(|costs| costs.output_cost_usd),
            resolved_costs.is_some(),
        );
        if let Some(test_id) = test_id {
            el.attach_semantics(SemanticsDecoration::default().test_id(test_id))
        } else {
            el
        }
    }
}

/// Usage row aligned with AI Elements `ContextReasoningUsage`.
#[derive(Debug, Default)]
pub struct ContextReasoningUsage {
    children: Option<AnyElement>,
    test_id: Option<Arc<str>>,
}

impl ContextReasoningUsage {
    pub fn children(mut self, el: AnyElement) -> Self {
        self.children = Some(el);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let ContextReasoningUsage { children, test_id } = self;

        if let Some(children) = children {
            return children;
        }

        let Some(schema) = use_context_schema(cx) else {
            return context_slot_placeholder(
                cx,
                CONTEXT_REASONING_SLOT_KEY,
                test_id.map(|id| SemanticsDecoration::default().test_id(id)),
                Vec::new(),
            );
        };
        let Some(ref usage) = schema.usage else {
            return hidden(cx);
        };
        let resolved_costs = resolved_usage_costs(&schema);

        let tokens = usage.reasoning_tokens.unwrap_or(0);
        if tokens == 0 {
            return hidden(cx);
        }

        let el = usage_row(
            cx,
            "Reasoning",
            tokens,
            resolved_costs.and_then(|costs| costs.reasoning_cost_usd),
            resolved_costs.is_some(),
        );
        if let Some(test_id) = test_id {
            el.attach_semantics(SemanticsDecoration::default().test_id(test_id))
        } else {
            el
        }
    }
}

/// Usage row aligned with AI Elements `ContextCacheUsage`.
#[derive(Debug, Default)]
pub struct ContextCacheUsage {
    children: Option<AnyElement>,
    test_id: Option<Arc<str>>,
}

impl ContextCacheUsage {
    pub fn children(mut self, el: AnyElement) -> Self {
        self.children = Some(el);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let ContextCacheUsage { children, test_id } = self;

        if let Some(children) = children {
            return children;
        }

        let Some(schema) = use_context_schema(cx) else {
            return context_slot_placeholder(
                cx,
                CONTEXT_CACHE_SLOT_KEY,
                test_id.map(|id| SemanticsDecoration::default().test_id(id)),
                Vec::new(),
            );
        };
        let Some(ref usage) = schema.usage else {
            return hidden(cx);
        };
        let resolved_costs = resolved_usage_costs(&schema);

        let tokens = usage.cached_input_tokens.unwrap_or(0);
        if tokens == 0 {
            return hidden(cx);
        }

        let el = usage_row(
            cx,
            "Cache",
            tokens,
            resolved_costs.and_then(|costs| costs.cached_cost_usd),
            resolved_costs.is_some(),
        );
        if let Some(test_id) = test_id {
            el.attach_semantics(SemanticsDecoration::default().test_id(test_id))
        } else {
            el
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::ElementKind;

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(600.0), Px(400.0)),
        )
    }

    fn assert_approx_eq(actual: Option<f64>, expected: f64) {
        let actual = actual.expect("expected a computed cost");
        let delta = (actual - expected).abs();
        assert!(
            delta < 0.000_001,
            "expected {expected}, got {actual} (delta={delta})"
        );
    }

    fn has_test_id(element: &AnyElement, expected: &str) -> bool {
        element
            .semantics_decoration
            .as_ref()
            .and_then(|decoration| decoration.test_id.as_deref())
            .map(|test_id| test_id == expected)
            .unwrap_or(false)
            || element
                .children
                .iter()
                .any(|child| has_test_id(child, expected))
    }

    #[test]
    fn context_content_inserts_separators_by_default() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let root =
                ContextContent::new([cx.text("a"), cx.text("b"), cx.text("c")]).into_element(cx);

            let ElementKind::Container(_) = &root.kind else {
                panic!(
                    "expected ContextContent root to be a Container, got {:?}",
                    root.kind
                );
            };

            let Some(inner_container) = root.children.first() else {
                panic!("expected ContextContent root container to have one child");
            };

            let ElementKind::Container(_) = &inner_container.kind else {
                panic!(
                    "expected ContextContent first child to be a Container, got {:?}",
                    inner_container.kind
                );
            };

            let Some(column) = inner_container.children.first() else {
                panic!("expected inner container to contain the content column");
            };

            match &column.kind {
                ElementKind::Column(_) => {}
                ElementKind::Flex(props) if props.direction == fret_core::Axis::Vertical => {}
                other => {
                    panic!(
                        "expected ContextContent innermost child to be a vertical stack, got {:?}",
                        other
                    );
                }
            }

            assert_eq!(
                column.children.len(),
                5,
                "expected ContextContent to interleave separators between children"
            );
        });
    }

    #[test]
    fn compact_token_formatting_matches_ai_elements_style() {
        assert_eq!(&*format_compact_u64(842), "842");
        assert_eq!(&*format_compact_u64(19_134), "19.1K");
        assert_eq!(&*format_compact_u64(100_000), "100K");
        assert_eq!(&*format_compact_u64(1_500_000), "1.5M");
        assert_eq!(&*format_compact_u64(2_100_000_000), "2.1B");
    }

    #[test]
    fn resolved_usage_costs_estimate_known_model_aliases() {
        let schema = ContextSchema {
            used_tokens: 42_560,
            max_tokens: 128_000,
            model_id: Some(Arc::<str>::from("openai:gpt-5")),
            usage: Some(ContextUsage {
                input_tokens: Some(32_000),
                output_tokens: Some(8_000),
                reasoning_tokens: Some(512),
                cached_input_tokens: Some(2_048),
                ..Default::default()
            }),
        };

        let costs = resolved_usage_costs(&schema).expect("resolved costs");
        assert_approx_eq(costs.input_cost_usd, 0.04);
        assert_approx_eq(costs.output_cost_usd, 0.08);
        assert_approx_eq(costs.reasoning_cost_usd, 0.005_12);
        assert_approx_eq(costs.cached_cost_usd, 0.000_256);
        assert_approx_eq(costs.total_cost_usd, 0.125_376);
    }

    #[test]
    fn resolved_usage_costs_prefer_explicit_values_over_auto_estimates() {
        let schema = ContextSchema {
            used_tokens: 40_000,
            max_tokens: 128_000,
            model_id: Some(Arc::<str>::from("openai-gpt-4o")),
            usage: Some(ContextUsage {
                input_tokens: Some(32_000),
                output_tokens: Some(8_000),
                input_cost_usd: Some(9.99),
                total_cost_usd: Some(12.34),
                ..Default::default()
            }),
        };

        let costs = resolved_usage_costs(&schema).expect("resolved costs");
        assert_approx_eq(costs.input_cost_usd, 9.99);
        assert_approx_eq(costs.output_cost_usd, 0.08);
        assert_approx_eq(costs.total_cost_usd, 12.34);
    }

    #[test]
    fn context_direct_children_resolve_deferred_parts() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let direct_children = vec![
                ContextTrigger::default()
                    .test_id("context-trigger")
                    .into_element(cx),
                ContextContent::new([
                    ContextContentHeader::default()
                        .test_id("context-header")
                        .into_element(cx),
                    ContextContentBody::new([
                        ContextInputUsage::default()
                            .test_id("context-input")
                            .into_element(cx),
                        ContextOutputUsage::default()
                            .test_id("context-output")
                            .into_element(cx),
                        ContextReasoningUsage::default()
                            .test_id("context-reasoning")
                            .into_element(cx),
                        ContextCacheUsage::default()
                            .test_id("context-cache")
                            .into_element(cx),
                    ])
                    .test_id("context-body")
                    .into_element(cx),
                    ContextContentFooter::default()
                        .test_id("context-footer")
                        .into_element(cx),
                ])
                .test_id("context-content")
                .into_element(cx),
            ];

            cx.provide(
                ContextSchema {
                    used_tokens: 42_560,
                    max_tokens: 128_000,
                    model_id: Some(Arc::<str>::from("openai:gpt-5")),
                    usage: Some(ContextUsage {
                        input_tokens: Some(32_000),
                        output_tokens: Some(8_000),
                        reasoning_tokens: Some(512),
                        cached_input_tokens: Some(2_048),
                        ..Default::default()
                    }),
                },
                |cx| {
                    let (trigger, content) = resolve_context_root_children(cx, direct_children);
                    let trigger = trigger.expect("resolved trigger");
                    let content = content.expect("resolved content");

                    assert!(has_test_id(&trigger, "context-trigger"));
                    assert!(has_test_id(&content, "context-content"));
                    assert!(has_test_id(&content, "context-header"));
                    assert!(has_test_id(&content, "context-body"));
                    assert!(has_test_id(&content, "context-footer"));
                    assert!(has_test_id(&content, "context-input"));
                    assert!(has_test_id(&content, "context-output"));
                    assert!(has_test_id(&content, "context-reasoning"));
                    assert!(has_test_id(&content, "context-cache"));
                },
            );
        });
    }

    #[test]
    fn context_body_renders_without_provider_scope() {
        let window = AppWindowId::default();
        let mut app = App::new();

        fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
            let body = ContextContentBody::new([cx.text("Body content")])
                .test_id("context-body")
                .into_element(cx);

            assert!(has_test_id(&body, "context-body"));
            assert!(!matches!(body.kind, ElementKind::InteractivityGate(_)));
        });
    }
}
