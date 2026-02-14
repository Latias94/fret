use std::sync::Arc;

use fret_core::{Color, FontId, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_icons::IconId;
use fret_ui::element::{AnyElement, LayoutStyle, SemanticsDecoration, SemanticsProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, LayoutRefinement, Radius, Space,
};
use fret_ui_shadcn::{
    Accordion, AccordionContent, AccordionItem, AccordionTrigger, Badge, BadgeVariant,
};

use crate::elements::CodeBlock;

fn token_color_with_alpha(
    theme: &Theme,
    key: &'static str,
    fallback_key: &'static str,
    alpha: f32,
) -> Color {
    let base = theme
        .color_by_key(key)
        .or_else(|| theme.color_by_key(fallback_key))
        .unwrap_or_else(|| theme.color_required("foreground"));
    let alpha = alpha.clamp(0.0, 1.0);
    Color {
        r: base.r,
        g: base.g,
        b: base.b,
        a: base.a * alpha,
    }
}

fn text_sm_style(theme: &Theme, weight: FontWeight, monospace: bool) -> TextStyle {
    let size = theme
        .metric_by_key("component.text.sm_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.text.sm_line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_required("font.line_height"));
    TextStyle {
        font: if monospace {
            FontId::monospace()
        } else {
            Default::default()
        },
        size,
        weight,
        slant: Default::default(),
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

fn muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_required("foreground"))
}

/// Agent root container aligned with AI Elements `agent.tsx`.
#[derive(Clone)]
pub struct Agent {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for Agent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Agent")
            .field("children_len", &self.children.len())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl Agent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
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

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default()
                .rounded(Radius::Md)
                .border_1()
                .border_color(ColorRef::Token {
                    key: "border",
                    fallback: ColorFallback::ThemePanelBorder,
                })
                .merge(self.chrome),
            self.layout,
        );

        let children = self.children;
        let body = cx.container(props, move |_cx| children);

        let Some(test_id) = self.test_id else {
            return body;
        };
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| [body],
        )
    }
}

/// Agent header row (bot icon + name + optional model badge).
#[derive(Clone)]
pub struct AgentHeader {
    name: Arc<str>,
    model: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for AgentHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentHeader")
            .field("name", &self.name.as_ref())
            .field("model", &self.model.as_deref())
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl AgentHeader {
    pub fn new(name: impl Into<Arc<str>>) -> Self {
        Self {
            name: name.into(),
            model: None,
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn model(mut self, model: impl Into<Arc<str>>) -> Self {
        self.model = Some(model.into());
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
        let theme = Theme::global(&*cx.app).clone();
        let muted = muted_fg(&theme);
        let name = self.name.clone();
        let model = self.model.clone();

        let bot_icon = decl_icon::icon_with(
            cx,
            IconId::new_static("lucide.bot"),
            Some(Px(16.0)),
            Some(ColorRef::Color(muted)),
        );

        let name_text = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: name,
            style: Some(text_sm_style(&theme, FontWeight::MEDIUM, false)),
            color: Some(theme.color_required("foreground")),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        });

        let model_badge = model.map(|m| {
            Badge::new(m)
                .variant(BadgeVariant::Secondary)
                .refine_style(ChromeRefinement::default().rounded(Radius::Full))
                .into_element(cx)
        });

        let left = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items(Items::Center),
            move |_cx| {
                let mut out = vec![bot_icon, name_text];
                if let Some(badge) = model_badge {
                    out.push(badge);
                }
                out
            },
        );

        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default().p(Space::N3).merge(self.chrome),
            self.layout,
        );

        let row = cx.container(props, move |_cx| [left]);

        let Some(test_id) = self.test_id else {
            return row;
        };
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Group,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| [row],
        )
    }
}

/// Agent content wrapper (`p-4 pt-0 space-y-4`).
#[derive(Clone)]
pub struct AgentContent {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for AgentContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentContent")
            .field("children_len", &self.children.len())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl AgentContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
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
        let theme = Theme::global(&*cx.app).clone();
        let children = self.children;
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N4)
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |_cx| children,
        );

        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default()
                .p(Space::N4)
                .pt(Space::N0)
                .merge(self.chrome),
            self.layout,
        );
        cx.container(props, move |_cx| [body])
    }
}

/// `Instructions` section (label + muted card).
#[derive(Clone)]
pub struct AgentInstructions {
    text: Arc<str>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for AgentInstructions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentInstructions")
            .field("text_len", &self.text.len())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl AgentInstructions {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
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
        let theme = Theme::global(&*cx.app).clone();
        let muted = muted_fg(&theme);

        let label = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: Arc::from("Instructions"),
            style: Some(text_sm_style(&theme, FontWeight::MEDIUM, false)),
            color: Some(muted),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        });

        let body_text = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: self.text,
            style: Some(text_sm_style(&theme, FontWeight::NORMAL, false)),
            color: Some(muted),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        });

        let bg = token_color_with_alpha(&theme, "muted", "accent", 0.5);
        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(bg))
                .p(Space::N3)
                .merge(self.chrome),
            self.layout,
        );
        let card = cx.container(props, move |_cx| [body_text]);

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |_cx| vec![label, card],
        )
    }
}

/// `Tools` section wrapper (label + bordered accordion).
#[derive(Clone)]
pub struct AgentTools {
    accordion: Accordion,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for AgentTools {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentTools")
            .field("accordion", &"<accordion>")
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl AgentTools {
    pub fn new(accordion: Accordion) -> Self {
        Self {
            accordion,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
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
        let theme = Theme::global(&*cx.app).clone();
        let muted = muted_fg(&theme);

        let label = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: Arc::from("Tools"),
            style: Some(text_sm_style(&theme, FontWeight::MEDIUM, false)),
            color: Some(muted),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        });

        let accordion = self.accordion.into_element(cx);
        let bordered = {
            let props = decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .rounded(Radius::Md)
                    .border_1()
                    .border_color(ColorRef::Token {
                        key: "border",
                        fallback: ColorFallback::ThemePanelBorder,
                    })
                    .merge(self.chrome),
                self.layout,
            );
            cx.container(props, move |_cx| [accordion])
        };

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |_cx| vec![label, bordered],
        )
    }
}

/// Minimal tool definition surface for `AgentTool`.
#[derive(Debug, Clone)]
pub struct AgentToolDefinition {
    pub description: Option<Arc<str>>,
    pub input_schema: serde_json::Value,
    pub json_schema: Option<serde_json::Value>,
}

impl AgentToolDefinition {
    pub fn schema_json(&self) -> &serde_json::Value {
        self.json_schema.as_ref().unwrap_or(&self.input_schema)
    }
}

/// A single tool disclosure item (accordion row + schema code block).
#[derive(Debug, Clone)]
pub struct AgentTool {
    value: Arc<str>,
    tool: AgentToolDefinition,
    trigger_test_id: Option<Arc<str>>,
}

impl AgentTool {
    pub fn new(value: impl Into<Arc<str>>, tool: AgentToolDefinition) -> Self {
        Self {
            value: value.into(),
            tool,
            trigger_test_id: None,
        }
    }

    pub fn trigger_test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.trigger_test_id = Some(id.into());
        self
    }

    pub fn into_item<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AccordionItem {
        let theme = Theme::global(&*cx.app).clone();
        let value = self.value;
        let desc = self
            .tool
            .description
            .clone()
            .unwrap_or_else(|| Arc::from("No description"));
        let schema = self.tool.schema_json().clone();

        let mut desc_el = cx.text(desc);
        if let Some(test_id) = self.trigger_test_id {
            desc_el = desc_el.attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Generic)
                    .test_id(test_id),
            );
        }

        let trigger = AccordionTrigger::new(vec![desc_el])
            .refine_style(ChromeRefinement::default().px(Space::N3).py(Space::N2))
            .refine_layout(LayoutRefinement::default().w_full().min_w_0());

        let pretty = serde_json::to_string_pretty(&schema).unwrap_or_else(|_| schema.to_string());
        let code = CodeBlock::new(Arc::<str>::from(pretty))
            .language("json")
            .show_header(false)
            .show_language(false)
            .into_element(cx);
        let bg = token_color_with_alpha(&theme, "muted", "accent", 0.5);
        let code_card = {
            let props = decl_style::container_props(
                &theme,
                ChromeRefinement::default()
                    .rounded(Radius::Md)
                    .bg(ColorRef::Color(bg)),
                LayoutRefinement::default().w_full().min_w_0(),
            );
            cx.container(props, move |_cx| [code])
        };

        let content = AccordionContent::new(vec![code_card])
            .refine_style(ChromeRefinement::default().px(Space::N3).pb(Space::N3));

        AccordionItem::new(value, trigger, content)
    }
}

/// `Output Schema` section.
#[derive(Clone)]
pub struct AgentOutput {
    schema: Arc<str>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for AgentOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AgentOutput")
            .field("schema_len", &self.schema.len())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl AgentOutput {
    pub fn new(schema: impl Into<Arc<str>>) -> Self {
        Self {
            schema: schema.into(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let muted = muted_fg(&theme);

        let label = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: Arc::from("Output Schema"),
            style: Some(text_sm_style(&theme, FontWeight::MEDIUM, false)),
            color: Some(muted),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        });

        let code = CodeBlock::new(self.schema)
            .language("typescript")
            .show_header(false)
            .show_language(false)
            .into_element(cx);
        let bg = token_color_with_alpha(&theme, "muted", "accent", 0.5);
        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default()
                .rounded(Radius::Md)
                .bg(ColorRef::Color(bg))
                .merge(self.chrome),
            self.layout,
        );
        let card = cx.container(props, move |_cx| [code]);

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N2)
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |_cx| vec![label, card],
        )
    }
}
