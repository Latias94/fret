//! AI Elements-aligned `SchemaDisplay` surfaces.

use std::sync::Arc;

use fret_core::{
    AttributedText, Color, Corners, Edges, FontId, FontWeight, Px, SemanticsRole, TextOverflow,
    TextPaintStyle, TextShapingStyle, TextSpan, TextStyle, TextWrap,
};
use fret_icons::ids;
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, ContainerProps, InteractivityGateProps, LayoutStyle, Length, PressableProps,
    SemanticsDecoration, SizeStyle, StyledTextProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::action_hooks::ActionHooksExt as _;
use fret_ui_kit::declarative::icon as decl_icon;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, Items, Justify, LayoutRefinement, MetricRef, Radius,
    Space, ui,
};
use fret_ui_shadcn::facade::{Badge, BadgeVariant, Collapsible, CollapsibleContent};

fn alpha(color: Color, a: f32) -> Color {
    Color {
        r: color.r,
        g: color.g,
        b: color.b,
        a: (color.a * a).clamp(0.0, 1.0),
    }
}

fn border_color(theme: &Theme) -> Color {
    theme.color_token("border")
}

fn muted_fg(theme: &Theme) -> Color {
    theme
        .color_by_key("muted-foreground")
        .or_else(|| theme.color_by_key("muted_foreground"))
        .unwrap_or_else(|| theme.color_token("foreground"))
}

fn monospace_style(theme: &Theme, size: Px, weight: FontWeight) -> TextStyle {
    typography::as_control_text(TextStyle {
        font: FontId::monospace(),
        size,
        weight,
        slant: Default::default(),
        line_height: Some(theme.metric_token("metric.font.mono_line_height")),
        letter_spacing_em: None,
        ..Default::default()
    })
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

    fn accent_color(self, theme: &Theme) -> Color {
        fn token(theme: &Theme, key: &'static str, fallback: Color) -> Color {
            theme.color_by_key(key).unwrap_or(fallback)
        }

        match self {
            Self::Get => token(
                theme,
                "component.schema_display.method.get",
                // Tailwind: green-600 (#16a34a).
                fret_ui_kit::colors::linear_from_hex_rgb(0x16_a3_4a),
            ),
            Self::Post => token(
                theme,
                "component.schema_display.method.post",
                // Tailwind: blue-600 (#2563eb).
                fret_ui_kit::colors::linear_from_hex_rgb(0x25_63_eb),
            ),
            Self::Put => token(
                theme,
                "component.schema_display.method.put",
                // Tailwind: orange-600 (#ea580c).
                fret_ui_kit::colors::linear_from_hex_rgb(0xea_58_0c),
            ),
            Self::Patch => token(
                theme,
                "component.schema_display.method.patch",
                // Tailwind: yellow-600 (#ca8a04).
                fret_ui_kit::colors::linear_from_hex_rgb(0xca_8a_04),
            ),
            Self::Delete => token(
                theme,
                "component.schema_display.method.delete",
                // Tailwind: red-600 (#dc2626).
                fret_ui_kit::colors::linear_from_hex_rgb(0xdc_26_26),
            ),
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
struct SchemaDisplayContext {
    method: HttpMethod,
    path: Arc<str>,
    description: Option<Arc<str>>,
    parameters: Arc<[SchemaParameter]>,
    request_body: Arc<[SchemaProperty]>,
    response_body: Arc<[SchemaProperty]>,
    default_open_parameters: bool,
    default_open_request: bool,
    default_open_response: bool,
    test_id_parameters_trigger: Option<Arc<str>>,
    test_id_request_trigger: Option<Arc<str>>,
    test_id_response_trigger: Option<Arc<str>>,
}

#[derive(Debug, Default, Clone)]
struct SchemaDisplayLocalState {
    context: Option<SchemaDisplayContext>,
}

fn schema_display_context<H: UiHost>(cx: &ElementContext<'_, H>) -> Option<SchemaDisplayContext> {
    cx.inherited_state::<SchemaDisplayLocalState>()
        .and_then(|state| state.context.clone())
}

#[track_caller]
fn require_schema_display_context<H: UiHost>(cx: &ElementContext<'_, H>) -> SchemaDisplayContext {
    schema_display_context(cx).unwrap_or_else(|| {
        panic!("SchemaDisplay context is missing. Use SchemaDisplay::into_element_with_children(...) when composing context-aware children such as SchemaDisplayMethod::from_context().")
    })
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
#[derive(Debug)]
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

    fn context(&self) -> SchemaDisplayContext {
        SchemaDisplayContext {
            method: self.method,
            path: self.path.clone(),
            description: self.description.clone(),
            parameters: self.parameters.clone(),
            request_body: self.request_body.clone(),
            response_body: self.response_body.clone(),
            default_open_parameters: self.default_open_parameters,
            default_open_request: self.default_open_request,
            default_open_response: self.default_open_response,
            test_id_parameters_trigger: self.test_id_parameters_trigger.clone(),
            test_id_request_trigger: self.test_id_request_trigger.clone(),
            test_id_response_trigger: self.test_id_response_trigger.clone(),
        }
    }

    pub fn into_element_with_children<H: UiHost>(
        mut self,
        cx: &mut ElementContext<'_, H>,
        children: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
    ) -> AnyElement {
        let context = self.context();
        cx.root_state(SchemaDisplayLocalState::default, |state| {
            state.context = Some(context);
        });

        self.children = Some(children(cx));
        self.into_element(cx)
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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
            let body_children = if let Some(children) = children_override {
                children
            } else {
                let header_row = ui::h_row(move |cx| {
                    vec![
                        SchemaDisplayMethod::new(method).into_element(cx),
                        SchemaDisplayPath::new(path.clone()).into_element(cx),
                    ]
                })
                .gap(Space::N3)
                .items(Items::Center)
                .layout(LayoutRefinement::default().min_w_0())
                .into_element(cx);

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
            };

            vec![
                ui::v_stack(move |_cx| body_children)
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(Space::N0)
                    .into_element(cx),
            ]
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
#[derive(Debug)]
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
            vec![
                ui::h_row(move |_cx| self.children)
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(Space::N3)
                    .items(Items::Center)
                    .into_element(cx),
            ]
        })
    }
}

/// Method badge aligned with AI Elements `SchemaDisplayMethod`.
#[derive(Debug, Clone, Copy)]
pub struct SchemaDisplayMethod {
    method: HttpMethod,
    from_context: bool,
}

impl SchemaDisplayMethod {
    pub fn new(method: HttpMethod) -> Self {
        Self {
            method,
            from_context: false,
        }
    }

    pub fn from_context() -> Self {
        Self {
            method: HttpMethod::Get,
            from_context: true,
        }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let method = if self.from_context {
            require_schema_display_context(cx).method
        } else {
            self.method
        };

        let accent = method.accent_color(&theme);
        let bg = alpha(accent, 0.18);

        let text_px = theme
            .metric_by_key("component.badge.text_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let line_height = theme
            .metric_by_key("component.badge.line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_token("font.line_height"));

        let mut props = ContainerProps::default();
        props.padding = Edges::symmetric(
            MetricRef::space(Space::N2).resolve(&theme),
            MetricRef::space(Space::N0p5).resolve(&theme),
        )
        .into();
        props.background = Some(bg);
        props.border = Edges::all(Px(1.0));
        props.border_color = Some(border_color(&theme));
        props.corner_radii = Corners::all(MetricRef::radius(Radius::Full).resolve(&theme));

        let label: Arc<str> = Arc::from(method.as_str());
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
                align: fret_core::TextAlign::Start,

                ink_overflow: fret_ui::element::TextInkOverflow::None,
            })]
        })
    }
}

/// Path label aligned with AI Elements `SchemaDisplayPath` (with `{param}` highlighting).
#[derive(Debug, Clone)]
pub struct SchemaDisplayPath {
    path: Option<Arc<str>>,
    from_context: bool,
}

impl SchemaDisplayPath {
    pub fn new(path: impl Into<Arc<str>>) -> Self {
        Self {
            path: Some(path.into()),
            from_context: false,
        }
    }

    pub fn from_context() -> Self {
        Self {
            path: None,
            from_context: true,
        }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let path = if self.from_context {
            require_schema_display_context(cx).path
        } else {
            self.path
                .expect("SchemaDisplayPath::new(...) should always provide a path.")
        };
        let base_color = theme.color_token("foreground");
        let highlight = theme
            .color_by_key("primary")
            // Tailwind: blue-600 (#2563eb).
            .unwrap_or_else(|| fret_ui_kit::colors::linear_from_hex_rgb(0x25_63_eb));

        let (text, spans) = highlighted_path_attributed_text(&path, base_color, highlight);

        let mut props = StyledTextProps::new(AttributedText::new(text, spans));
        props.layout.size.width = Length::Auto;
        props.layout.size.height = Length::Auto;
        props.style = Some(monospace_style(
            &theme,
            theme.metric_token("metric.font.mono_size"),
            FontWeight::NORMAL,
        ));
        props.color = Some(base_color);
        props.wrap = TextWrap::None;
        props.overflow = TextOverflow::Clip;

        cx.styled_text_props(props)
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
    text: Option<Arc<str>>,
    from_context: bool,
    layout: LayoutRefinement,
}

impl SchemaDisplayDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: Some(text.into()),
            from_context: false,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn from_context() -> Self {
        Self {
            text: None,
            from_context: true,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let text = if self.from_context {
            require_schema_display_context(cx).description
        } else {
            self.text
        };
        let Some(text) = text else {
            return hidden(cx);
        };

        let mut props = ContainerProps::default();
        props.layout = decl_style::layout_style(&theme, self.layout);
        props.padding = Edges::symmetric(
            MetricRef::space(Space::N4).resolve(&theme),
            MetricRef::space(Space::N3).resolve(&theme),
        )
        .into();
        props.border = Edges {
            top: Px(0.0),
            right: Px(0.0),
            bottom: Px(1.0),
            left: Px(0.0),
        };
        props.border_color = Some(border_color(&theme));

        typography::scope_description_text_with_fallbacks(
            cx.container(props, move |cx| vec![ui::raw_text(text).into_element(cx)]),
            &theme,
            "component.schema_display.description",
            Some("component.text.sm_px"),
            Some("component.text.sm_line_height"),
        )
    }
}

/// Content wrapper aligned with AI Elements `SchemaDisplayContent` (`divide-y`).
#[derive(Debug)]
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

        ui::v_stack(move |_cx| out)
            .layout(self.layout)
            .gap(Space::N0)
            .into_element(cx)
    }
}

#[derive(Debug)]
pub struct SchemaDisplayParameters {
    parameters: Arc<[SchemaParameter]>,
    from_context: bool,
    children: Option<Vec<AnyElement>>,
    default_open: Option<bool>,
    test_id_trigger: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl SchemaDisplayParameters {
    pub fn new(parameters: impl Into<Arc<[SchemaParameter]>>) -> Self {
        Self {
            parameters: parameters.into(),
            from_context: false,
            children: None,
            default_open: Some(true),
            test_id_trigger: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn from_context() -> Self {
        Self {
            parameters: Arc::from([]),
            from_context: true,
            children: None,
            default_open: None,
            test_id_trigger: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
        self
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = Some(default_open);
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let SchemaDisplayParameters {
            parameters,
            from_context,
            children,
            default_open,
            test_id_trigger,
            layout,
        } = self;

        let (params, inherited_default_open, inherited_test_id_trigger) = if from_context {
            let context = require_schema_display_context(cx);
            (
                context.parameters,
                context.default_open_parameters,
                context.test_id_parameters_trigger,
            )
        } else {
            (parameters, true, None)
        };

        if from_context && params.is_empty() && children.is_none() {
            return hidden(cx);
        }

        let count: Arc<str> = Arc::from(params.len().to_string());
        let default_open = default_open.unwrap_or(inherited_default_open);
        let test_id = test_id_trigger.or(inherited_test_id_trigger);

        Collapsible::uncontrolled(default_open)
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
                    let list = if let Some(children) = children {
                        ui::v_stack(move |_cx| children)
                            .layout(LayoutRefinement::default().w_full().min_w_0())
                            .gap(Space::N0)
                            .into_element(cx)
                    } else {
                        schema_parameter_list(cx, params.clone())
                    };
                    let wrapper = with_top_divider(cx, list);
                    CollapsibleContent::new([wrapper]).into_element(cx)
                },
            )
    }
}

#[derive(Debug)]
pub struct SchemaDisplayRequest {
    properties: Arc<[SchemaProperty]>,
    from_context: bool,
    children: Option<Vec<AnyElement>>,
    default_open: Option<bool>,
    test_id_trigger: Option<Arc<str>>,
    test_id_first_property_trigger: Option<Arc<str>>,
    test_id_first_property_child0_trigger: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl SchemaDisplayRequest {
    pub fn new(properties: impl Into<Arc<[SchemaProperty]>>) -> Self {
        Self {
            properties: properties.into(),
            from_context: false,
            children: None,
            default_open: Some(true),
            test_id_trigger: None,
            test_id_first_property_trigger: None,
            test_id_first_property_child0_trigger: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn from_context() -> Self {
        Self {
            properties: Arc::from([]),
            from_context: true,
            children: None,
            default_open: None,
            test_id_trigger: None,
            test_id_first_property_trigger: None,
            test_id_first_property_child0_trigger: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = Some(default_open);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let SchemaDisplayRequest {
            properties,
            from_context,
            children,
            default_open,
            test_id_trigger,
            test_id_first_property_trigger,
            test_id_first_property_child0_trigger,
            layout,
        } = self;

        let (props, inherited_default_open, inherited_test_id_trigger) = if from_context {
            let context = require_schema_display_context(cx);
            (
                context.request_body,
                context.default_open_request,
                context.test_id_request_trigger,
            )
        } else {
            (properties, true, None)
        };

        if from_context && props.is_empty() && children.is_none() {
            return hidden(cx);
        }

        let default_open = default_open.unwrap_or(inherited_default_open);
        let test_id = test_id_trigger.or(inherited_test_id_trigger);

        Collapsible::uncontrolled(default_open)
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
                    let list = if let Some(children) = children {
                        ui::v_stack(move |_cx| children)
                            .layout(LayoutRefinement::default().w_full().min_w_0())
                            .gap(Space::N0)
                            .into_element(cx)
                    } else {
                        let vec: Vec<SchemaProperty> = props.iter().cloned().collect();
                        schema_property_list_from_vec(
                            cx,
                            vec,
                            0,
                            test_id_first_property_trigger.clone(),
                            test_id_first_property_child0_trigger.clone(),
                        )
                    };
                    let wrapper = with_top_divider(cx, list);
                    CollapsibleContent::new([wrapper]).into_element(cx)
                },
            )
    }
}

#[derive(Debug)]
pub struct SchemaDisplayResponse {
    properties: Arc<[SchemaProperty]>,
    from_context: bool,
    children: Option<Vec<AnyElement>>,
    default_open: Option<bool>,
    test_id_trigger: Option<Arc<str>>,
    test_id_first_property_trigger: Option<Arc<str>>,
    test_id_first_property_child0_trigger: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl SchemaDisplayResponse {
    pub fn new(properties: impl Into<Arc<[SchemaProperty]>>) -> Self {
        Self {
            properties: properties.into(),
            from_context: false,
            children: None,
            default_open: Some(true),
            test_id_trigger: None,
            test_id_first_property_trigger: None,
            test_id_first_property_child0_trigger: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn from_context() -> Self {
        Self {
            properties: Arc::from([]),
            from_context: true,
            children: None,
            default_open: None,
            test_id_trigger: None,
            test_id_first_property_trigger: None,
            test_id_first_property_child0_trigger: None,
            layout: LayoutRefinement::default().w_full().min_w_0(),
        }
    }

    pub fn default_open(mut self, default_open: bool) -> Self {
        self.default_open = Some(default_open);
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let SchemaDisplayResponse {
            properties,
            from_context,
            children,
            default_open,
            test_id_trigger,
            test_id_first_property_trigger,
            test_id_first_property_child0_trigger,
            layout,
        } = self;

        let (props, inherited_default_open, inherited_test_id_trigger) = if from_context {
            let context = require_schema_display_context(cx);
            (
                context.response_body,
                context.default_open_response,
                context.test_id_response_trigger,
            )
        } else {
            (properties, true, None)
        };

        if from_context && props.is_empty() && children.is_none() {
            return hidden(cx);
        }

        let default_open = default_open.unwrap_or(inherited_default_open);
        let test_id = test_id_trigger.or(inherited_test_id_trigger);

        Collapsible::uncontrolled(default_open)
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
                    let list = if let Some(children) = children {
                        ui::v_stack(move |_cx| children)
                            .layout(LayoutRefinement::default().w_full().min_w_0())
                            .gap(Space::N0)
                            .into_element(cx)
                    } else {
                        let vec: Vec<SchemaProperty> = props.iter().cloned().collect();
                        schema_property_list_from_vec(
                            cx,
                            vec,
                            0,
                            test_id_first_property_trigger.clone(),
                            test_id_first_property_child0_trigger.clone(),
                        )
                    };
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

        let line = ui::h_row(move |cx| {
            let mut out = Vec::new();
            out.push(monospace_text(
                cx,
                &theme_for_line,
                name.clone(),
                theme_for_line.metric_token("metric.font.mono_size"),
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
        })
        .gap(Space::N2)
        .items(Items::Center)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);

        let desc = description
            .map(|d| schema_inline_description(cx, &theme, d, Px(0.0), Px(4.0), Px(0.0)));

        let mut props = ContainerProps::default();
        props.layout.size.width = Length::Fill;
        props.layout.size.height = Length::Auto;
        props.padding = Edges {
            top: pad_y,
            right: pad_x,
            bottom: pad_y,
            left: Px(pad_x.0 + 24.0),
        }
        .into();

        cx.container(props, move |cx| {
            let mut children = vec![line];
            if let Some(desc) = desc {
                children.push(desc);
            }
            vec![
                ui::v_stack(move |_cx| children)
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .gap(Space::N0)
                    .into_element(cx),
            ]
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
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

                    ui::v_stack(move |_cx| vec![trigger, desc_el])
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .gap(Space::N0)
                        .into_element(cx)
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
                    theme.metric_token("metric.font.mono_size"),
                    FontWeight::NORMAL,
                )),
                color: Some(theme.color_token("foreground")),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,

                ink_overflow: fret_ui::element::TextInkOverflow::None,
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

    let red = theme
        .color_by_key("destructive")
        // Tailwind: red-600 (#dc2626).
        .unwrap_or_else(|| fret_ui_kit::colors::linear_from_hex_rgb(0xdc_26_26));

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
        color: Some(theme.color_token("foreground")),
        wrap: TextWrap::None,
        overflow: TextOverflow::Clip,
        align: fret_core::TextAlign::Start,

        ink_overflow: fret_ui::element::TextInkOverflow::None,
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
    }
    .into();

    typography::scope_description_text_with_fallbacks(
        cx.container(props, move |cx| vec![ui::raw_text(text).into_element(cx)]),
        theme,
        "component.schema_display.description",
        Some("component.text.sm_px"),
        Some("component.text.sm_line_height"),
    )
}

fn schema_section_trigger<H: UiHost>(
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
        .unwrap_or_else(|| alpha(theme.color_token("accent"), 0.2));
    let pressed_bg = theme
        .color_by_key("accent")
        .map(|c| alpha(c, 0.35))
        .unwrap_or_else(|| alpha(theme.color_token("secondary"), 0.35));

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

        let title_text = ui::text(title_arc.clone())
            .font_medium()
            .nowrap()
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

        let row = ui::h_row(move |_cx| row_children)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N2)
            .items(Items::Center)
            .justify(Justify::Start)
            .into_element(cx);

        let mut props = ContainerProps::default();
        props.layout = decl_style::layout_style(
            &theme_for_row,
            LayoutRefinement::default().w_full().min_w_0(),
        );
        props.padding = Edges::symmetric(
            MetricRef::space(Space::N4).resolve(&theme_for_row),
            MetricRef::space(Space::N3).resolve(&theme_for_row),
        )
        .into();
        props.background = Some(bg);

        vec![cx.container(props, move |_cx| vec![row])]
    })
}

fn schema_parameter_list<H: UiHost>(
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

    ui::v_stack(move |_cx| rows)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N0)
        .into_element(cx)
}

fn schema_property_trigger_row<H: UiHost>(
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
        .unwrap_or_else(|| alpha(theme.color_token("accent"), 0.2));
    let pressed_bg = theme
        .color_by_key("accent")
        .map(|c| alpha(c, 0.35))
        .unwrap_or_else(|| alpha(theme.color_token("secondary"), 0.35));

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
            theme_for_row.metric_token("metric.font.mono_size"),
        );
        let ty_badge = Badge::new(type_name.clone())
            .variant(BadgeVariant::Outline)
            .into_element(cx);

        let mut row_children = vec![chevron, name_el, ty_badge];
        if required {
            row_children.push(required_badge(cx));
        }

        let row = ui::h_row(move |_cx| row_children)
            .layout(LayoutRefinement::default().w_full().min_w_0())
            .gap(Space::N2)
            .items(Items::Center)
            .into_element(cx);

        let mut props = ContainerProps::default();
        props.layout.size.width = Length::Fill;
        props.layout.size.height = Length::Auto;
        props.padding = Edges {
            top: MetricRef::space(Space::N3).resolve(&theme_for_row),
            right: MetricRef::space(Space::N4).resolve(&theme_for_row),
            bottom: MetricRef::space(Space::N3).resolve(&theme_for_row),
            left: padding_left,
        }
        .into();
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
        theme.metric_token("metric.font.mono_size"),
    );
    let ty_badge = Badge::new(prop.type_name.clone())
        .variant(BadgeVariant::Outline)
        .into_element(cx);

    let mut row_children = vec![spacer, name_el, ty_badge];
    if prop.required {
        row_children.push(required_badge(cx));
    }

    let row = ui::h_row(move |_cx| row_children)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N2)
        .items(Items::Center)
        .into_element(cx);

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
    }
    .into();

    let el = cx.container(props, move |cx| {
        vec![
            ui::v_stack(move |_cx| inner)
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .gap(Space::N0)
                .into_element(cx),
        ]
    });
    let Some(test_id) = test_id else {
        return el;
    };

    el.attach_semantics(
        SemanticsDecoration::default()
            .role(SemanticsRole::Group)
            .test_id(test_id),
    )
}

fn schema_property_list_from_vec<H: UiHost>(
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

    ui::v_stack(move |_cx| rows)
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .gap(Space::N0)
        .into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::ElementKind;
    use fret_ui::{Theme, ThemeConfig};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(720.0), Px(480.0)),
        )
    }

    fn find_text_element<'a>(element: &'a AnyElement, needle: &str) -> Option<&'a AnyElement> {
        match &element.kind {
            ElementKind::Text(props) if props.text.as_ref() == needle => Some(element),
            ElementKind::StyledText(props) if props.rich.text.as_ref() == needle => Some(element),
            _ => element
                .children
                .iter()
                .find_map(|child| find_text_element(child, needle)),
        }
    }

    #[test]
    fn schema_display_put_method_uses_orange_accent() {
        let app = App::new();
        let theme = Theme::global(&app).clone();
        assert_eq!(
            HttpMethod::Put.accent_color(&theme),
            fret_ui_kit::colors::linear_from_hex_rgb(0xea_58_0c)
        );
        assert_ne!(
            HttpMethod::Put.accent_color(&theme),
            HttpMethod::Patch.accent_color(&theme)
        );
    }

    #[test]
    fn schema_display_root_wraps_default_sections_in_single_stacked_child() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                SchemaDisplay::new(HttpMethod::Post, "/api/posts")
                    .description("Create a post")
                    .request_body([SchemaProperty::new("title", "string").required(true)])
                    .response_body([SchemaProperty::new("id", "string").required(true)])
                    .into_element(cx)
            });

        let ElementKind::Container(_) = &element.kind else {
            panic!("expected SchemaDisplay root to build a Container");
        };
        assert_eq!(
            element.children.len(),
            1,
            "SchemaDisplay should stack header/description/content inside one body child"
        );
        assert!(find_text_element(&element, "POST").is_some());
        assert!(find_text_element(&element, "Create a post").is_some());
        assert!(find_text_element(&element, "Request Body").is_some());
        assert!(find_text_element(&element, "Response").is_some());
    }

    #[test]
    fn schema_display_children_override_is_stacked_in_single_body_child() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                SchemaDisplay::new(HttpMethod::Get, "/api/users")
                    .children([cx.text("Custom header"), cx.text("Custom body")])
                    .into_element(cx)
            });

        let ElementKind::Container(_) = &element.kind else {
            panic!("expected SchemaDisplay override root to build a Container");
        };
        assert_eq!(
            element.children.len(),
            1,
            "SchemaDisplay override content should still flow through a single stacked body"
        );
        assert!(find_text_element(&element, "Custom header").is_some());
        assert!(find_text_element(&element, "Custom body").is_some());
    }

    #[test]
    fn schema_display_context_driven_children_can_consume_root_context() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                SchemaDisplay::new(HttpMethod::Delete, "/api/posts/{postId}")
                    .description("Delete a post")
                    .parameters([SchemaParameter::new("postId", "string").required(true)])
                    .request_body([SchemaProperty::new("reason", "string")])
                    .response_body([SchemaProperty::new("deleted", "boolean").required(true)])
                    .into_element_with_children(cx, |cx| {
                        vec![
                            SchemaDisplayHeader::new([
                                SchemaDisplayMethod::from_context().into_element(cx),
                                SchemaDisplayPath::from_context().into_element(cx),
                            ])
                            .into_element(cx),
                            SchemaDisplayDescription::from_context().into_element(cx),
                            SchemaDisplayContent::new([
                                SchemaDisplayParameters::from_context().into_element(cx),
                                SchemaDisplayRequest::from_context().into_element(cx),
                                SchemaDisplayResponse::from_context().into_element(cx),
                            ])
                            .into_element(cx),
                        ]
                    })
            });

        assert!(find_text_element(&element, "DELETE").is_some());
        assert!(find_text_element(&element, "/api/posts/{postId}").is_some());
        assert!(find_text_element(&element, "Delete a post").is_some());
        assert!(find_text_element(&element, "Parameters").is_some());
        assert!(find_text_element(&element, "Request Body").is_some());
        assert!(find_text_element(&element, "Response").is_some());
    }

    #[test]
    fn schema_display_request_children_override_replaces_default_property_rows() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                SchemaDisplayRequest::new([SchemaProperty::new("title", "string").required(true)])
                    .children([cx.text("Custom request row")])
                    .into_element(cx)
            });

        assert!(find_text_element(&element, "Request Body").is_some());
        assert!(find_text_element(&element, "Custom request row").is_some());
        assert!(find_text_element(&element, "title").is_none());
    }

    #[test]
    fn schema_display_description_scopes_inherited_description_typography() {
        let window = AppWindowId::default();
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Schema Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("component.text.sm_px".to_string(), 13.0),
                    ("component.text.sm_line_height".to_string(), 18.0),
                ]),
                colors: std::collections::HashMap::from([(
                    "muted-foreground".to_string(),
                    "#556677".to_string(),
                )]),
                ..ThemeConfig::default()
            });
        });

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                SchemaDisplayDescription::new("Schema summary").into_element(cx)
            });

        let ElementKind::Container(_) = &element.kind else {
            panic!("expected SchemaDisplayDescription to build a Container root");
        };
        let child = element.children.first().expect("expected raw text child");
        let ElementKind::Text(props) = &child.kind else {
            panic!("expected SchemaDisplayDescription child to be Text");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.wrap, TextWrap::Word);

        let theme = Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_foreground,
            Some(typography::muted_foreground_color(&theme))
        );
        assert_eq!(
            element.inherited_text_style,
            Some(typography::description_text_refinement_with_fallbacks(
                &theme,
                "component.schema_display.description",
                Some("component.text.sm_px"),
                Some("component.text.sm_line_height"),
            ))
        );
    }

    #[test]
    fn schema_inline_description_scopes_inherited_description_typography() {
        let window = AppWindowId::default();
        let mut app = App::new();
        Theme::with_global_mut(&mut app, |theme| {
            theme.apply_config(&ThemeConfig {
                name: "Schema Inline Test".to_string(),
                metrics: std::collections::HashMap::from([
                    ("component.text.sm_px".to_string(), 13.0),
                    ("component.text.sm_line_height".to_string(), 18.0),
                ]),
                colors: std::collections::HashMap::from([(
                    "muted-foreground".to_string(),
                    "#445566".to_string(),
                )]),
                ..ThemeConfig::default()
            });
        });

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                let theme = Theme::global(&*cx.app).clone();
                schema_inline_description(
                    cx,
                    &theme,
                    Arc::from("Inline description"),
                    Px(24.0),
                    Px(4.0),
                    Px(0.0),
                )
            });

        let ElementKind::Container(_) = &element.kind else {
            panic!("expected schema_inline_description to build a Container root");
        };
        let child = element.children.first().expect("expected raw text child");
        let ElementKind::Text(props) = &child.kind else {
            panic!("expected schema_inline_description child to be Text");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());
        assert_eq!(props.wrap, TextWrap::Word);

        let theme = Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_foreground,
            Some(typography::muted_foreground_color(&theme))
        );
        assert_eq!(
            element.inherited_text_style,
            Some(typography::description_text_refinement_with_fallbacks(
                &theme,
                "component.schema_display.description",
                Some("component.text.sm_px"),
                Some("component.text.sm_line_height"),
            ))
        );
    }
}
