use std::sync::Arc;

use fret_core::{Color, FontWeight, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_icons::{IconId, ids};
use fret_ui::element::{AnyElement, LayoutStyle, SemanticsProps, TextProps};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Justify, LayoutRefinement, Radius, Space,
    WidgetStateProperty, WidgetStates,
};
use fret_ui_shadcn::{Badge, Collapsible, CollapsibleContent, CollapsibleTrigger};
use fret_ui_shadcn::{TabsContent, TabsList, TabsRoot};

use crate::elements::ToolStatus;

fn muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_required("foreground"))
}

fn text_sm_style(theme: &Theme, weight: FontWeight) -> TextStyle {
    let size = theme
        .metric_by_key("component.text.sm_px")
        .or_else(|| theme.metric_by_key("font.size"))
        .unwrap_or_else(|| theme.metric_required("font.size"));
    let line_height = theme
        .metric_by_key("component.text.sm_line_height")
        .or_else(|| theme.metric_by_key("font.line_height"))
        .unwrap_or_else(|| theme.metric_required("font.line_height"));
    TextStyle {
        font: Default::default(),
        size,
        weight,
        slant: Default::default(),
        line_height: Some(line_height),
        letter_spacing_em: None,
    }
}

fn tool_status_badge<H: UiHost>(cx: &mut ElementContext<'_, H>, status: ToolStatus) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    Badge::new(status.label())
        .variant(status.badge_variant())
        .refine_style(ChromeRefinement::default().rounded(Radius::Full))
        .children([decl_icon::icon_with(
            cx,
            status.icon_id(),
            Some(Px(16.0)),
            status.icon_color(&theme).map(ColorRef::Color),
        )])
        .into_element(cx)
}

/// Sandbox disclosure root aligned with AI Elements `sandbox.tsx`.
#[derive(Debug, Clone)]
pub struct Sandbox {
    default_open: bool,
    header: SandboxHeader,
    content: SandboxContent,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl Sandbox {
    pub fn new(header: SandboxHeader, content: SandboxContent) -> Self {
        Self {
            default_open: true,
            header,
            content,
            layout: LayoutRefinement::default()
                .w_full()
                .mb(Space::N4)
                .overflow_hidden(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
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

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let base_chrome = ChromeRefinement::default()
            .rounded_md()
            .border_1()
            .border_color(ColorRef::Token {
                key: "border",
                fallback: ColorFallback::ThemePanelBorder,
            });

        let header = self.header;
        let content = self.content;

        Collapsible::uncontrolled(self.default_open)
            .refine_layout(self.layout)
            .refine_style(base_chrome.merge(self.chrome))
            .into_element_with_open_model(
                cx,
                move |cx, open_model, is_open| header.clone().into_trigger(cx, open_model, is_open),
                move |cx| content.clone().into_element(cx),
            )
    }
}

/// Sandbox header row (`CollapsibleTrigger`).
#[derive(Clone)]
pub struct SandboxHeader {
    title: Option<Arc<str>>,
    status: ToolStatus,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for SandboxHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SandboxHeader")
            .field("title", &self.title.as_deref())
            .field("status", &self.status)
            .field("test_id", &self.test_id.as_deref())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl SandboxHeader {
    pub fn new(status: ToolStatus) -> Self {
        Self {
            title: None,
            status,
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn title(mut self, title: impl Into<Arc<str>>) -> Self {
        self.title = Some(title.into());
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

    fn into_trigger<H: UiHost + 'static>(
        self,
        cx: &mut ElementContext<'_, H>,
        open_model: fret_runtime::Model<bool>,
        is_open: bool,
    ) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let muted = muted_fg(&theme);

        let label = self.title.unwrap_or_else(|| Arc::from("Sandbox"));
        let code_icon = decl_icon::icon_with(
            cx,
            IconId::new_static("lucide.code"),
            Some(Px(16.0)),
            Some(ColorRef::Color(muted)),
        );

        let label_text = cx.text_props(TextProps {
            layout: LayoutStyle::default(),
            text: label,
            style: Some(text_sm_style(&theme, FontWeight::MEDIUM)),
            color: Some(theme.color_required("foreground")),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        });

        let status_badge = tool_status_badge(cx, self.status);

        let left = stack::hstack(
            cx,
            stack::HStackProps::default().gap(Space::N2),
            move |_cx| vec![code_icon, label_text, status_badge],
        );

        let chevron = decl_icon::icon_with(
            cx,
            if is_open {
                ids::ui::CHEVRON_UP
            } else {
                ids::ui::CHEVRON_DOWN
            },
            Some(Px(16.0)),
            Some(ColorRef::Color(muted)),
        );

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .justify(Justify::Between)
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |_cx| vec![left, chevron],
        );

        let trigger_row = cx.container(
            decl_style::container_props(
                &theme,
                ChromeRefinement::default().p(Space::N3).merge(self.chrome),
                self.layout,
            ),
            move |_cx| [row],
        );

        let trigger = CollapsibleTrigger::new(open_model, vec![trigger_row])
            .a11y_label("Toggle sandbox details")
            .into_element(cx, is_open);

        let Some(test_id) = self.test_id else {
            return trigger;
        };
        cx.semantics(
            SemanticsProps {
                role: SemanticsRole::Button,
                test_id: Some(test_id),
                ..Default::default()
            },
            move |_cx| [trigger],
        )
    }
}

/// Sandbox body wrapper (`CollapsibleContent`).
#[derive(Clone)]
pub struct SandboxContent {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl std::fmt::Debug for SandboxContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SandboxContent")
            .field("children_len", &self.children.len())
            .field("layout", &self.layout)
            .field("chrome", &self.chrome)
            .finish()
    }
}

impl SandboxContent {
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

    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = self.children;
        let body = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap(Space::N0)
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |_cx| children,
        );

        CollapsibleContent::new([body])
            .refine_layout(self.layout)
            .refine_style(self.chrome)
            .into_element(cx)
    }
}

/// A tabs surface pre-styled to match AI Elements `sandbox.tsx`.
///
/// Note: Fret's shadcn `Tabs` currently hardcodes the list background to `muted`. We still align
/// trigger border/active styling here; list background parity can be tightened later if needed.
#[derive(Debug, Clone)]
pub struct SandboxTabs {
    inner: TabsRoot,
}

impl SandboxTabs {
    pub fn uncontrolled<T: Into<Arc<str>>>(default_value: Option<T>) -> Self {
        let primary = ColorRef::Token {
            key: "primary",
            fallback: ColorFallback::ThemeAccent,
        };
        let line_style = fret_ui_shadcn::tabs::TabsStyle::default()
            .trigger_background(WidgetStateProperty::new(Some(ColorRef::Color(
                Color::TRANSPARENT,
            ))))
            .trigger_border_color(
                WidgetStateProperty::new(Some(ColorRef::Color(Color::TRANSPARENT)))
                    .when(WidgetStates::SELECTED, Some(primary)),
            );

        Self {
            inner: TabsRoot::uncontrolled(default_value)
                .style(line_style)
                .refine_style(ChromeRefinement::default().bg(ColorRef::Color(Color::TRANSPARENT)))
                .refine_layout(LayoutRefinement::default().w_full().min_w_0()),
        }
    }

    pub fn list(mut self, list: TabsList) -> Self {
        self.inner = self.inner.list(list);
        self
    }

    pub fn content(mut self, content: TabsContent) -> Self {
        self.inner = self.inner.content(content);
        self
    }

    pub fn contents(mut self, contents: impl IntoIterator<Item = TabsContent>) -> Self {
        self.inner = self.inner.contents(contents);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.inner = self.inner.refine_layout(layout);
        self
    }

    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.inner = self.inner.refine_style(chrome);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        self.inner.into_element(cx)
    }
}
