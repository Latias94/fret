use std::sync::Arc;

use fret_core::{Edges, Px, SemanticsRole, TextOverflow, TextWrap};
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, ElementKind, LayoutQueryRegionProps,
    MainAlign, RowProps, SemanticsDecoration,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Space, ui};

fn muted_foreground(theme: &Theme) -> fret_core::Color {
    theme
        .color_by_key("muted.foreground")
        .or_else(|| theme.color_by_key("muted-foreground"))
        .unwrap_or_else(|| theme.color_required("muted.foreground"))
}

fn peel_single_child_wrappers<'a>(mut element: &'a AnyElement) -> &'a AnyElement {
    loop {
        match &element.kind {
            ElementKind::Semantics(_) | ElementKind::Container(_) => {
                if element.children.len() == 1 {
                    element = &element.children[0];
                    continue;
                }
            }
            _ => {}
        }
        break element;
    }
}

fn peel_semantics_wrappers<'a>(mut element: &'a AnyElement) -> &'a AnyElement {
    loop {
        match &element.kind {
            ElementKind::Semantics(_) if element.children.len() == 1 => {
                element = &element.children[0];
                continue;
            }
            _ => break element,
        }
    }
}

fn is_field_legend_container(element: &AnyElement) -> bool {
    let element = peel_semantics_wrappers(element);
    let ElementKind::Container(props) = &element.kind else {
        return false;
    };

    let fret_ui::element::MarginEdge::Px(px) = props.layout.margin.bottom else {
        return false;
    };

    (px.0 - 12.0).abs() <= 0.5
}

fn is_field_legend_variant_legend(element: &AnyElement) -> bool {
    let element = peel_semantics_wrappers(element);
    let ElementKind::Container(_props) = &element.kind else {
        return false;
    };
    if element.children.len() != 1 {
        return false;
    }

    let child = peel_single_child_wrappers(&element.children[0]);
    let line_height = match &child.kind {
        ElementKind::Text(props) => props.style.as_ref().and_then(|s| s.line_height),
        ElementKind::StyledText(props) => props.style.as_ref().and_then(|s| s.line_height),
        _ => None,
    };

    line_height.is_some_and(|lh| (lh.0 - 24.0).abs() <= 0.5)
}

fn is_field_description(muted: fret_core::Color, element: &AnyElement) -> bool {
    let element = peel_single_child_wrappers(element);
    match &element.kind {
        ElementKind::Text(props) => props.wrap == TextWrap::Word && props.color == Some(muted),
        ElementKind::StyledText(props) => {
            props.wrap == TextWrap::Word && props.color == Some(muted)
        }
        _ => false,
    }
}

fn kind_flex_grow(kind: &ElementKind) -> Option<f32> {
    match kind {
        ElementKind::Container(props) => Some(props.layout.flex.grow),
        ElementKind::Semantics(props) => Some(props.layout.flex.grow),
        ElementKind::SemanticFlex(props) => Some(props.flex.layout.flex.grow),
        ElementKind::Pressable(props) => Some(props.layout.flex.grow),
        ElementKind::PointerRegion(props) => Some(props.layout.flex.grow),
        ElementKind::TextInputRegion(props) => Some(props.layout.flex.grow),
        ElementKind::InternalDragRegion(props) => Some(props.layout.flex.grow),
        ElementKind::Opacity(props) => Some(props.layout.flex.grow),
        ElementKind::InteractivityGate(props) => Some(props.layout.flex.grow),
        ElementKind::VisualTransform(props) => Some(props.layout.flex.grow),
        ElementKind::RenderTransform(props) => Some(props.layout.flex.grow),
        ElementKind::FractionalRenderTransform(props) => Some(props.layout.flex.grow),
        ElementKind::Anchored(props) => Some(props.layout.flex.grow),
        ElementKind::Column(props) => Some(props.layout.flex.grow),
        ElementKind::Row(props) => Some(props.layout.flex.grow),
        ElementKind::Stack(props) => Some(props.layout.flex.grow),
        ElementKind::Flex(props) => Some(props.layout.flex.grow),
        ElementKind::Grid(props) => Some(props.layout.flex.grow),
        ElementKind::Text(props) => Some(props.layout.flex.grow),
        ElementKind::StyledText(props) => Some(props.layout.flex.grow),
        ElementKind::SelectableText(props) => Some(props.layout.flex.grow),
        ElementKind::TextInput(props) => Some(props.layout.flex.grow),
        ElementKind::TextArea(props) => Some(props.layout.flex.grow),
        ElementKind::Image(props) => Some(props.layout.flex.grow),
        ElementKind::Canvas(props) => Some(props.layout.flex.grow),
        ElementKind::SvgIcon(props) => Some(props.layout.flex.grow),
        ElementKind::Spinner(props) => Some(props.layout.flex.grow),
        ElementKind::Scroll(props) => Some(props.layout.flex.grow),
        ElementKind::Scrollbar(props) => Some(props.layout.flex.grow),
        ElementKind::Spacer(props) => Some(props.layout.flex.grow),
        ElementKind::HoverRegion(props) => Some(props.layout.flex.grow),
        ElementKind::WheelRegion(props) => Some(props.layout.flex.grow),
        ElementKind::EffectLayer(props) => Some(props.layout.flex.grow),
        ElementKind::FocusScope(props) => Some(props.layout.flex.grow),
        ElementKind::RovingFlex(props) => Some(props.flex.layout.flex.grow),
        ElementKind::VirtualList(props) => Some(props.layout.flex.grow),
        ElementKind::ResizablePanelGroup(props) => Some(props.layout.flex.grow),
        ElementKind::ViewportSurface(props) => Some(props.layout.flex.grow),
        ElementKind::ViewCache(props) => Some(props.layout.flex.grow),
        _ => None,
    }
}

fn subtree_has_flex_grow(element: &AnyElement) -> bool {
    if kind_flex_grow(&element.kind).is_some_and(|grow| grow > 0.0) {
        return true;
    }
    element.children.iter().any(subtree_has_flex_grow)
}

fn responsive_md_width_auto(mut element: AnyElement) -> AnyElement {
    if subtree_has_flex_grow(&element) {
        return element;
    }

    match &mut element.kind {
        ElementKind::Semantics(props) => {
            props.layout.size.width = fret_ui::element::Length::Auto;
        }
        ElementKind::Container(props) => {
            props.layout.size.width = fret_ui::element::Length::Auto;
        }
        ElementKind::Pressable(props) => {
            props.layout.size.width = fret_ui::element::Length::Auto;
        }
        ElementKind::TextInput(props) => {
            // When shadcn's `Field(orientation="responsive")` flips to a row layout, upstream
            // applies `w-auto` to direct children via container queries. For `<input>` / `<textarea>`
            // this surfaces the HTML default `cols=20` intrinsic width (≈218px at `text-sm`).
            //
            // Fret's `TextInput` intrinsic sizing is placeholder/content driven, so we approximate
            // the browser behavior by explicitly setting a 20ch-like width derived from the input's
            // text size and chrome.
            let ch = props.text_style.size.0 * 0.685;
            let cols = 20.0;
            let chrome_w = props.chrome.padding.left.0
                + props.chrome.padding.right.0
                + props.chrome.border.left.0
                + props.chrome.border.right.0;
            props.layout.size.width = fret_ui::element::Length::Px(Px(ch * cols + chrome_w));
        }
        ElementKind::TextArea(props) => {
            let ch = props.text_style.size.0 * 0.685;
            let cols = 20.0;
            let chrome_w = props.chrome.padding_x.0 * 2.0
                + props.chrome.border.left.0
                + props.chrome.border.right.0;
            props.layout.size.width = fret_ui::element::Length::Px(Px(ch * cols + chrome_w));
        }
        _ => {}
    }

    element
}

fn is_radio_group_element(element: &AnyElement) -> bool {
    if element
        .semantics_decoration
        .as_ref()
        .and_then(|d| d.role)
        .is_some_and(|role| role == SemanticsRole::RadioGroup)
    {
        return true;
    }

    match &element.kind {
        ElementKind::Semantics(props) if props.role == SemanticsRole::RadioGroup => true,
        ElementKind::SemanticFlex(props) if props.role == SemanticsRole::RadioGroup => true,
        ElementKind::Pressable(props)
            if props
                .a11y
                .role
                .is_some_and(|role| role == SemanticsRole::RadioGroup) =>
        {
            true
        }
        _ => element.children.iter().any(is_radio_group_element),
    }
}

fn is_checkbox_group_element(element: &AnyElement) -> bool {
    if element
        .semantics_decoration
        .as_ref()
        .and_then(|d| d.role)
        .is_some_and(|role| role == SemanticsRole::List)
    {
        return true;
    }

    match &element.kind {
        ElementKind::Semantics(props) if props.role == SemanticsRole::List => true,
        ElementKind::SemanticFlex(props) if props.role == SemanticsRole::List => true,
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FieldOrientation {
    #[default]
    Vertical,
    Horizontal,
    /// Matches the upstream `orientation="responsive"` variant (container-query driven in web).
    ///
    /// In Fret we currently approximate the `@md/field-group` container query with a viewport-width
    /// breakpoint at `768px` (`md`).
    Responsive,
}

#[derive(Debug, Clone)]
pub struct FieldSet {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
}

impl FieldSet {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let has_radio_or_checkbox_group = self.children.iter().any(is_radio_group_element)
            || self.children.iter().any(is_checkbox_group_element);

        let (gap, layout, rest_layout, legend_gap, muted, desc_mt_neg_n1, desc_mt_neg_n1p5) = {
            let theme = Theme::global(&*cx.app);
            // Upstream `FieldSet` uses `gap-6`, but overrides to `gap-3` when a checkbox/radio group
            // is present via CSS `:has` selectors.
            let gap = if has_radio_or_checkbox_group {
                MetricRef::space(Space::N3).resolve(theme)
            } else {
                MetricRef::space(Space::N6).resolve(theme)
            };
            let layout = decl_style::layout_style(
                theme,
                LayoutRefinement::default().w_full().merge(self.layout),
            );
            let rest_layout = decl_style::layout_style(theme, LayoutRefinement::default().w_full());
            let legend_gap = MetricRef::space(Space::N3).resolve(theme);
            let muted = muted_foreground(theme);
            let desc_mt_neg_n1 =
                decl_style::layout_style(theme, LayoutRefinement::default().mt_neg(Space::N1));
            let desc_mt_neg_n1p5 =
                decl_style::layout_style(theme, LayoutRefinement::default().mt_neg(Space::N1p5));
            (
                gap,
                layout,
                rest_layout,
                legend_gap,
                muted,
                desc_mt_neg_n1,
                desc_mt_neg_n1p5,
            )
        };

        let mut children = self.children;
        let has_leading_legend =
            children.first().is_some_and(is_field_legend_container) && children.len() > 1;

        if has_leading_legend {
            let legend = children.remove(0);
            let legend_is_variant_legend = is_field_legend_variant_legend(&legend);
            let rest_children = children;
            let outer_gap = match &peel_semantics_wrappers(&legend).kind {
                ElementKind::Container(props) if matches!(props.layout.margin.bottom, fret_ui::element::MarginEdge::Px(px) if (px.0 - legend_gap.0).abs() <= 0.5) => {
                    Px(0.0)
                }
                _ => legend_gap,
            };

            cx.column(
                ColumnProps {
                    layout,
                    gap: outer_gap,
                    ..Default::default()
                },
                move |cx| {
                    let rest = cx.column(
                        ColumnProps {
                            layout: rest_layout,
                            gap,
                            ..Default::default()
                        },
                        move |cx| {
                            let legend_is_variant_legend = legend_is_variant_legend;
                            let len = rest_children.len();
                            rest_children
                                .into_iter()
                                .enumerate()
                                .map(|(idx, child)| {
                                    if len >= 2
                                        && idx == len - 2
                                        && is_field_description(muted, &child)
                                    {
                                        let layout = if legend_is_variant_legend && idx == 0 {
                                            desc_mt_neg_n1p5.clone()
                                        } else {
                                            desc_mt_neg_n1.clone()
                                        };
                                        cx.container(
                                            ContainerProps {
                                                layout,
                                                ..Default::default()
                                            },
                                            move |_cx| vec![child],
                                        )
                                    } else {
                                        child
                                    }
                                })
                                .collect::<Vec<_>>()
                        },
                    );

                    vec![legend, rest]
                },
            )
        } else {
            cx.column(
                ColumnProps {
                    layout,
                    gap,
                    ..Default::default()
                },
                move |cx| {
                    let len = children.len();
                    children
                        .into_iter()
                        .enumerate()
                        .map(|(idx, child)| {
                            if len >= 2 && idx == len - 2 && is_field_description(muted, &child) {
                                let layout = desc_mt_neg_n1.clone();
                                cx.container(
                                    ContainerProps {
                                        layout,
                                        ..Default::default()
                                    },
                                    move |_cx| vec![child],
                                )
                            } else {
                                child
                            }
                        })
                        .collect::<Vec<_>>()
                },
            )
        }
    }
}

pub fn field_set<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
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
    layout: LayoutRefinement,
}

impl FieldLegend {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            variant: FieldLegendVariant::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn variant(mut self, variant: FieldLegendVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (fg, size, line_height, layout) = {
            let theme = Theme::global(&*cx.app);
            let fg = theme
                .color_by_key("foreground")
                .unwrap_or_else(|| theme.color_required("foreground"));

            let (size, line_height) = match self.variant {
                FieldLegendVariant::Legend => {
                    // Tailwind `text-base` = 16px / 24px by default.
                    let size = theme
                        .metric_by_key("component.field.legend_px")
                        .unwrap_or(Px(16.0));
                    let line_height = theme
                        .metric_by_key("component.field.legend_line_height")
                        .unwrap_or(Px(24.0));
                    (size, line_height)
                }
                FieldLegendVariant::Label => {
                    let size = theme.metric_required("font.size");
                    let line_height = theme.metric_required("font.line_height");
                    (size, line_height)
                }
            };

            // Upstream has `mb-3` on the legend.
            let layout = decl_style::layout_style(
                theme,
                LayoutRefinement::default()
                    .w_full()
                    .mb(Space::N3)
                    .merge(self.layout),
            );

            (fg, size, line_height, layout)
        };

        let text = ui::label(cx, self.text)
            .w_full()
            .text_size_px(size)
            .line_height_px(line_height)
            .font_medium()
            .text_color(ColorRef::Color(fg))
            .wrap(TextWrap::Word)
            .into_element(cx);

        cx.container(
            ContainerProps {
                layout,
                ..Default::default()
            },
            move |_cx| vec![text],
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FieldGroupSlot {
    #[default]
    Default,
    CheckboxGroup,
}

#[derive(Debug, Clone)]
pub struct FieldGroup {
    children: Vec<AnyElement>,
    slot: FieldGroupSlot,
    gap: Option<MetricRef>,
    layout: LayoutRefinement,
}

impl FieldGroup {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            slot: FieldGroupSlot::default(),
            gap: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn checkbox_group(mut self) -> Self {
        self.slot = FieldGroupSlot::CheckboxGroup;
        self
    }

    pub fn gap(mut self, space: Space) -> Self {
        self.gap = Some(MetricRef::space(space));
        self
    }

    pub fn gap_px(mut self, px: Px) -> Self {
        self.gap = Some(px.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (gap, layout) = {
            let theme = Theme::global(&*cx.app);
            let gap = self
                .gap
                .map(|g| g.resolve(theme))
                .unwrap_or_else(|| match self.slot {
                    FieldGroupSlot::Default => theme
                        .metric_by_key("component.field.group_gap")
                        .unwrap_or_else(|| MetricRef::space(Space::N8).resolve(theme)),
                    FieldGroupSlot::CheckboxGroup => MetricRef::space(Space::N3).resolve(theme),
                });
            let layout = decl_style::layout_style(
                theme,
                LayoutRefinement::default().w_full().merge(self.layout),
            );
            (gap, layout)
        };
        let children = self.children;
        let column = cx.column(
            ColumnProps {
                layout,
                gap,
                ..Default::default()
            },
            move |_cx| children,
        );

        match self.slot {
            FieldGroupSlot::Default => column,
            FieldGroupSlot::CheckboxGroup => {
                column.attach_semantics(SemanticsDecoration::default().role(SemanticsRole::List))
            }
        }
    }
}

pub fn field_group<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    FieldGroup::new(f(cx)).into_element(cx)
}

#[derive(Debug, Clone)]
pub struct FieldContent {
    children: Vec<AnyElement>,
}

impl FieldContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (gap, layout) = {
            let theme = Theme::global(&*cx.app);
            let gap = MetricRef::space(Space::N1p5).resolve(theme);
            let layout =
                decl_style::layout_style(theme, LayoutRefinement::default().flex_1().min_w_0());
            (gap, layout)
        };
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
        let (fg, px, line_height) = {
            let theme = Theme::global(&*cx.app);
            let fg = theme
                .color_by_key("foreground")
                .unwrap_or_else(|| theme.color_required("foreground"));
            let px = theme
                .metric_by_key("component.field.title_px")
                .or_else(|| theme.metric_by_key("font.size"))
                .unwrap_or_else(|| theme.metric_required("font.size"));
            let line_height = theme
                .metric_by_key("component.field.title_line_height")
                .or_else(|| theme.metric_by_key("font.line_height"))
                .unwrap_or_else(|| theme.metric_required("font.line_height"));
            (fg, px, line_height)
        };

        ui::label(cx, self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_medium()
            .text_color(ColorRef::Color(fg))
            .wrap(TextWrap::Word)
            .into_element(cx)
    }
}

#[derive(Debug, Clone)]
pub struct FieldLabel {
    text: Arc<str>,
    layout: LayoutRefinement,
}

impl FieldLabel {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (fg, px, line_height) = {
            let theme = Theme::global(&*cx.app);

            // Upstream `FieldLabel` uses `leading-snug` instead of the plain `Label` default.
            // See: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/field.tsx`.
            let fg = theme
                .color_by_key("foreground")
                .unwrap_or_else(|| theme.color_required("foreground"));
            let px = theme
                .metric_by_key("component.field.label_px")
                .or_else(|| theme.metric_by_key("component.label.text_px"))
                .or_else(|| theme.metric_by_key("font.size"))
                .unwrap_or_else(|| theme.metric_required("font.size"));
            let line_height = theme
                .metric_by_key("component.field.label_line_height")
                .or_else(|| theme.metric_by_key("component.label.line_height"))
                .or_else(|| theme.metric_by_key("font.line_height"))
                .unwrap_or_else(|| theme.metric_required("font.line_height"));
            (fg, px, line_height)
        };

        ui::label(cx, self.text)
            .layout(self.layout)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_medium()
            .text_color(ColorRef::Color(fg))
            .wrap(TextWrap::Word)
            .into_element(cx)
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
        let (fg, px, line_height) = {
            let theme = Theme::global(&*cx.app);
            let fg = muted_foreground(theme);
            let px = theme
                .metric_by_key("component.field.description_px")
                .or_else(|| theme.metric_by_key("font.size"))
                .unwrap_or_else(|| theme.metric_required("font.size"));
            let line_height = theme
                .metric_by_key("component.field.description_line_height")
                .or_else(|| theme.metric_by_key("font.line_height"))
                .unwrap_or_else(|| theme.metric_required("font.line_height"));
            (fg, px, line_height)
        };

        ui::text(cx, self.text)
            .w_full()
            .text_size_px(px)
            .line_height_px(line_height)
            .font_normal()
            .text_color(ColorRef::Color(fg))
            .wrap(TextWrap::Word)
            .overflow(TextOverflow::Clip)
            .into_element(cx)
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
        let (fg, px, line_height) = {
            let theme = Theme::global(&*cx.app);
            let fg = theme.color_required("destructive");
            let px = theme
                .metric_by_key("component.field.error_px")
                .or_else(|| theme.metric_by_key("font.size"))
                .unwrap_or_else(|| theme.metric_required("font.size"));
            let line_height = theme
                .metric_by_key("component.field.error_line_height")
                .or_else(|| theme.metric_by_key("font.line_height"))
                .unwrap_or_else(|| theme.metric_required("font.line_height"));
            (fg, px, line_height)
        };

        ui::text(cx, self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_normal()
            .text_color(ColorRef::Color(fg))
            .wrap(TextWrap::Word)
            .overflow(TextOverflow::Clip)
            .into_element(cx)
    }
}

#[derive(Debug, Clone)]
pub struct FieldSeparator {
    label: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl FieldSeparator {
    pub fn new() -> Self {
        Self {
            label: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (outer_layout, layout, line_layout, label_props, border) = {
            let theme = Theme::global(&*cx.app);

            let h = theme
                .metric_by_key("component.field.separator_h")
                .unwrap_or_else(|| MetricRef::space(Space::N5).resolve(theme));
            let border = theme.color_required("border");
            let bg = theme.color_required("background");

            // Upstream uses `-my-2` (negative 8px) to visually tighten the separator in a group.
            let outer_layout = decl_style::layout_style(
                theme,
                LayoutRefinement::default()
                    .w_full()
                    .mt_neg(Space::N2)
                    .mb_neg(Space::N2)
                    .merge(self.layout),
            );

            let layout = decl_style::layout_style(
                theme,
                LayoutRefinement::default().relative().w_full().h_px(h),
            );

            let line_layout = decl_style::layout_style(
                theme,
                LayoutRefinement::default()
                    .absolute()
                    .left(Space::N0)
                    .right(Space::N0)
                    .top(Space::N2p5)
                    .h_px(Px(1.0)),
            );

            let label_layout = decl_style::layout_style(
                theme,
                LayoutRefinement::default()
                    .absolute()
                    .left(Space::N0)
                    .right(Space::N0)
                    .top(Space::N0),
            );
            let chrome = ChromeRefinement::default()
                .bg(fret_ui_kit::ColorRef::Color(bg))
                .px(Space::N2);
            let mut label_props =
                decl_style::container_props(theme, chrome, LayoutRefinement::default());
            label_props.layout = label_layout;

            (outer_layout, layout, line_layout, label_props, border)
        };

        cx.container(
            ContainerProps {
                layout: outer_layout,
                ..Default::default()
            },
            move |cx| {
                let label = self.label.clone();
                let separator = cx.container(
                    ContainerProps {
                        layout,
                        ..Default::default()
                    },
                    move |cx| {
                        let mut children: Vec<AnyElement> = Vec::new();

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

                        if let Some(label) = label {
                            children.push(cx.container(label_props, move |cx| {
                                vec![FieldDescription::new(label).into_element(cx)]
                            }));
                        }

                        children
                    },
                );

                vec![separator]
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
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Field {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            orientation: FieldOrientation::default(),
            children,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn orientation(mut self, orientation: FieldOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (gap, wrapper, inner_layout, muted, desc_mt_neg) = {
            let theme = Theme::global(&*cx.app);
            let gap = MetricRef::space(Space::N3).resolve(theme);
            let wrapper = decl_style::container_props(
                theme,
                self.chrome,
                LayoutRefinement::default().w_full().merge(self.layout),
            );
            let inner_layout =
                decl_style::layout_style(theme, LayoutRefinement::default().w_full().min_w_0());
            let muted = muted_foreground(theme);
            let desc_mt_neg =
                decl_style::layout_style(theme, LayoutRefinement::default().mt_neg(Space::N1));
            (gap, wrapper, inner_layout, muted, desc_mt_neg)
        };

        let orientation = self.orientation;
        let children = self.children;
        let align_horizontal = if children.iter().any(subtree_has_flex_grow) {
            CrossAlign::Start
        } else {
            CrossAlign::Center
        };

        let region_props = LayoutQueryRegionProps {
            layout: inner_layout.clone(),
            name: None,
        };

        fret_ui_kit::declarative::container_query_region_with_id(
            cx,
            "shadcn.field",
            region_props,
            move |cx, region_id| {
                vec![cx.container(wrapper, move |cx| {
                    let md_breakpoint = fret_ui_kit::declarative::container_breakpoints(
                        cx,
                        region_id,
                        Invalidation::Layout,
                        false,
                        &[(
                            fret_ui_kit::declarative::container_queries::tailwind::MD,
                            true,
                        )],
                        fret_ui_kit::declarative::ContainerQueryHysteresis::default(),
                    );

                    let inner = match orientation {
                        FieldOrientation::Vertical => cx.column(
                            ColumnProps {
                                layout: inner_layout.clone(),
                                gap,
                                ..Default::default()
                            },
                            move |cx| {
                                // Upstream `FieldDescription` includes `nth-last-2:-mt-1`.
                                let len = children.len();
                                children
                                    .into_iter()
                                    .enumerate()
                                    .map(|(idx, child)| {
                                        if len >= 2
                                            && idx == len - 2
                                            && is_field_description(muted, &child)
                                        {
                                            cx.container(
                                                ContainerProps {
                                                    layout: desc_mt_neg.clone(),
                                                    ..Default::default()
                                                },
                                                move |_cx| vec![child],
                                            )
                                        } else {
                                            child
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            },
                        ),
                        FieldOrientation::Horizontal => cx.row(
                            RowProps {
                                layout: inner_layout,
                                gap,
                                justify: MainAlign::Start,
                                align: align_horizontal,
                                ..Default::default()
                            },
                            move |_cx| children,
                        ),
                        FieldOrientation::Responsive => {
                            let children_row = children.clone();
                            let children_col = children;
                            if md_breakpoint {
                                let children_row = children_row
                                    .into_iter()
                                    .map(responsive_md_width_auto)
                                    .collect::<Vec<_>>();
                                cx.row(
                                    RowProps {
                                        layout: inner_layout,
                                        gap,
                                        justify: MainAlign::Start,
                                        align: align_horizontal,
                                        ..Default::default()
                                    },
                                    move |_cx| children_row,
                                )
                            } else {
                                cx.column(
                                    ColumnProps {
                                        layout: inner_layout.clone(),
                                        gap,
                                        ..Default::default()
                                    },
                                    move |cx| {
                                        // Upstream `FieldDescription` includes `nth-last-2:-mt-1`.
                                        let len = children_col.len();
                                        children_col
                                            .into_iter()
                                            .enumerate()
                                            .map(|(idx, child)| {
                                                if len >= 2
                                                    && idx == len - 2
                                                    && is_field_description(muted, &child)
                                                {
                                                    cx.container(
                                                        ContainerProps {
                                                            layout: desc_mt_neg.clone(),
                                                            ..Default::default()
                                                        },
                                                        move |_cx| vec![child],
                                                    )
                                                } else {
                                                    child
                                                }
                                            })
                                            .collect::<Vec<_>>()
                                    },
                                )
                            }
                        }
                    };

                    vec![inner]
                })]
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};

    #[test]
    fn checkbox_group_stamps_list_role_without_layout_wrapper() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let bounds = Rect::new(Point::new(Px(0.0), Px(0.0)), Size::new(Px(120.0), Px(80.0)));

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            FieldGroup::new([cx.text("A")])
                .checkbox_group()
                .into_element(cx)
        });

        assert!(
            !matches!(element.kind, ElementKind::Semantics(_)),
            "expected checkbox group to avoid `Semantics` wrappers; use `attach_semantics` instead"
        );
        assert_eq!(
            element.semantics_decoration.as_ref().and_then(|d| d.role),
            Some(SemanticsRole::List)
        );
    }
}
