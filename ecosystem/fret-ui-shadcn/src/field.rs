use std::sync::Arc;

use crate::LayoutDirection;
use fret_core::{Edges, Px, SemanticsLive, SemanticsRole, TextAlign, TextOverflow, TextWrap};
use fret_ui::element::{
    AnyElement, ColumnProps, ContainerProps, CrossAlign, ElementKind, LayoutQueryRegionProps,
    LayoutStyle, MainAlign, PointerRegionProps, RowProps, SemanticsDecoration, SemanticsProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::primitives::control_registry::{
    ControlAction, ControlId, DescriptionEntry, ErrorEntry, LabelEntry, control_registry_model,
};
use fret_ui_kit::primitives::field_state as field_state_prim;
use fret_ui_kit::theme_tokens;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Space, ui};

use fret_ui_kit::typography::{
    description_text_refinement_with_fallbacks, muted_foreground_color,
    scope_description_text_with_fallbacks,
};

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

fn passive_text_color(element: &AnyElement) -> Option<fret_core::Color> {
    match &element.kind {
        ElementKind::Text(props) => props.color.or(element.inherited_foreground),
        ElementKind::StyledText(props) => props.color.or(element.inherited_foreground),
        ElementKind::SelectableText(props) => props.color.or(element.inherited_foreground),
        _ => None,
    }
}

fn passive_text_line_height(element: &AnyElement) -> Option<Px> {
    let explicit = match &element.kind {
        ElementKind::Text(props) => props.style.as_ref().and_then(|style| style.line_height),
        ElementKind::StyledText(props) => props.style.as_ref().and_then(|style| style.line_height),
        ElementKind::SelectableText(props) => {
            props.style.as_ref().and_then(|style| style.line_height)
        }
        _ => return None,
    };

    explicit.or_else(|| {
        element
            .inherited_text_style
            .as_ref()
            .and_then(|style| style.line_height)
    })
}

fn is_field_description(
    muted: fret_core::Color,
    desc_line_height: Px,
    element: &AnyElement,
) -> bool {
    let element = peel_single_child_wrappers(element);
    let is_wrapped_text = match &element.kind {
        ElementKind::Text(props) => matches!(
            props.wrap,
            TextWrap::Word | TextWrap::Balance | TextWrap::WordBreak | TextWrap::Grapheme
        ),
        ElementKind::StyledText(props) => matches!(
            props.wrap,
            TextWrap::Word | TextWrap::Balance | TextWrap::WordBreak | TextWrap::Grapheme
        ),
        ElementKind::SelectableText(props) => matches!(
            props.wrap,
            TextWrap::Word | TextWrap::Balance | TextWrap::WordBreak | TextWrap::Grapheme
        ),
        _ => false,
    };

    is_wrapped_text
        && passive_text_color(element) == Some(muted)
        && passive_text_line_height(element)
            .is_some_and(|lh| (lh.0 - desc_line_height.0).abs() <= 0.5)
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

#[cfg(test)]
fn kind_layout(kind: &ElementKind) -> Option<&LayoutStyle> {
    match kind {
        ElementKind::Container(props) => Some(&props.layout),
        ElementKind::Semantics(props) => Some(&props.layout),
        ElementKind::SemanticFlex(props) => Some(&props.flex.layout),
        ElementKind::Pressable(props) => Some(&props.layout),
        ElementKind::PointerRegion(props) => Some(&props.layout),
        ElementKind::TextInputRegion(props) => Some(&props.layout),
        ElementKind::InternalDragRegion(props) => Some(&props.layout),
        ElementKind::Opacity(props) => Some(&props.layout),
        ElementKind::InteractivityGate(props) => Some(&props.layout),
        ElementKind::VisualTransform(props) => Some(&props.layout),
        ElementKind::RenderTransform(props) => Some(&props.layout),
        ElementKind::FractionalRenderTransform(props) => Some(&props.layout),
        ElementKind::Anchored(props) => Some(&props.layout),
        ElementKind::Column(props) => Some(&props.layout),
        ElementKind::Row(props) => Some(&props.layout),
        ElementKind::Stack(props) => Some(&props.layout),
        ElementKind::Flex(props) => Some(&props.layout),
        ElementKind::Grid(props) => Some(&props.layout),
        ElementKind::Text(props) => Some(&props.layout),
        ElementKind::StyledText(props) => Some(&props.layout),
        ElementKind::SelectableText(props) => Some(&props.layout),
        ElementKind::TextInput(props) => Some(&props.layout),
        ElementKind::TextArea(props) => Some(&props.layout),
        ElementKind::Image(props) => Some(&props.layout),
        ElementKind::Canvas(props) => Some(&props.layout),
        ElementKind::SvgIcon(props) => Some(&props.layout),
        ElementKind::Spinner(props) => Some(&props.layout),
        ElementKind::Scroll(props) => Some(&props.layout),
        ElementKind::Scrollbar(props) => Some(&props.layout),
        ElementKind::Spacer(props) => Some(&props.layout),
        ElementKind::HoverRegion(props) => Some(&props.layout),
        ElementKind::WheelRegion(props) => Some(&props.layout),
        ElementKind::EffectLayer(props) => Some(&props.layout),
        ElementKind::FocusScope(props) => Some(&props.layout),
        ElementKind::RovingFlex(props) => Some(&props.flex.layout),
        ElementKind::VirtualList(props) => Some(&props.layout),
        ElementKind::ResizablePanelGroup(props) => Some(&props.layout),
        ElementKind::ViewportSurface(props) => Some(&props.layout),
        ElementKind::ViewCache(props) => Some(&props.layout),
        _ => None,
    }
}

fn kind_layout_mut(kind: &mut ElementKind) -> Option<&mut LayoutStyle> {
    match kind {
        ElementKind::Container(props) => Some(&mut props.layout),
        ElementKind::Semantics(props) => Some(&mut props.layout),
        ElementKind::SemanticFlex(props) => Some(&mut props.flex.layout),
        ElementKind::Pressable(props) => Some(&mut props.layout),
        ElementKind::PointerRegion(props) => Some(&mut props.layout),
        ElementKind::TextInputRegion(props) => Some(&mut props.layout),
        ElementKind::InternalDragRegion(props) => Some(&mut props.layout),
        ElementKind::Opacity(props) => Some(&mut props.layout),
        ElementKind::InteractivityGate(props) => Some(&mut props.layout),
        ElementKind::VisualTransform(props) => Some(&mut props.layout),
        ElementKind::RenderTransform(props) => Some(&mut props.layout),
        ElementKind::FractionalRenderTransform(props) => Some(&mut props.layout),
        ElementKind::Anchored(props) => Some(&mut props.layout),
        ElementKind::Column(props) => Some(&mut props.layout),
        ElementKind::Row(props) => Some(&mut props.layout),
        ElementKind::Stack(props) => Some(&mut props.layout),
        ElementKind::Flex(props) => Some(&mut props.layout),
        ElementKind::Grid(props) => Some(&mut props.layout),
        ElementKind::Text(props) => Some(&mut props.layout),
        ElementKind::StyledText(props) => Some(&mut props.layout),
        ElementKind::SelectableText(props) => Some(&mut props.layout),
        ElementKind::TextInput(props) => Some(&mut props.layout),
        ElementKind::TextArea(props) => Some(&mut props.layout),
        ElementKind::Image(props) => Some(&mut props.layout),
        ElementKind::Canvas(props) => Some(&mut props.layout),
        ElementKind::SvgIcon(props) => Some(&mut props.layout),
        ElementKind::Spinner(props) => Some(&mut props.layout),
        ElementKind::Scroll(props) => Some(&mut props.layout),
        ElementKind::Scrollbar(props) => Some(&mut props.layout),
        ElementKind::Spacer(props) => Some(&mut props.layout),
        ElementKind::HoverRegion(props) => Some(&mut props.layout),
        ElementKind::WheelRegion(props) => Some(&mut props.layout),
        ElementKind::EffectLayer(props) => Some(&mut props.layout),
        ElementKind::FocusScope(props) => Some(&mut props.layout),
        ElementKind::RovingFlex(props) => Some(&mut props.flex.layout),
        ElementKind::VirtualList(props) => Some(&mut props.layout),
        ElementKind::ResizablePanelGroup(props) => Some(&mut props.layout),
        ElementKind::ViewportSurface(props) => Some(&mut props.layout),
        ElementKind::ViewCache(props) => Some(&mut props.layout),
        _ => None,
    }
}

fn responsive_md_content_flex_1_min_w_0(mut element: AnyElement) -> AnyElement {
    let Some(layout) = kind_layout_mut(&mut element.kind) else {
        return element;
    };

    layout.flex.grow = 1.0;
    layout.flex.shrink = 1.0;
    layout.flex.basis = fret_ui::element::Length::Px(Px(0.0));
    layout.size.min_width = Some(fret_ui::element::Length::Px(Px(0.0)));

    element
}

fn approx_w_fit_under_stretch(mut element: AnyElement) -> AnyElement {
    let Some(layout) = kind_layout_mut(&mut element.kind) else {
        return element;
    };

    if matches!(layout.size.width, fret_ui::element::Length::Auto)
        && layout.flex.align_self.is_none()
    {
        layout.flex.align_self = Some(CrossAlign::Start);
    }

    element
}

fn responsive_md_width_auto(mut element: AnyElement) -> AnyElement {
    if kind_flex_grow(&element.kind).is_some_and(|grow| grow > 0.0) {
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
            // this surfaces the HTML default `cols=20` intrinsic width (~218px at `text-sm`).
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

    if matches!(
        element.kind,
        ElementKind::Semantics(_) | ElementKind::Container(_) | ElementKind::Pressable(_)
    ) && element.children.len() == 1
    {
        let child = element.children.remove(0);
        element.children.push(responsive_md_width_auto(child));
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
    Responsive,
}

#[derive(Debug)]
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let has_radio_or_checkbox_group = self.children.iter().any(is_radio_group_element)
            || self.children.iter().any(is_checkbox_group_element);

        let (
            gap,
            layout,
            rest_layout,
            legend_gap,
            muted,
            desc_mt_neg_n1,
            desc_mt_neg_n1p5,
            desc_line_height,
        ) = {
            let theme = Theme::global(&*cx.app).snapshot();
            // Upstream `FieldSet` uses `gap-6`, but overrides to `gap-3` when a checkbox/radio group
            // is present via CSS `:has` selectors.
            let gap = if has_radio_or_checkbox_group {
                MetricRef::space(Space::N3).resolve(&theme)
            } else {
                MetricRef::space(Space::N6).resolve(&theme)
            };
            let layout = decl_style::layout_style(
                &theme,
                LayoutRefinement::default().w_full().merge(self.layout),
            );
            let rest_layout =
                decl_style::layout_style(&theme, LayoutRefinement::default().w_full());
            let legend_gap = MetricRef::space(Space::N3).resolve(&theme);
            let muted = muted_foreground_color(&theme);
            let desc_mt_neg_n1 =
                decl_style::layout_style(&theme, LayoutRefinement::default().mt_neg(Space::N1));
            let desc_mt_neg_n1p5 =
                decl_style::layout_style(&theme, LayoutRefinement::default().mt_neg(Space::N1p5));
            let desc_line_height = description_text_refinement_with_fallbacks(
                &theme,
                "component.field.description",
                Some(theme_tokens::metric::COMPONENT_TEXT_SM_PX),
                Some(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT),
            )
            .line_height
            .unwrap_or_else(|| theme.metric_token("font.line_height"));
            (
                gap,
                layout,
                rest_layout,
                legend_gap,
                muted,
                desc_mt_neg_n1,
                desc_mt_neg_n1p5,
                desc_line_height,
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
                    gap: outer_gap.into(),
                    ..Default::default()
                },
                move |cx| {
                    let rest = cx.column(
                        ColumnProps {
                            layout: rest_layout,
                            gap: gap.into(),
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
                                        && is_field_description(muted, desc_line_height, &child)
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
                    gap: gap.into(),
                    ..Default::default()
                },
                move |cx| {
                    let len = children.len();
                    children
                        .into_iter()
                        .enumerate()
                        .map(|(idx, child)| {
                            if len >= 2
                                && idx == len - 2
                                && is_field_description(muted, desc_line_height, &child)
                            {
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (fg, size, line_height, layout) = {
            let theme = Theme::global(&*cx.app);
            let fg = theme
                .color_by_key("foreground")
                .unwrap_or_else(|| theme.color_token("foreground"));

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
                    let size = theme.metric_token("font.size");
                    let line_height = theme.metric_token("font.line_height");
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

        let text = ui::label(self.text)
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

#[derive(Debug)]
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
            gap: None.into(),
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

    #[track_caller]
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
                gap: gap.into(),
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

#[derive(Debug)]
pub struct FieldContent {
    children: Vec<AnyElement>,
}

impl FieldContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    #[track_caller]
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
                gap: gap.into(),
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (fg, px, line_height) = {
            let theme = Theme::global(&*cx.app);
            let field_state = field_state_prim::use_field_state_in_scope(cx, None);
            let fg = if field_state.invalid {
                theme.color_token("destructive")
            } else {
                theme
                    .color_by_key("foreground")
                    .unwrap_or_else(|| theme.color_token("foreground"))
            };
            let px = theme
                .metric_by_key("component.field.title_px")
                .or_else(|| theme.metric_by_key("font.size"))
                .unwrap_or_else(|| theme.metric_token("font.size"));
            let line_height = theme
                .metric_by_key("component.field.title_line_height")
                .or_else(|| theme.metric_by_key("font.line_height"))
                .unwrap_or_else(|| theme.metric_token("font.line_height"));
            (fg, px, line_height)
        };

        let align = match crate::use_direction(cx, None) {
            LayoutDirection::Rtl => TextAlign::End,
            LayoutDirection::Ltr => TextAlign::Start,
        };
        let el = approx_w_fit_under_stretch(
            ui::label(self.text)
                .text_size_px(px)
                .line_height_px(line_height)
                .font_medium()
                .text_color(ColorRef::Color(fg))
                .wrap(TextWrap::Word)
                .text_align(align)
                .into_element(cx),
        );

        let field_state = field_state_prim::use_field_state_in_scope(cx, None);
        if field_state.disabled {
            cx.opacity(0.5, |_cx| vec![el])
        } else {
            el
        }
    }
}

#[derive(Debug)]
pub struct FieldLabel {
    text: Arc<str>,
    layout: LayoutRefinement,
    for_control: Option<ControlId>,
    children: Option<Vec<AnyElement>>,
    render_text: bool,
    text_color: Option<ColorRef>,
    test_id: Option<Arc<str>>,
}

impl FieldLabel {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            layout: LayoutRefinement::default(),
            for_control: None,
            children: None,
            render_text: true,
            text_color: None,
            test_id: None,
        }
    }

    /// Wrap an arbitrary subtree in a clickable label that forwards activation to `for_control`.
    ///
    /// This aligns with upstream shadcn/ui's `<FieldLabel htmlFor="...">...</FieldLabel>` usage
    /// where the label can contain a full `Field` layout (title + description + control).
    ///
    /// By default, calling this disables rendering of `text` (it is still used for accessibility
    /// label and diagnostics surfaces). Set [`Self::render_text`] back to `true` if you want both.
    pub fn wrap(mut self, children: impl IntoIterator<Item = AnyElement>) -> Self {
        self.children = Some(children.into_iter().collect());
        self.render_text = false;
        self
    }

    /// Controls whether the label's `text` is rendered as a visible child.
    pub fn render_text(mut self, render_text: bool) -> Self {
        self.render_text = render_text;
        self
    }

    /// Binds this label to a logical form control id (similar to HTML `label[for]`).
    ///
    /// When set, pointer activation on the label forwards to the registered control action and
    /// requests focus for the control.
    pub fn for_control(mut self, id: impl Into<ControlId>) -> Self {
        self.for_control = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn text_color(mut self, text_color: ColorRef) -> Self {
        self.text_color = Some(text_color);
        self
    }

    pub fn test_id(mut self, test_id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(test_id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (default_fg, px, line_height) = {
            let theme = Theme::global(&*cx.app);

            // Upstream `FieldLabel` uses `leading-snug` instead of the plain `Label` default.
            // See: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/field.tsx`.
            let fg = theme
                .color_by_key("foreground")
                .unwrap_or_else(|| theme.color_token("foreground"));
            let px = theme
                .metric_by_key("component.field.label_px")
                .or_else(|| theme.metric_by_key("component.label.text_px"))
                .or_else(|| theme.metric_by_key("font.size"))
                .unwrap_or_else(|| theme.metric_token("font.size"));
            let line_height = theme
                .metric_by_key("component.field.label_line_height")
                .or_else(|| theme.metric_by_key("component.label.line_height"))
                .or_else(|| theme.metric_by_key("font.line_height"))
                .unwrap_or_else(|| theme.metric_token("font.line_height"));
            (fg, px, line_height)
        };

        let field_state = field_state_prim::use_field_state_in_scope(cx, None);
        let fg = if let Some(fg) = self.text_color {
            fg
        } else if field_state.invalid {
            ColorRef::Color(Theme::global(&*cx.app).color_token("destructive"))
        } else {
            ColorRef::Color(default_fg)
        };

        let wrap_children = self.children;
        let Some(for_control) = self.for_control else {
            let align = match crate::use_direction(cx, None) {
                LayoutDirection::Rtl => TextAlign::End,
                LayoutDirection::Ltr => TextAlign::Start,
            };
            let mut el = if let Some(children) = wrap_children {
                let theme = Theme::global(&*cx.app);
                let border = theme.color_token("border");
                let wrapper = decl_style::container_props(
                    theme,
                    ChromeRefinement::default()
                        .rounded_md()
                        .border_1()
                        .border_color(ColorRef::Color(border))
                        .p_4(),
                    LayoutRefinement::default().w_full().merge(self.layout),
                );
                cx.container(wrapper, move |_cx| children)
            } else {
                approx_w_fit_under_stretch(
                    ui::label(self.text)
                        .layout(self.layout)
                        .text_size_px(px)
                        .line_height_px(line_height)
                        .font_medium()
                        .text_color(fg)
                        .wrap(TextWrap::Word)
                        .text_align(align)
                        .into_element(cx),
                )
            };

            if let Some(test_id) = self.test_id {
                el = el.test_id(test_id);
            }

            if field_state.disabled {
                return cx.opacity(0.5, |_cx| vec![el]);
            }
            return el;
        };

        let theme = Theme::global(&*cx.app).snapshot();
        let pressable_layout_default = if wrap_children.is_some() {
            LayoutRefinement::default().w_full().min_w_0()
        } else {
            // `FieldLabel` is commonly used inside `FieldContent` (a vertical column). Using
            // `flex-1` here makes the label a main-axis flex item, which can collapse to a zero
            // main size in auto-sized columns and cause visible overlap with sibling text.
            //
            // Keep the default layout non-flex; parent recipes can opt into `flex-1` explicitly
            // when they truly need a horizontally flexible label in a row.
            LayoutRefinement::default().min_w_0()
        };
        let pressable_layout =
            decl_style::layout_style(&theme, pressable_layout_default.merge(self.layout));
        let render_text_block =
            !matches!(pressable_layout.size.width, fret_ui::element::Length::Auto);
        let control_registry = control_registry_model(cx);
        let text = self.text.clone();
        let fg = fg.clone();
        let test_id = self.test_id.clone();
        let render_text = self.render_text;

        let control_snapshot = cx
            .app
            .models()
            .read(&control_registry, |reg| {
                reg.control_for(cx.window, &for_control).cloned()
            })
            .ok()
            .flatten();
        let controls_element = control_snapshot.as_ref().map(|c| c.element.0);
        let enabled = control_snapshot.as_ref().map(|c| c.enabled).unwrap_or(true);
        let semantics_props = SemanticsProps {
            role: SemanticsRole::Text,
            label: Some(text.clone()),
            test_id: test_id.clone(),
            controls_element,
            disabled: !enabled,
            ..Default::default()
        };

        let el = cx.semantics(semantics_props, move |cx| {
            let id = cx.root_id();
            let _ = cx.app.models_mut().update(&control_registry, |reg| {
                reg.register_label(
                    cx.window,
                    cx.frame_id,
                    for_control.clone(),
                    LabelEntry { element: id },
                );
            });

            let mut interactive_layout = LayoutStyle::default();
            interactive_layout.size.width = fret_ui::element::Length::Fill;
            if !matches!(pressable_layout.size.height, fret_ui::element::Length::Auto) {
                interactive_layout.size.height = fret_ui::element::Length::Fill;
            }

            let control_snapshot = cx
                .app
                .models()
                .read(&control_registry, |reg| {
                    reg.control_for(cx.window, &for_control).cloned()
                })
                .ok()
                .flatten();

            vec![cx.container(
                ContainerProps {
                    layout: pressable_layout,
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.pointer_region(
                        PointerRegionProps {
                            layout: interactive_layout,
                            enabled: true,
                            ..Default::default()
                        },
                        move |cx| {
                            let control_registry_on_pointer = control_registry.clone();
                            let for_control_on_pointer = for_control.clone();
                            let control_snapshot_on_pointer = control_snapshot.clone();
                            cx.pointer_region_add_on_pointer_down(Arc::new(
                                move |host, acx, down| {
                                    if down.hit_pressable_target.is_some() {
                                        return false;
                                    }

                                    let target = host
                                        .models_mut()
                                        .read(&control_registry_on_pointer, |reg| {
                                            reg.control_for(acx.window, &for_control_on_pointer)
                                                .map(|c| {
                                                    (
                                                        c.enabled,
                                                        c.element,
                                                        matches!(
                                                            c.action,
                                                            ControlAction::FocusOnly
                                                        ),
                                                    )
                                                })
                                        })
                                        .ok()
                                        .flatten()
                                        .or_else(|| {
                                            control_snapshot_on_pointer.as_ref().map(|c| {
                                                (
                                                    c.enabled,
                                                    c.element,
                                                    matches!(c.action, ControlAction::FocusOnly),
                                                )
                                            })
                                        });
                                    if let Some((true, element, focus_on_pointer_down)) = target {
                                        if focus_on_pointer_down {
                                            host.request_focus(element);
                                            return true;
                                        }
                                        host.capture_pointer();
                                    }
                                    true
                                },
                            ));

                            let control_registry_on_pointer_up = control_registry.clone();
                            let for_control_on_pointer_up = for_control.clone();
                            let control_snapshot_on_pointer_up = control_snapshot.clone();
                            cx.pointer_region_add_on_pointer_up(Arc::new(move |host, acx, up| {
                                host.release_pointer_capture();
                                if !up.is_click {
                                    return true;
                                }
                                if up.down_hit_pressable_target.is_some() {
                                    return false;
                                }
                                let control = host
                                    .models_mut()
                                    .read(&control_registry_on_pointer_up, |reg| {
                                        reg.control_for(acx.window, &for_control_on_pointer_up)
                                            .cloned()
                                    })
                                    .ok()
                                    .flatten();
                                let Some(control) =
                                    control.or_else(|| control_snapshot_on_pointer_up.clone())
                                else {
                                    return true;
                                };
                                if !control.enabled {
                                    return true;
                                }

                                let focus_only = matches!(control.action, ControlAction::FocusOnly);
                                if focus_only {
                                    return true;
                                }
                                host.request_focus(control.element);
                                control.action.invoke(host, acx);
                                host.request_redraw(acx.window);
                                true
                            }));

                            if let Some(children) = wrap_children {
                                let theme = Theme::global(&*cx.app);
                                let border = theme.color_token("border");
                                let wrapper = decl_style::container_props(
                                    theme,
                                    ChromeRefinement::default()
                                        .rounded_md()
                                        .border_1()
                                        .border_color(ColorRef::Color(border))
                                        .p_4(),
                                    LayoutRefinement::default(),
                                );
                                let inner = cx.container(wrapper, move |_cx| children);
                                vec![inner]
                            } else if render_text {
                                let align = match crate::use_direction(cx, None) {
                                    LayoutDirection::Rtl => TextAlign::End,
                                    LayoutDirection::Ltr => TextAlign::Start,
                                };
                                let mut builder = ui::label(text.clone());
                                if render_text_block {
                                    builder = builder.w_full().min_w_0();
                                }
                                let label = builder
                                    .text_size_px(px)
                                    .line_height_px(line_height)
                                    .font_medium()
                                    .text_color(fg.clone())
                                    .wrap(TextWrap::Word)
                                    .text_align(align)
                                    .into_element(cx);
                                vec![label]
                            } else {
                                Vec::new()
                            }
                        },
                    )]
                },
            )]
        });

        if field_state.disabled {
            cx.opacity(0.5, |_cx| vec![el])
        } else {
            el
        }
    }
}

#[derive(Debug, Clone)]
pub struct FieldDescription {
    text: Arc<str>,
    for_control: Option<ControlId>,
    wrap: Option<TextWrap>,
    overflow: Option<TextOverflow>,
}

impl FieldDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            for_control: None,
            wrap: None,
            overflow: None,
        }
    }

    pub fn for_control(mut self, id: impl Into<ControlId>) -> Self {
        self.for_control = Some(id.into());
        self
    }

    pub fn wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = Some(wrap);
        self
    }

    pub fn overflow(mut self, overflow: TextOverflow) -> Self {
        self.overflow = Some(overflow);
        self
    }

    pub fn text_balance(self) -> Self {
        self.wrap(TextWrap::Balance)
    }

    pub fn break_words(self) -> Self {
        self.wrap(TextWrap::WordBreak)
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();

        let align = match crate::use_direction(cx, None) {
            LayoutDirection::Rtl => TextAlign::End,
            LayoutDirection::Ltr => TextAlign::Start,
        };
        let wrap = self.wrap.unwrap_or(TextWrap::Word);
        let overflow = self.overflow.unwrap_or(TextOverflow::Clip);
        let el = scope_description_text_with_fallbacks(
            ui::raw_text(self.text)
                .wrap(wrap)
                .overflow(overflow)
                .text_align(align)
                .w_full()
                .min_w_0()
                .into_element(cx),
            &theme,
            "component.field.description",
            Some(theme_tokens::metric::COMPONENT_TEXT_SM_PX),
            Some(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT),
        );

        if let Some(for_control) = self.for_control {
            let control_registry = control_registry_model(cx);
            let _ = cx.app.models_mut().update(&control_registry, |reg| {
                reg.register_description(
                    cx.window,
                    cx.frame_id,
                    for_control,
                    DescriptionEntry { element: el.id },
                );
            });
        }

        el
    }
}

#[derive(Debug, Clone)]
pub struct FieldError {
    text: Arc<str>,
    for_control: Option<ControlId>,
    wrap: Option<TextWrap>,
    overflow: Option<TextOverflow>,
}

impl FieldError {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            for_control: None,
            wrap: None,
            overflow: None,
        }
    }

    pub fn for_control(mut self, id: impl Into<ControlId>) -> Self {
        self.for_control = Some(id.into());
        self
    }

    pub fn wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = Some(wrap);
        self
    }

    pub fn overflow(mut self, overflow: TextOverflow) -> Self {
        self.overflow = Some(overflow);
        self
    }

    pub fn text_balance(self) -> Self {
        self.wrap(TextWrap::Balance)
    }

    pub fn break_words(self) -> Self {
        self.wrap(TextWrap::WordBreak)
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (fg, px, line_height) = {
            let theme = Theme::global(&*cx.app);
            let fg = theme.color_token("destructive");
            let px = theme
                .metric_by_key("component.field.error_px")
                .or_else(|| theme.metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_PX))
                .or_else(|| theme.metric_by_key("font.size"))
                .unwrap_or_else(|| theme.metric_token("font.size"));
            let line_height = theme
                .metric_by_key("component.field.error_line_height")
                .or_else(|| {
                    theme.metric_by_key(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT)
                })
                .or_else(|| theme.metric_by_key("font.line_height"))
                .unwrap_or_else(|| theme.metric_token("font.line_height"));
            (fg, px, line_height)
        };

        let align = match crate::use_direction(cx, None) {
            LayoutDirection::Rtl => TextAlign::End,
            LayoutDirection::Ltr => TextAlign::Start,
        };
        let wrap = self.wrap.unwrap_or(TextWrap::Word);
        let overflow = self.overflow.unwrap_or(TextOverflow::Clip);
        let el = ui::text(self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .line_height_policy(fret_core::TextLineHeightPolicy::FixedFromStyle)
            .font_normal()
            .text_color(ColorRef::Color(fg))
            .wrap(wrap)
            .overflow(overflow)
            .text_align(align)
            .w_full()
            .min_w_0()
            .into_element(cx)
            .attach_semantics(
                SemanticsDecoration::default()
                    .role(SemanticsRole::Alert)
                    .live(Some(SemanticsLive::Assertive))
                    .live_atomic(true),
            );

        if let Some(for_control) = self.for_control {
            let control_registry = control_registry_model(cx);
            let _ = cx.app.models_mut().update(&control_registry, |reg| {
                reg.register_error(
                    cx.window,
                    cx.frame_id,
                    for_control,
                    ErrorEntry { element: el.id },
                );
            });
        }

        el
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (outer_layout, layout, line_layout, label_props, border) = {
            let theme = Theme::global(&*cx.app);

            let h = theme
                .metric_by_key("component.field.separator_h")
                .unwrap_or_else(|| MetricRef::space(Space::N5).resolve(theme));
            let border = theme.color_token("border");
            let bg = theme.color_token("background");

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

#[derive(Debug)]
pub struct Field {
    orientation: FieldOrientation,
    children: Vec<AnyElement>,
    invalid: bool,
    disabled: bool,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Field {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            orientation: FieldOrientation::default(),
            children,
            invalid: false,
            disabled: false,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Apply the upstream `data-invalid` styling state to this field grouping.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Apply the upstream `data-disabled` styling state to this field grouping.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let field_state = field_state_prim::FieldState {
            invalid: self.invalid,
            disabled: self.disabled,
        };

        field_state_prim::with_field_state_provider(cx, field_state, |cx| {
            let (gap, wrapper, inner_layout, muted, desc_mt_neg, desc_line_height) = {
                let theme = Theme::global(&*cx.app).snapshot();
                // shadcn-web fixture: label->input and input->desc gaps settle at ~12px.
                let gap = MetricRef::space(Space::N3).resolve(&theme);
                let wrapper = decl_style::container_props(
                    &theme,
                    self.chrome,
                    LayoutRefinement::default().w_full().merge(self.layout),
                );
                let inner_layout = decl_style::layout_style(
                    &theme,
                    LayoutRefinement::default().w_full().min_w_0(),
                );
                let muted = muted_foreground_color(&theme);
                let desc_mt_neg =
                    decl_style::layout_style(&theme, LayoutRefinement::default().mt_neg(Space::N1));
                let desc_line_height = description_text_refinement_with_fallbacks(
                    &theme,
                    "component.field.description",
                    Some(theme_tokens::metric::COMPONENT_TEXT_SM_PX),
                    Some(theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT),
                )
                .line_height
                .unwrap_or_else(|| theme.metric_token("font.line_height"));
                (
                    gap,
                    wrapper,
                    inner_layout,
                    muted,
                    desc_mt_neg,
                    desc_line_height,
                )
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
                        // Container queries are frame-lagged. When the region width is
                        // temporarily unknown (e.g. in single-pass layout test harnesses), fall
                        // back to viewport behavior so we avoid branching on a missing
                        // measurement.
                        let default_when_unknown =
                            cx.environment_viewport_width(Invalidation::Layout).0
                                >= fret_ui_kit::declarative::container_queries::tailwind::MD.0;
                        let md_breakpoint = fret_ui_kit::declarative::container_width_at_least(
                            cx,
                            region_id,
                            Invalidation::Layout,
                            default_when_unknown,
                            fret_ui_kit::declarative::container_queries::tailwind::MD,
                            fret_ui_kit::declarative::ContainerQueryHysteresis::default(),
                        );

                        let inner = match orientation {
                            FieldOrientation::Vertical => cx.column(
                                ColumnProps {
                                    layout: inner_layout.clone(),
                                    gap: gap.into(),
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
                                                && is_field_description(
                                                    muted,
                                                    desc_line_height,
                                                    &child,
                                                )
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
                                    gap: gap.into(),
                                    justify: MainAlign::Start,
                                    align: align_horizontal,
                                    ..Default::default()
                                },
                                move |_cx| children,
                            ),
                            FieldOrientation::Responsive => {
                                if md_breakpoint {
                                    let mut children_row = children.into_iter().collect::<Vec<_>>();
                                    let first_has_flex_grow = children_row
                                        .first()
                                        .and_then(|e| kind_flex_grow(&e.kind))
                                        .is_some_and(|grow| grow > 0.0);
                                    if children_row.len() >= 2 && !first_has_flex_grow {
                                        let first = children_row.remove(0);
                                        children_row
                                            .insert(0, responsive_md_content_flex_1_min_w_0(first));
                                    }

                                    let children_row = children_row
                                        .into_iter()
                                        .enumerate()
                                        .map(|(idx, child)| {
                                            if idx == 0 {
                                                child
                                            } else {
                                                responsive_md_width_auto(child)
                                            }
                                        })
                                        .collect::<Vec<_>>();
                                    cx.row(
                                        RowProps {
                                            layout: inner_layout,
                                            gap: gap.into(),
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
                                            gap: gap.into(),
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
                                                        && is_field_description(
                                                            muted,
                                                            desc_line_height,
                                                            &child,
                                                        )
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
            .attach_semantics(SemanticsDecoration::default().role(SemanticsRole::Group))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{
        AppWindowId, MaterialService, Modifiers, MouseButton, PathCommand, PathConstraints, PathId,
        PathMetrics, PathService, PathStyle, Point, Px, Rect, Size, SvgId, SvgService, TextBlobId,
        TextConstraints, TextInput, TextMetrics, TextService,
    };
    use fret_runtime::Model;
    use fret_ui::tree::UiTree;
    use fret_ui_kit::primitives::control_registry::{ControlId, control_registry_model};

    use crate::shadcn_themes::{ShadcnBaseColor, ShadcnColorScheme, apply_shadcn_new_york};

    fn bounds() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(800.0), Px(240.0)),
        )
    }

    struct FakeServices;

    impl TextService for FakeServices {
        fn prepare(
            &mut self,
            _input: &TextInput,
            _constraints: TextConstraints,
        ) -> (TextBlobId, TextMetrics) {
            (
                TextBlobId::default(),
                TextMetrics {
                    size: Size::new(Px(10.0), Px(10.0)),
                    baseline: Px(8.0),
                },
            )
        }

        fn release(&mut self, _blob: TextBlobId) {}
    }

    impl PathService for FakeServices {
        fn prepare(
            &mut self,
            _commands: &[PathCommand],
            _style: PathStyle,
            _constraints: PathConstraints,
        ) -> (PathId, PathMetrics) {
            (PathId::default(), PathMetrics::default())
        }

        fn release(&mut self, _path: PathId) {}
    }

    impl SvgService for FakeServices {
        fn register_svg(&mut self, _bytes: &[u8]) -> SvgId {
            SvgId::default()
        }

        fn unregister_svg(&mut self, _svg: SvgId) -> bool {
            true
        }
    }

    impl MaterialService for FakeServices {
        fn register_material(
            &mut self,
            _desc: fret_core::MaterialDescriptor,
        ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
            Ok(fret_core::MaterialId::default())
        }

        fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
            true
        }
    }

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

    #[test]
    fn field_vertical_defaults_to_gap_3() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(120.0), Px(120.0)),
        );

        let theme = Theme::global(&app).snapshot();
        let expected = MetricRef::space(Space::N3).resolve(&theme);

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            Field::new([cx.text("Label"), cx.text("Control")])
                .orientation(FieldOrientation::Vertical)
                .into_element(cx)
        });

        fn find_first_column_gap(el: &AnyElement) -> Option<Px> {
            if let ElementKind::Column(props) = &el.kind {
                return match props.gap {
                    fret_ui::element::SpacingLength::Px(px) => Some(px),
                    _ => None,
                };
            }
            for child in &el.children {
                if let Some(found) = find_first_column_gap(child) {
                    return Some(found);
                }
            }
            None
        }

        let gap = find_first_column_gap(&element).expect("field should contain a Column");
        assert!(
            (gap.0 - expected.0).abs() <= 0.5,
            "expected Field gap ~ {}px, got {}px",
            expected.0,
            gap.0
        );
    }

    #[test]
    fn field_title_and_plain_label_approximate_upstream_w_fit_defaults() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );

        let title = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            FieldTitle::new("Price range").into_element(cx)
        });
        let title_layout = kind_layout(&title.kind).expect("expected FieldTitle layout");
        assert_eq!(title_layout.size.width, fret_ui::element::Length::Auto);
        assert_eq!(title_layout.flex.align_self, Some(CrossAlign::Start));

        let label = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            FieldLabel::new("Email").into_element(cx)
        });
        let label_layout = kind_layout(&label.kind).expect("expected FieldLabel layout");
        assert_eq!(label_layout.size.width, fret_ui::element::Length::Auto);
        assert_eq!(label_layout.flex.align_self, Some(CrossAlign::Start));
    }

    #[test]
    fn field_description_scopes_inherited_text_style() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds(), "test", |cx| {
                FieldDescription::new("We will never share it.").into_element(cx)
            });

        let fret_ui::element::ElementKind::Text(props) = &element.kind else {
            panic!("expected FieldDescription to be a text element");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());

        let theme = fret_ui::Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_text_style.as_ref(),
            Some(
                &fret_ui_kit::typography::description_text_refinement_with_fallbacks(
                    &theme,
                    "component.field.description",
                    Some(fret_ui_kit::theme_tokens::metric::COMPONENT_TEXT_SM_PX),
                    Some(fret_ui_kit::theme_tokens::metric::COMPONENT_TEXT_SM_LINE_HEIGHT),
                )
            )
        );
        assert_eq!(
            element.inherited_foreground,
            Some(fret_ui_kit::typography::muted_foreground_color(&theme))
        );
    }

    #[test]
    fn field_registers_label_and_helper_text_for_control_registry_association() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(180.0)),
        );

        let model = app.models_mut().insert(String::new());
        let id = ControlId::from("email");

        let _ = fret_ui::elements::with_element_cx(&mut app, window, bounds, "field-assoc", |cx| {
            cx.column(fret_ui::element::ColumnProps::default(), |cx| {
                vec![
                    FieldLabel::new("Email")
                        .for_control(id.clone())
                        .into_element(cx),
                    FieldDescription::new("We will never share it.")
                        .for_control(id.clone())
                        .into_element(cx),
                    FieldError::new("Required.")
                        .for_control(id.clone())
                        .into_element(cx),
                    crate::Input::new(model.clone())
                        .control_id(id.clone())
                        .into_element(cx),
                ]
            })
        });
        let root =
            fret_ui::elements::with_element_cx(&mut app, window, bounds, "field-assoc", |cx| {
                cx.column(fret_ui::element::ColumnProps::default(), |cx| {
                    vec![
                        FieldLabel::new("Email")
                            .for_control(id.clone())
                            .into_element(cx),
                        FieldDescription::new("We will never share it.")
                            .for_control(id.clone())
                            .into_element(cx),
                        FieldError::new("Required.")
                            .for_control(id.clone())
                            .into_element(cx),
                        crate::Input::new(model.clone())
                            .control_id(id.clone())
                            .into_element(cx),
                    ]
                })
            });

        fn find_text_input(el: &AnyElement) -> Option<&AnyElement> {
            if matches!(el.kind, ElementKind::TextInput(_)) {
                return Some(el);
            }
            for child in &el.children {
                if let Some(found) = find_text_input(child) {
                    return Some(found);
                }
            }
            None
        }

        let label = &root.children[0];
        let desc = &root.children[1];
        let err = &root.children[2];
        let text_input = find_text_input(&root).expect("expected a TextInput node");

        let decoration = text_input
            .semantics_decoration
            .as_ref()
            .expect("expected semantics decoration on TextInput");
        assert_eq!(decoration.labelled_by_element, Some(label.id.0));
        // Error takes precedence over description for `described-by`.
        assert_eq!(decoration.described_by_element, Some(err.id.0));

        let ElementKind::Semantics(semantics) = &label.kind else {
            panic!("expected FieldLabel(for_control) to be a Semantics wrapper");
        };
        assert_eq!(semantics.role, SemanticsRole::Text);
        assert!(
            semantics.controls_element.is_some(),
            "expected FieldLabel to set `controls_element` when a control is registered"
        );

        // Sanity: the description/error nodes we register are stable elements in the tree.
        assert!(desc.id.0 != 0);
        assert!(err.id.0 != 0);
    }

    #[test]
    fn field_stamps_group_role_for_accessibility() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            Field::new([cx.text("Label"), cx.text("Control")]).into_element(cx)
        });

        assert_eq!(
            element.semantics_decoration.as_ref().and_then(|d| d.role),
            Some(SemanticsRole::Group)
        );
    }

    #[test]
    fn field_error_stamps_alert_role_and_live_region() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            FieldError::new("Required.").into_element(cx)
        });

        let decoration = element
            .semantics_decoration
            .as_ref()
            .expect("expected semantics decoration on FieldError");
        assert_eq!(decoration.role, Some(SemanticsRole::Alert));
        assert_eq!(decoration.live, Some(Some(SemanticsLive::Assertive)));
        assert_eq!(decoration.live_atomic, Some(true));
    }

    #[test]
    fn field_label_click_to_focus_skips_nested_pressables() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = bounds();
        let model: Model<String> = app.models_mut().insert(String::new());
        let control_id = ControlId::from("email");

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-field-label-nested-pressable-focus-suppression",
            |cx| {
                let label_contents = ui::h_flex(|cx| {
                    vec![
                        ui::text("Email").into_element(cx),
                        crate::Button::new("Help")
                            .variant(crate::ButtonVariant::Secondary)
                            .size(crate::ButtonSize::Sm)
                            .test_id("field.label.nested_button")
                            .into_element(cx),
                    ]
                })
                .justify_between()
                .items_center()
                .layout(LayoutRefinement::default().w_full().min_w_0())
                .into_element(cx);

                vec![cx.column(fret_ui::element::ColumnProps::default(), |cx| {
                    vec![
                        FieldLabel::new("Email")
                            .for_control(control_id.clone())
                            .wrap([label_contents])
                            .test_id("field.label")
                            .into_element(cx),
                        crate::Input::new(model.clone())
                            .control_id(control_id.clone())
                            .into_element(cx),
                    ]
                })]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let column_node = ui.children(root)[0];
        let label_node = ui.children(column_node)[0];
        let snap = ui.semantics_snapshot_arc().expect("semantics snapshot");
        let nested_button_node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("field.label.nested_button"))
            .map(|n| n.id)
            .expect("expected nested button semantics node");

        let center_of = |bounds: fret_core::Rect| {
            Point::new(
                Px(bounds.origin.x.0 + bounds.size.width.0 * 0.5),
                Px(bounds.origin.y.0 + bounds.size.height.0 * 0.5),
            )
        };

        let nested_button_bounds = ui
            .debug_node_bounds(nested_button_node)
            .expect("nested button bounds");
        let label_bounds = ui.debug_node_bounds(label_node).expect("label bounds");
        let registry_model = fret_ui::elements::with_element_cx(
            &mut app,
            window,
            bounds,
            "shadcn-field-label-nested-pressable-focus-suppression-registry",
            |cx| control_registry_model(cx),
        );
        let control_focus_target = app
            .models()
            .read(&registry_model, |reg| {
                reg.control_for(window, &control_id)
                    .map(|entry| entry.element)
            })
            .ok()
            .flatten()
            .and_then(|element| fret_ui::elements::node_for_element(&mut app, window, element))
            .expect("expected registered control focus target");

        assert_eq!(ui.focus(), None);

        // Clicking the nested pressable should not cause the label to forward focus to the
        // associated control.
        let nested_button_position = center_of(nested_button_bounds);
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(0),
                position: nested_button_position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(0),
                position: nested_button_position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );
        assert_ne!(ui.focus(), Some(control_focus_target));

        // Clicking the label outside the nested pressable should still forward focus.
        let y_mid = Px(label_bounds.origin.y.0 + label_bounds.size.height.0 * 0.5);
        let x0 = label_bounds.origin.x.0;
        let w = label_bounds.size.width.0;
        let candidates = [
            Point::new(Px(x0 + 4.0), y_mid),
            Point::new(Px(x0 + w * 0.25), y_mid),
            Point::new(Px(x0 + w * 0.75), y_mid),
            Point::new(Px(x0 + w - 4.0), y_mid),
        ];
        let label_position = candidates
            .into_iter()
            .find(|p| label_bounds.contains(*p) && !nested_button_bounds.contains(*p))
            .expect("expected a point in label bounds outside the nested pressable bounds");

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(1),
                position: label_position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(1),
                position: label_position,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        assert_eq!(ui.focus(), Some(control_focus_target));
    }

    #[test]
    fn field_label_click_invokes_registered_control_action() {
        let window = AppWindowId::default();
        let mut app = App::new();
        apply_shadcn_new_york(&mut app, ShadcnBaseColor::Neutral, ShadcnColorScheme::Light);

        let mut ui: UiTree<App> = UiTree::new();
        ui.set_window(window);

        let mut services = FakeServices;
        let bounds = bounds();
        let model: Model<bool> = app.models_mut().insert(false);
        let control_id = ControlId::from("field.toggle.action");

        let root = fret_ui::declarative::render_root(
            &mut ui,
            &mut app,
            &mut services,
            window,
            bounds,
            "shadcn-field-label-click-invokes-control-action",
            |cx| {
                vec![cx.column(fret_ui::element::ColumnProps::default(), |cx| {
                    vec![
                        FieldLabel::new("Enabled")
                            .for_control(control_id.clone())
                            .test_id("field.label.action")
                            .into_element(cx),
                        crate::Toggle::new(model.clone())
                            .label("Enabled")
                            .control_id(control_id.clone())
                            .into_element(cx),
                    ]
                })]
            },
        );
        ui.set_root(root);
        ui.request_semantics_snapshot();
        ui.layout_all(&mut app, &mut services, bounds, 1.0);

        let snap = ui.semantics_snapshot_arc().expect("semantics snapshot");
        let label_node = snap
            .nodes
            .iter()
            .find(|n| n.test_id.as_deref() == Some("field.label.action"))
            .map(|n| n.id)
            .expect("expected label semantics node");
        let label_bounds = ui
            .debug_node_bounds(label_node)
            .expect("expected label bounds");
        let label_center = Point::new(
            Px(label_bounds.origin.x.0 + label_bounds.size.width.0 * 0.5),
            Px(label_bounds.origin.y.0 + label_bounds.size.height.0 * 0.5),
        );

        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Down {
                pointer_id: fret_core::PointerId(2),
                position: label_center,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                pointer_type: fret_core::PointerType::Mouse,
                click_count: 1,
            }),
        );
        ui.dispatch_event(
            &mut app,
            &mut services,
            &fret_core::Event::Pointer(fret_core::PointerEvent::Up {
                pointer_id: fret_core::PointerId(2),
                position: label_center,
                button: MouseButton::Left,
                modifiers: Modifiers::default(),
                is_click: true,
                click_count: 1,
                pointer_type: fret_core::PointerType::Mouse,
            }),
        );

        assert!(
            app.models().get_copied(&model).unwrap_or(false),
            "expected FieldLabel click to invoke the registered control action"
        );
    }
}
