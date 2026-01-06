use std::sync::Arc;

use fret_core::{Edges, FontId, FontWeight, Px, TextOverflow, TextStyle, TextWrap};
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, MainAlign, RowProps, TextProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement, MetricRef, Space};

use crate::label::label as shadcn_label;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FieldOrientation {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Debug, Clone)]
pub struct FieldSet {
    children: Vec<AnyElement>,
}

impl FieldSet {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let gap = MetricRef::space(Space::N6).resolve(&theme);
        let layout = decl_style::layout_style(&theme, LayoutRefinement::default().w_full());
        let children = self.children;
        cx.column(
            ColumnProps {
                layout,
                gap,
                ..Default::default()
            },
            move |_cx| children,
        )
    }
}

pub fn field_set<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    FieldSet::new(f(cx)).into_element(cx)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FieldLegendVariant {
    #[default]
    Legend,
    Label,
}

#[derive(Debug, Clone)]
pub struct FieldLegend {
    text: Arc<str>,
    variant: FieldLegendVariant,
}

impl FieldLegend {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            variant: FieldLegendVariant::default(),
        }
    }

    pub fn variant(mut self, variant: FieldLegendVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let fg = theme
            .color_by_key("foreground")
            .unwrap_or(theme.colors.text_primary);
        let base_px = theme
            .metric_by_key("font.size")
            .unwrap_or(theme.metrics.font_size);
        let line_height = theme
            .metric_by_key("font.line_height")
            .unwrap_or(theme.metrics.font_line_height);

        let size = match self.variant {
            FieldLegendVariant::Legend => Px((base_px.0 + 1.0).max(0.0)),
            FieldLegendVariant::Label => base_px,
        };

        cx.text_props(TextProps {
            layout: Default::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size,
                weight: FontWeight::MEDIUM,
                line_height: Some(line_height),
                letter_spacing_em: None,
            }),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FieldGroup {
    children: Vec<AnyElement>,
}

impl FieldGroup {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let gap = theme
            .metric_by_key("component.field.group_gap")
            .unwrap_or_else(|| MetricRef::space(Space::N8).resolve(&theme));
        let layout = decl_style::layout_style(&theme, LayoutRefinement::default().w_full());
        let children = self.children;
        cx.column(
            ColumnProps {
                layout,
                gap,
                ..Default::default()
            },
            move |_cx| children,
        )
    }
}

pub fn field_group<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    FieldGroup::new(f(cx)).into_element(cx)
}

#[derive(Debug, Clone)]
pub struct FieldContent {
    children: Vec<AnyElement>,
}

impl FieldContent {
    pub fn new(children: Vec<AnyElement>) -> Self {
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let gap = MetricRef::space(Space::N1p5).resolve(&theme);
        let layout =
            decl_style::layout_style(&theme, LayoutRefinement::default().flex_1().min_w_0());
        let children = self.children;
        cx.column(
            ColumnProps {
                layout,
                gap,
                ..Default::default()
            },
            move |_cx| children,
        )
    }
}

#[derive(Debug, Clone)]
pub struct FieldTitle {
    text: Arc<str>,
}

impl FieldTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let fg = theme
            .color_by_key("foreground")
            .unwrap_or(theme.colors.text_primary);
        let px = theme
            .metric_by_key("component.field.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or(theme.metrics.font_size);
        let line_height = theme
            .metric_by_key("component.field.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or(theme.metrics.font_line_height);

        cx.text_props(TextProps {
            layout: Default::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: px,
                weight: FontWeight::MEDIUM,
                line_height: Some(line_height),
                letter_spacing_em: None,
            }),
            color: Some(fg),
            wrap: TextWrap::None,
            overflow: TextOverflow::Clip,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FieldLabel {
    text: Arc<str>,
}

impl FieldLabel {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        shadcn_label(cx, self.text)
    }
}

#[derive(Debug, Clone)]
pub struct FieldDescription {
    text: Arc<str>,
}

impl FieldDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let fg = theme
            .color_by_key("muted-foreground")
            .or_else(|| theme.color_by_key("muted-foreground"))
            .unwrap_or(theme.colors.text_muted);
        let px = theme
            .metric_by_key("component.field.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or(theme.metrics.font_size);
        let line_height = theme
            .metric_by_key("component.field.description_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or(theme.metrics.font_line_height);

        cx.text_props(TextProps {
            layout: Default::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: px,
                weight: FontWeight::NORMAL,
                line_height: Some(line_height),
                letter_spacing_em: None,
            }),
            color: Some(fg),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FieldError {
    text: Arc<str>,
}

impl FieldError {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let fg = theme
            .color_by_key("destructive")
            .or_else(|| theme.color_by_key("destructive-foreground"))
            .or_else(|| theme.color_by_key("destructive-foreground"))
            .unwrap_or(theme.colors.viewport_gizmo_x);
        let px = theme
            .metric_by_key("component.field.error_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or(theme.metrics.font_size);
        let line_height = theme
            .metric_by_key("component.field.error_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or(theme.metrics.font_line_height);

        cx.text_props(TextProps {
            layout: Default::default(),
            text: self.text,
            style: Some(TextStyle {
                font: FontId::default(),
                size: px,
                weight: FontWeight::NORMAL,
                line_height: Some(line_height),
                letter_spacing_em: None,
            }),
            color: Some(fg),
            wrap: TextWrap::Word,
            overflow: TextOverflow::Clip,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FieldSeparator {
    label: Option<Arc<str>>,
}

impl FieldSeparator {
    pub fn new() -> Self {
        Self { label: None }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let h = theme
            .metric_by_key("component.field.separator_h")
            .unwrap_or_else(|| MetricRef::space(Space::N5).resolve(&theme));
        let border = theme
            .color_by_key("border")
            .unwrap_or(theme.colors.panel_border);
        let bg = theme
            .color_by_key("background")
            .unwrap_or(theme.colors.surface_background);

        let layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .relative()
                .w_full()
                .h_px(MetricRef::Px(h)),
        );

        cx.container(
            ContainerProps {
                layout,
                ..Default::default()
            },
            move |cx| {
                let mut children: Vec<AnyElement> = Vec::new();

                let line_layout = decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default()
                        .absolute()
                        .left(Space::N0)
                        .right(Space::N0)
                        .top(Space::N2p5)
                        .h_px(MetricRef::Px(Px(1.0))),
                );
                children.push(cx.container(
                    ContainerProps {
                        layout: line_layout,
                        border: Edges {
                            top: Px(1.0),
                            right: Px(0.0),
                            bottom: Px(0.0),
                            left: Px(0.0),
                        },
                        border_color: Some(border),
                        ..Default::default()
                    },
                    |_cx| Vec::new(),
                ));

                if let Some(label) = self.label.clone() {
                    let label_layout = decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default()
                            .absolute()
                            .left(Space::N0)
                            .right(Space::N0)
                            .top(Space::N0),
                    );
                    let chrome = ChromeRefinement::default()
                        .bg(fret_ui_kit::ColorRef::Color(bg))
                        .px(Space::N2);
                    let props =
                        decl_style::container_props(&theme, chrome, LayoutRefinement::default());
                    let mut props = ContainerProps { ..props };
                    props.layout = label_layout;
                    children.push(cx.container(props, move |cx| {
                        vec![FieldDescription::new(label).into_element(cx)]
                    }));
                }

                children
            },
        )
    }
}

impl Default for FieldSeparator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Field {
    orientation: FieldOrientation,
    label: Option<AnyElement>,
    control: AnyElement,
    description: Option<AnyElement>,
    error: Option<AnyElement>,
}

impl Field {
    pub fn new(control: AnyElement) -> Self {
        Self {
            orientation: FieldOrientation::default(),
            label: None,
            control,
            description: None,
            error: None,
        }
    }

    pub fn orientation(mut self, orientation: FieldOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn label(mut self, label: AnyElement) -> Self {
        self.label = Some(label);
        self
    }

    pub fn description(mut self, description: AnyElement) -> Self {
        self.description = Some(description);
        self
    }

    pub fn error(mut self, error: AnyElement) -> Self {
        self.error = Some(error);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let gap = MetricRef::space(Space::N3).resolve(&theme);
        let layout = decl_style::layout_style(&theme, LayoutRefinement::default().w_full());

        let label = self.label;
        let control = self.control;
        let description = self.description;
        let error = self.error;

        match self.orientation {
            FieldOrientation::Vertical => cx.column(
                ColumnProps {
                    layout,
                    gap,
                    ..Default::default()
                },
                move |_cx| {
                    let mut out: Vec<AnyElement> = Vec::new();
                    if let Some(label) = label {
                        out.push(label);
                    }
                    out.push(control);
                    if let Some(desc) = description {
                        out.push(desc);
                    }
                    if let Some(err) = error {
                        out.push(err);
                    }
                    out
                },
            ),
            FieldOrientation::Horizontal => {
                let label_layout = decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default().flex_1().min_w_0(),
                );
                let right_layout = decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default().flex_1().min_w_0(),
                );

                let label = label.map(|l| {
                    cx.container(
                        ContainerProps {
                            layout: label_layout,
                            ..Default::default()
                        },
                        move |_cx| vec![l],
                    )
                });

                let content = cx.column(
                    ColumnProps {
                        layout: right_layout,
                        gap: MetricRef::space(Space::N1p5).resolve(&theme),
                        ..Default::default()
                    },
                    move |_cx| {
                        let mut out: Vec<AnyElement> = Vec::new();
                        out.push(control);
                        if let Some(desc) = description {
                            out.push(desc);
                        }
                        if let Some(err) = error {
                            out.push(err);
                        }
                        out
                    },
                );

                cx.row(
                    RowProps {
                        layout,
                        gap,
                        justify: MainAlign::Start,
                        align: CrossAlign::Center,
                        ..Default::default()
                    },
                    move |_cx| {
                        let mut out: Vec<AnyElement> = Vec::new();
                        if let Some(label) = label {
                            out.push(label);
                        }
                        out.push(content);
                        out
                    },
                )
            }
        }
    }
}
