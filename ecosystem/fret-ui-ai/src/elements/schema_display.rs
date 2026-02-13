//! AI Elements-aligned `SchemaDisplay` surfaces.

use std::sync::Arc;

use fret_core::{
    AttributedText, Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow,
    TextPaintStyle, TextShapingStyle, TextSpan, TextStyle, TextWrap,
};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableProps, SelectableTextProps,
    SemanticsDecoration, SizeStyle, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Justify, LayoutRefinement, MetricRef, Radius, Space,
    ui,
};
use fret_ui_shadcn::{Badge, BadgeVariant, Collapsible, CollapsibleContent};

fn alpha(color: Color, a: f32) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: (color.a * a).clamp(0.0, 1.0),
    }
}

fn border_color(theme: &Theme) -> Color {
    theme.color_required("border")
}

fn muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_required("foreground"))
}

fn monospace_style(theme: &Theme, size: Px, weight: FontWeight) -> TextStyle {
    TextStyle {
        font: FontId::monospace(),
        size,
        weight,
        slant: Default::default(),
        line_height: Some(theme.metric_required("metric.font.mono_line_height")),
        letter_spacing_em: None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

impl HttpMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
        }
    }

    fn accent_color(self) -> Color {
        match self {
            Self::Get => Color {
                r: 0.086,
                g: 0.639,
                b: 0.290,
                a: 1.0,
            },
            Self::Post => Color {
                r: 0.145,
                g: 0.388,
                b: 0.922,
                a: 1.0,
            },
            Self::Put => Color {
                r: 0.792,
                g: 0.541,
                b: 0.016,
                a: 1.0,
            },
            Self::Patch => Color {
                r: 0.792,
                g: 0.541,
                b: 0.016,
                a: 1.0,
            },
            Self::Delete => Color {
                r: 0.863,
                g: 0.149,
                b: 0.149,
                a: 1.0,
            },
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchemaParameterLocation {
    Path,
    Query,
    Header,
}

impl SchemaParameterLocation {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Path => "path",
            Self::Query => "query",
            Self::Header => "header",
        }
    }
}

impl std::fmt::Display for SchemaParameterLocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct SchemaParameter {
    pub name: Arc<str>,
    pub type_name: Arc<str>,
    pub required: bool,
    pub description: Option<Arc<str>>,
    pub location: Option<SchemaParameterLocation>,
}

impl SchemaParameter {
    pub fn new(name: impl Into<Arc<str>>, type_name: impl Into<Arc<str>>) -> Self {
        Self {
            name: name.into(),
            type_name: type_name.into(),
            required: false,
            description: None,
            location: None,
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn description(mut self, description: impl Into<Arc<str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn location(mut self, location: SchemaParameterLocation) -> Self {
        self.location = Some(location);
        self
    }
}

#[derive(Debug, Clone)]
pub struct SchemaProperty {
    pub name: Arc<str>,
    pub type_name: Arc<str>,
    pub required: bool,
    pub description: Option<Arc<str>>,
    pub properties: Arc<[SchemaProperty]>,
    pub items: Option<Box<SchemaProperty>>,
}

impl SchemaProperty {
    pub fn new(name: impl Into<Arc<str>>, type_name: impl Into<Arc<str>>) -> Self {
        Self {
            name: name.into(),
            type_name: type_name.into(),
            required: false,
            description: None,
            properties: Arc::from([]),
            items: None,
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn description(mut self, description: impl Into<Arc<str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn properties(mut self, properties: impl Into<Arc<[SchemaProperty]>>) -> Self {
        self.properties = properties.into();
        self
    }

    pub fn items(mut self, items: SchemaProperty) -> Self {
        self.items = Some(Box::new(items));
        self
    }
}

/// Schema display root aligned with AI Elements `schema-display.tsx`.
#[derive(Debug, Clone)]
pub struct SchemaDisplay {
    method: HttpMethod,
    path: Arc<str>,
    description: Option<Arc<str>>,
    parameters: Arc<[SchemaParameter]>,
    request_body: Arc<[SchemaProperty]>,
    response_body: Arc<[SchemaProperty]>,
    children: Option<Vec<AnyElement>>,
    default_open_parameters: bool,
    default_open_request: bool,
    default_open_response: bool,
    test_id_root: Option<Arc<str>>,
    test_id_parameters_trigger: Option<Arc<str>>,
    test_id_request_trigger: Option<Arc<str>>,
    test_id_response_trigger: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl SchemaDisplay {
    pub fn new(method: HttpMethod, path: impl Into<Arc<str>>) -> Self {
        Self {
            method,
            path: path.into(),
            description: None,
            parameters: Arc::from([]),
            request_body: Arc::from([]),
            response_body: Arc::from([]),
            children: None,
            default_open_parameters: true,
            default_open_request: true,
            default_open_response: true,
            test_id_root: None,
            test_id_parameters_trigger: None,
            test_id_request_trigger: None,
            test_id_response_trigger: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn description(mut self, description: impl Into<Arc<str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn parameters(mut self, parameters: impl Into<Arc<[SchemaParameter]>>) -> Self {
        self.parameters = parameters.into();
        self
    }

    pub fn request_body(mut self, request_body: impl Into<Arc<[SchemaProperty]>>) -> Self {
        self.request_body = request_body.into();
        self
    }

    pub fn response_body(mut self, response_body: impl Into<Arc<[SchemaProperty]>>) -> Self {
        self.response_body = response_body.into();
        self
    }

    /// Overrides the default header/description/content subtree.
    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }

    pub fn default_open_parameters(mut self, open: bool) -> Self {
        self.default_open_parameters = open;
        self
    }

    pub fn default_open_request(mut self, open: bool) -> Self {
        self.default_open_request = open;
        self
    }

    pub fn default_open_response(mut self, open: bool) -> Self {
        self.default_open_response = open;
        self
    }

    pub fn test_id_root(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_root = Some(test_id.into());
        self
    }

    pub fn test_id_parameters_trigger(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_parameters_trigger = Some(test_id.into());
        self
    }

    pub fn test_id_request_trigger(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_request_trigger = Some(test_id.into());
        self
    }

    pub fn test_id_response_trigger(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_response_trigger = Some(test_id.into());
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
        let theme = Theme::global(&*cx.app).clone();

        let base_chrome = ChromeRefinement::default()
            .rounded(Radius::Lg)
            .border_1()
            .bg(ColorRef::Token {
                key: "background",
                fallback: ColorFallback::ThemePanelBackground,
            })
            .border_color(ColorRef::Token {
                key: "border",
                fallback: ColorFallback::ThemePanelBorder,
            });

        let wrapper = decl_style::container_props(
            &theme,
            base_chrome.merge(self.chrome),
            self.layout.overflow_hidden(),
        );

        let method = self.method;
        let path = self.path;
        let description = self.description;
        let parameters = self.parameters;
        let request_body = self.request_body;
        let response_body = self.response_body;

        let default_open_parameters = self.default_open_parameters;
        let default_open_request = self.default_open_request;
        let default_open_response = self.default_open_response;

        let test_id_root = self.test_id_root;
        let test_id_parameters_trigger = self.test_id_parameters_trigger;
        let test_id_request_trigger = self.test_id_request_trigger;
        let test_id_response_trigger = self.test_id_response_trigger;

        let children_override = self.children;

        let el = cx.container(wrapper, move |cx| {
            if let Some(children) = children_override {
                return children;
            }

            let header_row = stack::hstack(
                cx,
                stack::HStackProps::default()
                    .gap(Space::N3)
                    .items_center()
                    .layout(LayoutRefinement::default().min_w_0()),
                move |cx| {
                    vec![
                        SchemaDisplayMethod::new(method).into_element(cx),
                        SchemaDisplayPath::new(path.clone()).into_element(cx),
                    ]
                },
            );

            let header = SchemaDisplayHeader::new([header_row]).into_element(cx);

            let mut out = vec![header];

            if let Some(description) = description.clone() {
                out.push(SchemaDisplayDescription::new(description).into_element(cx));
            }

            let mut sections: Vec<AnyElement> = Vec::new();

            if !parameters.is_empty() {
                sections.push(
                    SchemaDisplayParameters::new(parameters.clone())
                        .default_open(default_open_parameters)
                        .test_id_trigger_opt(test_id_parameters_trigger.clone())
                        .into_element(cx),
                );
            }

            if !request_body.is_empty() {
                sections.push(
                    SchemaDisplayRequest::new(request_body.clone())
                        .default_open(default_open_request)
                        .test_id_trigger_opt(test_id_request_trigger.clone())
                        .into_element(cx),
                );
            }

            if !response_body.is_empty() {
                sections.push(
                    SchemaDisplayResponse::new(response_body.clone())
                        .default_open(default_open_response)
                        .test_id_trigger_opt(test_id_response_trigger.clone())
                        .into_element(cx),
                );
            }

            if !sections.is_empty() {
                out.push(SchemaDisplayContent::new(sections).into_element(cx));
            }

            out
        });

        let Some(test_id) = test_id_root else {
            return el;
        };

        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

/// Header row wrapper (`border-b`, padding) aligned with AI Elements `SchemaDisplayHeader`.
#[derive(Debug, Clone)]
pub struct SchemaDisplayHeader {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl SchemaDisplayHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default().px(Space::N4).py(Space::N3),
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

        let mut props = decl_style::container_props(&theme, self.chrome, self.layout);
        props.border = Edges {
            top: Px(0.0),
            right: Px(0.0),
            bottom: Px(1.0),
            left: Px(0.0),
        };
        props.border_color = Some(border_color(&theme));

        cx.container(props, move |cx| {
            vec![stack::hstack(
                cx,
                stack::HStackProps::default()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(Space::N3)
                    .items_center(),
                move |_cx| self.children,
            )]
        })
    }
}

/// Method badge aligned with AI Elements `SchemaDisplayMethod`.
#[derive(Debug, Clone, Copy)]
pub struct SchemaDisplayMethod {
    method: HttpMethod,
}

impl SchemaDisplayMethod {
    pub fn new(method: HttpMethod) -> Self {
        Self { method }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let accent = self.method.accent_color();
        let bg = alpha(accent, 0.18);

        let text_px = theme
            .metric_by_key("component.badge.text_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.badge.line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        let mut props = ContainerProps::default();
        props.padding = Edges::symmetric(
            MetricRef::space(Space::N2).resolve(&theme),
            MetricRef::space(Space::N0p5).resolve(&theme),
        );
        props.background = Some(bg);
        props.border = Edges::all(Px(1.0));
        props.border_color = Some(border_color(&theme));
        props.corner_radii = Corners::all(MetricRef::radius(Radius::Full).resolve(&theme));

        let label: Arc<str> = Arc::from(self.method.as_str());
        cx.container(props, move |cx| {
            vec![cx.text_props(TextProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Auto,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: label.clone(),
                style: Some(TextStyle {
                    line_height: Some(line_height),
                    ..monospace_style(&theme, text_px, FontWeight::SEMIBOLD)
                }),
                color: Some(accent),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            })]
        })
    }
}

/// Path label aligned with AI Elements `SchemaDisplayPath` (with `{param}` highlighting).
#[derive(Debug, Clone)]
pub struct SchemaDisplayPath {
    path: Arc<str>,
}

impl SchemaDisplayPath {
    pub fn new(path: impl Into<Arc<str>>) -> Self {
        Self { path: path.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let base_color = theme.color_required("foreground");
        let highlight = theme.color_by_key("primary").unwrap_or_else(|| Color {
            r: 0.145,
            g: 0.388,
            b: 0.922,
            a: 1.0,
        });

        let (text, spans) = highlighted_path_attributed_text(&self.path, base_color, highlight);

        let mut props = SelectableTextProps::new(AttributedText::new(text, spans));
        props.layout.size.width = Length::Auto;
        props.layout.size.height = Length::Auto;
        props.style = Some(monospace_style(
            &theme,
            theme.metric_required("metric.font.mono_size"),
            FontWeight::NORMAL,
        ));
        props.color = Some(base_color);
        props.wrap = TextWrap::None;
        props.overflow = TextOverflow::Clip;

        cx.selectable_text_props(props)
    }
}

fn highlighted_path_attributed_text(
    path: &str,
    base: Color,
    highlight: Color,
) -> (Arc<str>, Arc<[TextSpan]>) {
    let mut text = String::new();
    let mut spans: Vec<TextSpan> = Vec::new();

    let bytes = path.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let start = i;
        if bytes[i] == b'{' {
            if let Some(end_rel) = path[i..].find('}') {
                let end = i + end_rel + 1;
                let seg = &path[i..end];
                text.push_str(seg);
                spans.push(TextSpan {
                    len: seg.len(),
                    shaping: TextShapingStyle::default(),
                    paint: TextPaintStyle::default().with_fg(highlight),
                });
                i = end;
                continue;
            }
        }

        while i < bytes.len() && bytes[i] != b'{' {
            i += 1;
        }

        let seg = &path[start..i];
        if !seg.is_empty() {
            text.push_str(seg);
            spans.push(TextSpan {
                len: seg.len(),
                shaping: TextShapingStyle::default(),
                paint: TextPaintStyle::default().with_fg(base),
            });
        }
    }

    (Arc::<str>::from(text), Arc::<[TextSpan]>::from(spans))
}

/// Description paragraph aligned with AI Elements `SchemaDisplayDescription`.
#[derive(Debug, Clone)]
pub struct SchemaDisplayDescription {
    text: Arc<str>,
    layout: LayoutRefinement,
}

impl SchemaDisplayDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let mut props = ContainerProps::default();
        props.layout = decl_style::layout_style(&theme, self.layout);
        props.padding = Edges::symmetric(
            MetricRef::space(Space::N4).resolve(&theme),
            MetricRef::space(Space::N3).resolve(&theme),
        );
        props.border = Edges {
            top: Px(0.0),
            right: Px(0.0),
            bottom: Px(1.0),
            left: Px(0.0),
        };
        props.border_color = Some(border_color(&theme));

        let text = self.text;
        cx.container(props, move |cx| {
            vec![
                ui::text(cx, text)
                    .text_color(ColorRef::Color(muted_fg(&theme)))
                    .into_element(cx),
            ]
        })
    }
}

/// Content wrapper aligned with AI Elements `SchemaDisplayContent` (`divide-y`).
#[derive(Debug, Clone)]
pub struct SchemaDisplayContent {
    sections: Vec<AnyElement>,
    layout: LayoutRefinement,
}

impl SchemaDisplayContent {
    pub fn new(sections: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            sections: sections.into_iter().collect(),
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let mut out: Vec<AnyElement> = Vec::new();

        for (index, section) in self.sections.into_iter().enumerate() {
            if index == 0 {
                out.push(section);
                continue;
            }

            let mut props = ContainerProps::default();
            props.layout.size.width = Length::Fill;
            props.layout.size.height = Length::Auto;
            props.border = Edges {
                top: Px(1.0),
                right: Px(0.0),
                bottom: Px(0.0),
                left: Px(0.0),
            };
            props.border_color = Some(border_color(&theme));

            out.push(cx.container(props, move |_cx| vec![section]));
        }

        stack::vstack(
            cx,
            stack::VStackProps::default()
                .layout(self.layout)
                .gap(Space::N0),
            move |_cx| out,
        )
    }
}

#[derive(Debug, Clone)]
pub struct SchemaDisplayParameters {
    parameters: Arc<[SchemaParameter]>,
    default_open: bool,
    test_id_trigger: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl SchemaDisplayParameters {
    pub fn new(parameters: impl Into<Arc<[SchemaParameter]>>) -> Self {
        Self {
            parameters: parameters.into(),
            default_open: true,
            test_id_trigger: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn test_id_trigger(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_trigger = Some(test_id.into());
        self
    }

    fn test_id_trigger_opt(mut self, test_id: Option<Arc<str>>) -> Self {
        if test_id.is_some() {
            self.test_id_trigger = test_id;
        }
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let params = self.parameters;
        let count: Arc<str> = Arc::from(params.len().to_string());
        let test_id = self.test_id_trigger;
        let layout = self.layout;

        Collapsible::uncontrolled(self.default_open)
            .refine_layout(layout)
            .into_element_with_open_model(
                cx,
                move |cx, open_model, is_open| {
                    schema_section_trigger(
                        cx,
                        open_model,
                        is_open,
                        "Parameters",
                        Some(count.clone()),
                        test_id.clone(),
                    )
                },
                move |cx| {
                    let list = schema_parameter_list(cx, params.clone());
                    let wrapper = with_top_divider(cx, list);
                    CollapsibleContent::new([wrapper]).into_element(cx)
                },
            )
    }
}

#[derive(Debug, Clone)]
pub struct SchemaDisplayRequest {
    properties: Arc<[SchemaProperty]>,
    default_open: bool,
    test_id_trigger: Option<Arc<str>>,
    test_id_first_property_trigger: Option<Arc<str>>,
    test_id_first_property_child0_trigger: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl SchemaDisplayRequest {
    pub fn new(properties: impl Into<Arc<[SchemaProperty]>>) -> Self {
        Self {
            properties: properties.into(),
            default_open: true,
            test_id_trigger: None,
            test_id_first_property_trigger: None,
            test_id_first_property_child0_trigger: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn test_id_trigger(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_trigger = Some(test_id.into());
        self
    }

    pub fn test_id_first_property_trigger(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_first_property_trigger = Some(test_id.into());
        self
    }

    pub fn test_id_first_property_child0_trigger(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_first_property_child0_trigger = Some(test_id.into());
        self
    }

    fn test_id_trigger_opt(mut self, test_id: Option<Arc<str>>) -> Self {
        if test_id.is_some() {
            self.test_id_trigger = test_id;
        }
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = self.properties;
        let test_id = self.test_id_trigger;
        let test_id_first_property_trigger = self.test_id_first_property_trigger;
        let test_id_first_property_child0_trigger = self.test_id_first_property_child0_trigger;
        let layout = self.layout;

        Collapsible::uncontrolled(self.default_open)
            .refine_layout(layout)
            .into_element_with_open_model(
                cx,
                move |cx, open_model, is_open| {
                    schema_section_trigger(
                        cx,
                        open_model,
                        is_open,
                        "Request Body",
                        None,
                        test_id.clone(),
                    )
                },
                move |cx| {
                    let vec: Vec<SchemaProperty> = props.iter().cloned().collect();
                    let list = schema_property_list_from_vec(
                        cx,
                        vec,
                        0,
                        test_id_first_property_trigger.clone(),
                        test_id_first_property_child0_trigger.clone(),
                    );
                    let wrapper = with_top_divider(cx, list);
                    CollapsibleContent::new([wrapper]).into_element(cx)
                },
            )
    }
}

#[derive(Debug, Clone)]
pub struct SchemaDisplayResponse {
    properties: Arc<[SchemaProperty]>,
    default_open: bool,
    test_id_trigger: Option<Arc<str>>,
    test_id_first_property_trigger: Option<Arc<str>>,
    test_id_first_property_child0_trigger: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl SchemaDisplayResponse {
    pub fn new(properties: impl Into<Arc<[SchemaProperty]>>) -> Self {
        Self {
            properties: properties.into(),
            default_open: true,
            test_id_trigger: None,
            test_id_first_property_trigger: None,
            test_id_first_property_child0_trigger: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = default_open;
        self
    }

    pub fn test_id_trigger(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_trigger = Some(test_id.into());
        self
    }

    pub fn test_id_first_property_trigger(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_first_property_trigger = Some(test_id.into());
        self
    }

    pub fn test_id_first_property_child0_trigger(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_first_property_child0_trigger = Some(test_id.into());
        self
    }

    fn test_id_trigger_opt(mut self, test_id: Option<Arc<str>>) -> Self {
        if test_id.is_some() {
            self.test_id_trigger = test_id;
        }
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = self.properties;
        let test_id = self.test_id_trigger;
        let test_id_first_property_trigger = self.test_id_first_property_trigger;
        let test_id_first_property_child0_trigger = self.test_id_first_property_child0_trigger;
        let layout = self.layout;

        Collapsible::uncontrolled(self.default_open)
            .refine_layout(layout)
            .into_element_with_open_model(
                cx,
                move |cx, open_model, is_open| {
                    schema_section_trigger(
                        cx,
                        open_model,
                        is_open,
                        "Response",
                        None,
                        test_id.clone(),
                    )
                },
                move |cx| {
                    let vec: Vec<SchemaProperty> = props.iter().cloned().collect();
                    let list = schema_property_list_from_vec(
                        cx,
                        vec,
                        0,
                        test_id_first_property_trigger.clone(),
                        test_id_first_property_child0_trigger.clone(),
                    );
                    let wrapper = with_top_divider(cx, list);
                    CollapsibleContent::new([wrapper]).into_element(cx)
                },
            )
    }
}

#[derive(Debug, Clone)]
pub struct SchemaDisplayParameter {
    parameter: SchemaParameter,
}

impl SchemaDisplayParameter {
    pub fn new(parameter: SchemaParameter) -> Self {
        Self { parameter }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let theme_for_line = theme.clone();

        let pad_x = MetricRef::space(Space::N4).resolve(&theme);
        let pad_y = MetricRef::space(Space::N3).resolve(&theme);

        let name = self.parameter.name.clone();
        let type_name = self.parameter.type_name.clone();
        let required = self.parameter.required;
        let description = self.parameter.description.clone();
        let location = self.parameter.location;

        let line = stack::hstack(
            cx,
            stack::HStackProps::default()
                .gap(Space::N2)
                .items_center()
                .layout(LayoutRefinement::default().w_full().min_w_0()),
            move |cx| {
                let mut out = Vec::new();
                out.push(monospace_text(
                    cx,
                    &theme_for_line,
                    name.clone(),
                    theme_for_line.metric_required("metric.font.mono_size"),
                ));
                out.push(
                    Badge::new(type_name.clone())
                        .variant(BadgeVariant::Outline)
                        .into_element(cx),
                );

                if let Some(location) = location {
                    out.push(
                        Badge::new(location.as_str())
                            .variant(BadgeVariant::Secondary)
                            .into_element(cx),
                    );
                }

                if required {
                    out.push(required_badge(cx));
                }

                out
            },
        );

        let desc = description.map(|d| {
            schema_inline_description(cx, &theme, d, Px(pad_x.0 + 24.0), Px(4.0), Px(0.0))
        });

        let mut props = ContainerProps::default();
        props.layout.size.width = Length::Fill;
        props.layout.size.height = Length::Auto;
        props.padding = Edges {
            top: pad_y,
            right: pad_x,
            bottom: pad_y,
            left: Px(pad_x.0 + 24.0),
        };

        cx.container(props, move |_cx| {
            let mut children = vec![line];
            if let Some(desc) = desc {
                children.push(desc);
            }
            children
        })
    }
}

#[derive(Debug, Clone)]
pub struct SchemaDisplayProperty {
    property: SchemaProperty,
    depth: u32,
    default_open_children: bool,
    test_id_trigger: Option<Arc<str>>,
    test_id_first_child_trigger: Option<Arc<str>>,
}

impl SchemaDisplayProperty {
    pub fn new(property: SchemaProperty) -> Self {
        Self {
            property,
            depth: 0,
            default_open_children: true,
            test_id_trigger: None,
            test_id_first_child_trigger: None,
        }
    }

    pub fn depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    /// Controls whether children are open by default at shallow depths (AI Elements uses `depth < 2`).
    pub fn default_open_children(mut self, open: bool) -> Self {
        self.default_open_children = open;
        self
    }

    pub fn test_id_trigger(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_trigger = Some(test_id.into());
        self
    }

    pub fn test_id_first_child_trigger(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id_first_child_trigger = Some(test_id.into());
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let base_left = 40.0;
        let indent_per_depth = 16.0;
        let padding_left = Px(base_left + (self.depth as f32) * indent_per_depth);

        let prop = self.property;
        let has_children = !prop.properties.is_empty() || prop.items.is_some();

        if has_children {
            let default_open = self.default_open_children && self.depth < 2;
            let name = prop.name.clone();
            let name_for_items = name.clone();
            let type_name = prop.type_name.clone();
            let required = prop.required;
            let description = prop.description.clone();
            let properties = prop.properties.clone();
            let items = prop.items.clone();
            let test_id = self.test_id_trigger.clone();
            let test_id_first_child_trigger = self.test_id_first_child_trigger.clone();
            let depth = self.depth;

            return Collapsible::uncontrolled(default_open).into_element_with_open_model(
                cx,
                move |cx, open_model, is_open| {
                    let trigger = schema_property_trigger_row(
                        cx,
                        &theme,
                        open_model,
                        is_open,
                        padding_left,
                        name.clone(),
                        type_name.clone(),
                        required,
                        test_id.clone(),
                    );

                    let Some(description) = description.clone() else {
                        return trigger;
                    };

                    let desc_el = schema_inline_description(
                        cx,
                        &theme,
                        description,
                        Px(padding_left.0 + 24.0),
                        Px(0.0),
                        Px(8.0),
                    );

                    stack::vstack(
                        cx,
                        stack::VStackProps::default()
                            .layout(LayoutRefinement::default().w_full().min_w_0())
                            .gap(Space::N0),
                        move |_cx| vec![trigger, desc_el],
                    )
                },
                move |cx| {
                    let mut children: Vec<SchemaProperty> = Vec::new();
                    children.extend(properties.iter().cloned());
                    if let Some(items) = items.clone() {
                        let mut item = (*items).clone();
                        item.name = Arc::<str>::from(format!("{}[]", name_for_items));
                        children.push(item);
                    }

                    let list = schema_property_list_from_vec(
                        cx,
                        children,
                        depth + 1,
                        test_id_first_child_trigger.clone(),
                        None,
                    );
                    let wrapper = with_top_divider(cx, list);
                    CollapsibleContent::new([wrapper]).into_element(cx)
                },
            );
        }

        schema_property_leaf(cx, &theme, prop, padding_left, self.test_id_trigger.clone())
    }
}

/// Example surface aligned with AI Elements `SchemaDisplayExample` (a muted, scrollable pre).
#[derive(Debug, Clone)]
pub struct SchemaDisplayExample {
    text: Arc<str>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
    chrome: ChromeRefinement,
}

impl SchemaDisplayExample {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            test_id: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
            chrome: ChromeRefinement::default(),
        }
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
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

        let chrome = ChromeRefinement::default()
            .bg(ColorRef::Token {
                key: "muted",
                fallback: ColorFallback::ThemeHoverBackground,
            })
            .rounded(Radius::Md)
            .p(Space::N4)
            .merge(self.chrome);

        let wrapper = decl_style::container_props(&theme, chrome, self.layout);
        let text = self.text;
        let el = cx.container(wrapper, move |cx| {
            vec![cx.text_props(TextProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text,
                style: Some(monospace_style(
                    &theme,
                    theme.metric_required("metric.font.mono_size"),
                    FontWeight::NORMAL,
                )),
                color: Some(theme.color_required("foreground")),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
            })]
        });

        let Some(test_id) = self.test_id else {
            return el;
        };

        el.attach_semantics(
            SemanticsDecoration::default()
                .role(SemanticsRole::Group)
                .test_id(test_id),
        )
    }
}

fn with_top_divider<H: UiHost>(cx: &mut ElementContext<'_, H>, child: AnyElement) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Auto;
    props.border = Edges {
        top: Px(1.0),
        right: Px(0.0),
        bottom: Px(0.0),
        left: Px(0.0),
    };
    props.border_color = Some(border_color(&theme));
    cx.container(props, move |_cx| vec![child])
}

fn required_badge<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();

    let red = theme.color_by_key("destructive").unwrap_or_else(|| Color {
        r: 0.863,
        g: 0.149,
        b: 0.149,
        a: 1.0,
    });

    Badge::new("required")
        .variant(BadgeVariant::Secondary)
        .refine_style(
            ChromeRefinement::default()
                .bg(ColorRef::Color(alpha(red, 0.18)))
                .text_color(ColorRef::Color(red)),
        )
        .into_element(cx)
}

fn monospace_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    text: Arc<str>,
    size: Px,
) -> AnyElement {
    cx.text_props(TextProps {
        layout: LayoutStyle {
            size: SizeStyle {
                width: Length::Auto,
                height: Length::Auto,
                ..Default::default()
            },
            ..Default::default()
        },
        text,
        style: Some(monospace_style(theme, size, FontWeight::NORMAL)),
        color: Some(theme.color_required("foreground")),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
    })
}

fn schema_inline_description<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    text: Arc<str>,
    padding_left: Px,
    pad_top: Px,
    pad_bottom: Px,
) -> AnyElement {
    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Auto;
    props.padding = Edges {
        top: pad_top,
        right: MetricRef::space(Space::N4).resolve(theme),
        bottom: pad_bottom,
        left: padding_left,
    };

    cx.container(props, move |cx| {
        vec![
            ui::text(cx, text)
                .text_color(ColorRef::Color(muted_fg(theme)))
                .into_element(cx),
        ]
    })
}

fn schema_section_trigger<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    open_model: Model<bool>,
    is_open: bool,
    title: &'static str,
    count: Option<Arc<str>>,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app).clone();
    let fg = muted_fg(&theme);
    let hover_bg = theme
        .color_by_key("muted")
        .map(|c| alpha(c, 0.5))
        .unwrap_or_else(|| alpha(theme.color_required("accent"), 0.2));
    let pressed_bg = theme
        .color_by_key("accent")
        .map(|c| alpha(c, 0.35))
        .unwrap_or_else(|| alpha(theme.color_required("secondary"), 0.35));

    let label: Arc<str> = Arc::from(format!("Toggle {title}"));
    let title_arc: Arc<str> = Arc::from(title);

    let mut pressable = PressableProps::default();
    pressable.enabled = true;
    pressable.focusable = true;
    pressable.a11y.role = Some(SemanticsRole::Button);
    pressable.a11y.label = Some(label);
    pressable.a11y.expanded = Some(is_open);
    pressable.a11y.test_id = test_id;

    let theme_for_row = theme.clone();
    cx.pressable(pressable, move |cx, st| {
        cx.pressable_toggle_bool(&open_model);

        let bg = if st.pressed {
            pressed_bg
        } else if st.hovered {
            hover_bg
        } else {
            Color::TRANSPARENT
        };

        let chevron_id = if is_open {
            ids::ui::CHEVRON_DOWN
        } else {
            ids::ui::CHEVRON_RIGHT
        };
        let chevron =
            decl_icon::icon_with(cx, chevron_id, Some(Px(16.0)), Some(ColorRef::Color(fg)));

        let title_text = ui::text(cx, title_arc.clone())
            .font_medium()
            .into_element(cx);

        let count_badge = count.clone().map(|c| {
            Badge::new(c)
                .variant(BadgeVariant::Secondary)
                .refine_layout(LayoutRefinement::default().ml_auto())
                .into_element(cx)
        });

        let mut row_children = vec![chevron, title_text];
        if let Some(badge) = count_badge {
            row_children.push(badge);
        }

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N2)
                .items_center()
                .justify(Justify::Start),
            move |_cx| row_children,
        );

        let mut props = ContainerProps::default();
        props.layout = decl_style::layout_style(
            &theme_for_row,
            LayoutRefinement::default().w_full().min_w_0(),
        );
        props.padding = Edges::symmetric(
            MetricRef::space(Space::N4).resolve(&theme_for_row),
            MetricRef::space(Space::N3).resolve(&theme_for_row),
        );
        props.background = Some(bg);

        vec![cx.container(props, move |_cx| vec![row])]
    })
}

fn schema_parameter_list<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    parameters: Arc<[SchemaParameter]>,
) -> AnyElement {
    let mut rows: Vec<AnyElement> = Vec::new();
    for (index, p) in parameters.iter().cloned().enumerate() {
        let row = SchemaDisplayParameter::new(p).into_element(cx);
        if index == 0 {
            rows.push(row);
        } else {
            rows.push(with_top_divider(cx, row));
        }
    }

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N0),
        move |_cx| rows,
    )
}

fn schema_property_trigger_row<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    open_model: Model<bool>,
    is_open: bool,
    padding_left: Px,
    name: Arc<str>,
    type_name: Arc<str>,
    required: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let fg = muted_fg(theme);
    let hover_bg = theme
        .color_by_key("muted")
        .map(|c| alpha(c, 0.5))
        .unwrap_or_else(|| alpha(theme.color_required("accent"), 0.2));
    let pressed_bg = theme
        .color_by_key("accent")
        .map(|c| alpha(c, 0.35))
        .unwrap_or_else(|| alpha(theme.color_required("secondary"), 0.35));

    let label: Arc<str> = Arc::from(format!("Toggle {name}"));

    let mut pressable = PressableProps::default();
    pressable.enabled = true;
    pressable.focusable = true;
    pressable.a11y.role = Some(SemanticsRole::Button);
    pressable.a11y.label = Some(label);
    pressable.a11y.expanded = Some(is_open);
    pressable.a11y.test_id = test_id;

    let theme_for_row = theme.clone();
    cx.pressable(pressable, move |cx, st| {
        cx.pressable_toggle_bool(&open_model);

        let bg = if st.pressed {
            pressed_bg
        } else if st.hovered {
            hover_bg
        } else {
            Color::TRANSPARENT
        };

        let chevron_id = if is_open {
            ids::ui::CHEVRON_DOWN
        } else {
            ids::ui::CHEVRON_RIGHT
        };
        let chevron =
            decl_icon::icon_with(cx, chevron_id, Some(Px(16.0)), Some(ColorRef::Color(fg)));

        let name_el = monospace_text(
            cx,
            &theme_for_row,
            name.clone(),
            theme_for_row.metric_required("metric.font.mono_size"),
        );
        let ty_badge = Badge::new(type_name.clone())
            .variant(BadgeVariant::Outline)
            .into_element(cx);

        let mut row_children = vec![chevron, name_el, ty_badge];
        if required {
            row_children.push(required_badge(cx));
        }

        let row = stack::hstack(
            cx,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N2)
                .items_center(),
            move |_cx| row_children,
        );

        let mut props = ContainerProps::default();
        props.layout.size.width = Length::Fill;
        props.layout.size.height = Length::Auto;
        props.padding = Edges {
            top: MetricRef::space(Space::N3).resolve(&theme_for_row),
            right: MetricRef::space(Space::N4).resolve(&theme_for_row),
            bottom: MetricRef::space(Space::N3).resolve(&theme_for_row),
            left: padding_left,
        };
        props.background = Some(bg);

        vec![cx.container(props, move |_cx| vec![row])]
    })
}

fn schema_property_leaf<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    theme: &Theme,
    prop: SchemaProperty,
    padding_left: Px,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let spacer = {
        let mut props = ContainerProps::default();
        props.layout.size.width = Length::Px(Px(16.0));
        props.layout.size.height = Length::Px(Px(16.0));
        props.layout.flex.shrink = 0.0;
        cx.container(props, |_cx| Vec::new())
    };

    let name_el = monospace_text(
        cx,
        theme,
        prop.name.clone(),
        theme.metric_required("metric.font.mono_size"),
    );
    let ty_badge = Badge::new(prop.type_name.clone())
        .variant(BadgeVariant::Outline)
        .into_element(cx);

    let mut row_children = vec![spacer, name_el, ty_badge];
    if prop.required {
        row_children.push(required_badge(cx));
    }

    let row = stack::hstack(
        cx,
        stack::HStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N2)
            .items_center(),
        move |_cx| row_children,
    );

    let mut inner: Vec<AnyElement> = vec![row];
    if let Some(desc) = prop.description.clone() {
        inner.push(schema_inline_description(
            cx,
            theme,
            desc,
            Px(padding_left.0 + 24.0),
            Px(4.0),
            Px(0.0),
        ));
    }

    let mut props = ContainerProps::default();
    props.layout.size.width = Length::Fill;
    props.layout.size.height = Length::Auto;
    props.padding = Edges {
        top: MetricRef::space(Space::N3).resolve(theme),
        right: MetricRef::space(Space::N4).resolve(theme),
        bottom: MetricRef::space(Space::N3).resolve(theme),
        left: padding_left,
    };

    let el = cx.container(props, move |_cx| inner);
    let Some(test_id) = test_id else {
        return el;
    };

    el.attach_semantics(
        SemanticsDecoration::default()
            .role(SemanticsRole::Group)
            .test_id(test_id),
    )
}

fn schema_property_list_from_vec<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    properties: Vec<SchemaProperty>,
    depth: u32,
    test_id_first_trigger: Option<Arc<str>>,
    test_id_first_child_trigger: Option<Arc<str>>,
) -> AnyElement {
    let mut rows: Vec<AnyElement> = Vec::new();
    for (index, prop) in properties.into_iter().enumerate() {
        let mut node = SchemaDisplayProperty::new(prop).depth(depth);
        if index == 0 {
            if let Some(id) = test_id_first_trigger.clone() {
                node = node.test_id_trigger(id);
            }
            if let Some(child0) = test_id_first_child_trigger.clone() {
                node = node.test_id_first_child_trigger(child0);
            }
        }
        let el = node.into_element(cx);
        if index == 0 {
            rows.push(el);
        } else {
            rows.push(with_top_divider(cx, el));
        }
    }

    stack::vstack(
        cx,
        stack::VStackProps::default()
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N0),
        move |_cx| rows,
    )
}
