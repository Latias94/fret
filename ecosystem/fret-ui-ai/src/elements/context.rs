//! AI Elements-aligned `Context` surfaces.
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/context.tsx`.

use std::sync::Arc;

use fret_core::{FontWeight, Px, SemanticsRole, TextOverflow, TextWrap};
use fret_runtime::Model;
use fret_ui::SvgSource;
use fret_ui::element::{
    AnyElement, InteractivityGateProps, LayoutStyle, Length, SemanticsDecoration, SvgIconProps,
    TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::{ChromeRefinement, ColorRef, Items, Justify, LayoutRefinement, Space};
use fret_ui_shadcn::{Button, ButtonVariant, HoverCard, HoverCardContent, Progress, Separator};

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
    /// Optional precomputed USD costs (app-owned). When omitted, the UI renders `$0.00`.
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

#[derive(Debug, Default, Clone)]
struct ContextProviderState {
    schema: Option<ContextSchema>,
}

fn use_context_schema<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<ContextSchema> {
    cx.inherited_state::<ContextProviderState>()
        .and_then(|st| st.schema.clone())
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

/// AI Elements-aligned context usage hovercard (`context.tsx`).
pub struct Context {
    used_tokens: u64,
    max_tokens: u64,
    model_id: Option<Arc<str>>,
    usage: Option<ContextUsage>,
    open_delay_frames: u32,
    close_delay_frames: u32,
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

    pub fn into_element_with_children<H: UiHost + 'static>(
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
                cx.with_state(ContextProviderState::default, |st| {
                    st.schema = Some(schema.clone());
                });

                let (default_trigger, default_content) = children(cx);
                let trigger = trigger_override.unwrap_or(default_trigger);
                let content = content_override.unwrap_or(default_content);

                let hover = HoverCard::new(trigger, content)
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
            },
        );

        root
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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
        if let Some(children) = self.children {
            return if let Some(test_id) = self.test_id {
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

        match self.test_id {
            Some(id) => content.test_id(id).into_element(cx),
            None => content.into_element(cx),
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let Some(schema) = use_context_schema(cx) else {
            return hidden(cx);
        };

        let theme = Theme::global(&*cx.app).clone();

        let children = self.children.unwrap_or_else(|| {
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

            let header_row = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .items_center()
                    .justify(Justify::Between)
                    .gap(Space::N3),
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

            let progress = Progress::new(progress_model)
                .a11y_label("Context usage")
                .refine_style(
                    ChromeRefinement::default().bg(ColorRef::Color(theme.color_token("muted"))),
                )
                .into_element(cx);

            vec![header_row, progress]
        });

        let wrapper = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N2),
            move |_cx| children,
        );

        let wrapper = cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default().p(Space::N3),
                LayoutRefinement::default().w_full().min_w_0(),
            ),
            move |_cx| vec![wrapper],
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
        let Some(_schema) = use_context_schema(cx) else {
            return hidden(cx);
        };

        let theme = Theme::global(&*cx.app).clone();

        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N2)
                .items(Items::Start),
            move |_cx| self.children,
        );

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
        let Some(schema) = use_context_schema(cx) else {
            return hidden(cx);
        };

        let theme = Theme::global(&*cx.app).clone();

        let children = self.children.unwrap_or_else(|| {
            if schema.model_id.is_none()
                && schema
                    .usage
                    .as_ref()
                    .and_then(|u| u.total_cost_usd)
                    .is_none()
            {
                return Vec::new();
            }

            let total_cost = schema
                .usage
                .as_ref()
                .and_then(|u| u.total_cost_usd)
                .unwrap_or(0.0);

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

            let row = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .items_center()
                    .justify(Justify::Between)
                    .gap(Space::N3),
                move |_cx| vec![label, value],
            );

            vec![row]
        });

        if children.is_empty() {
            return hidden(cx);
        }

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

        stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().min_w_0())
                .items_center()
                .gap(Space::N2),
            move |_cx| vec![token_text, cost_text],
        )
    } else {
        token_text
    };

    stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .items_center()
            .justify(Justify::Between)
            .gap(Space::N3),
        move |_cx| vec![label_text, trailing],
    )
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
        if let Some(children) = self.children {
            return children;
        }

        let Some(schema) = use_context_schema(cx) else {
            return hidden(cx);
        };
        let Some(usage) = schema.usage else {
            return hidden(cx);
        };

        let tokens = usage.input_tokens.unwrap_or(0);
        if tokens == 0 {
            return hidden(cx);
        }

        let el = usage_row(
            cx,
            "Input",
            tokens,
            usage.input_cost_usd,
            schema.model_id.is_some(),
        );
        if let Some(test_id) = self.test_id {
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
        if let Some(children) = self.children {
            return children;
        }

        let Some(schema) = use_context_schema(cx) else {
            return hidden(cx);
        };
        let Some(usage) = schema.usage else {
            return hidden(cx);
        };

        let tokens = usage.output_tokens.unwrap_or(0);
        if tokens == 0 {
            return hidden(cx);
        }

        let el = usage_row(
            cx,
            "Output",
            tokens,
            usage.output_cost_usd,
            schema.model_id.is_some(),
        );
        if let Some(test_id) = self.test_id {
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
        if let Some(children) = self.children {
            return children;
        }

        let Some(schema) = use_context_schema(cx) else {
            return hidden(cx);
        };
        let Some(usage) = schema.usage else {
            return hidden(cx);
        };

        let tokens = usage.reasoning_tokens.unwrap_or(0);
        if tokens == 0 {
            return hidden(cx);
        }

        let el = usage_row(
            cx,
            "Reasoning",
            tokens,
            usage.reasoning_cost_usd,
            schema.model_id.is_some(),
        );
        if let Some(test_id) = self.test_id {
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
        if let Some(children) = self.children {
            return children;
        }

        let Some(schema) = use_context_schema(cx) else {
            return hidden(cx);
        };
        let Some(usage) = schema.usage else {
            return hidden(cx);
        };

        let tokens = usage.cached_input_tokens.unwrap_or(0);
        if tokens == 0 {
            return hidden(cx);
        }

        let el = usage_row(
            cx,
            "Cache",
            tokens,
            usage.cached_cost_usd,
            schema.model_id.is_some(),
        );
        if let Some(test_id) = self.test_id {
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

            let Some(column) = root.children.first() else {
                panic!("expected ContextContent root container to have one child");
            };

            let ElementKind::Column(_) = &column.kind else {
                panic!(
                    "expected ContextContent inner to be a Column, got {:?}",
                    column.kind
                );
            };

            assert_eq!(
                column.children.len(),
                5,
                "expected ContextContent to interleave separators between children"
            );
        });
    }
}
