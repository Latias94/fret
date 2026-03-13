use std::marker::PhantomData;
use std::sync::Arc;

use fret_core::{Edges, Px, TextAlign, TextOverflow};
use fret_ui::element::{AnyElement, CrossAlign, FlexProps, LayoutQueryRegionProps, MainAlign};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::typography::scope_description_text;
use fret_ui_kit::{
    ChromeRefinement, ColorRef, IntoUiElement, LayoutRefinement, MetricRef, Radius, Space, UiPatch,
    UiPatchTarget, UiSupportsChrome, UiSupportsLayout, ui,
};

#[derive(Debug)]
pub struct Empty {
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl Empty {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            children: children.into_iter().collect(),
        }
    }

    pub fn build<H: UiHost, B>(build: B) -> EmptyBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        EmptyBuild {
            build: Some(build),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            _phantom: PhantomData,
        }
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();

        let border = theme.color_token("border");
        let fg = theme.color_token("foreground");

        let chrome_override = self.chrome;
        let layout_override = self.layout;
        let children = self.children;

        let region_props = LayoutQueryRegionProps {
            layout: decl_style::layout_style(
                &theme,
                LayoutRefinement::default().w_full().min_w_0(),
            ),
            name: None,
        };

        // Upstream `new-york-v4` mirrors `p-6 md:p-12`. In editor-grade layouts the effective
        // breakpoint should follow the Empty container width (panel resize), not the viewport.
        fret_ui_kit::declarative::container_query_region_with_id(
            cx,
            "shadcn.empty",
            region_props,
            move |cx, region_id| {
                let md = fret_ui_kit::declarative::container_width_at_least(
                    cx,
                    region_id,
                    Invalidation::Layout,
                    false,
                    fret_ui_kit::declarative::container_queries::tailwind::MD,
                    fret_ui_kit::declarative::ContainerQueryHysteresis::default(),
                );

                let padding = if md { Space::N12 } else { Space::N6 };

                let chrome = ChromeRefinement::default()
                    .p(padding)
                    .rounded(Radius::Lg)
                    .border_1()
                    .border_dash(fret_core::scene::DashPatternV1::new(
                        Px(4.0),
                        Px(4.0),
                        Px(0.0),
                    ))
                    .border_color(ColorRef::Color(border))
                    .text_color(ColorRef::Color(fg))
                    .merge(chrome_override);

                let layout = LayoutRefinement::default()
                    .min_w_0()
                    .w_full()
                    .merge(layout_override);

                let props = decl_style::container_props(&theme, chrome, layout);

                vec![cx.container(props, move |cx| {
                    let layout = decl_style::layout_style(
                        &theme,
                        LayoutRefinement::default().w_full().min_w_0(),
                    );
                    let gap = MetricRef::space(Space::N6).resolve(&theme);
                    vec![cx.flex(
                        FlexProps {
                            layout,
                            direction: fret_core::Axis::Vertical,
                            gap: gap.into(),
                            padding: Edges::all(Px(0.0)).into(),
                            justify: MainAlign::Center,
                            align: CrossAlign::Center,
                            wrap: false,
                        },
                        move |_cx| children,
                    )]
                })]
            },
        )
    }
}

#[derive(Debug)]
pub struct EmptyHeader {
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl EmptyHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            layout: LayoutRefinement::default(),
            children: children.into_iter().collect(),
        }
    }

    pub fn build<H: UiHost, B>(build: B) -> EmptyHeaderBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        EmptyHeaderBuild {
            build: Some(build),
            layout: LayoutRefinement::default(),
            _phantom: PhantomData,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let gap = MetricRef::space(Space::N2).resolve(&theme);
        let max_w = MetricRef::Px(Px(384.0));
        let layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .max_w(max_w)
                .min_w_0()
                .merge(self.layout),
        );
        let children = self.children;
        cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Vertical,
                gap: gap.into(),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| children,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmptyMediaVariant {
    #[default]
    Default,
    Icon,
}

#[derive(Debug)]
pub struct EmptyMedia {
    variant: EmptyMediaVariant,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl EmptyMedia {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            variant: EmptyMediaVariant::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            children: children.into_iter().collect(),
        }
    }

    pub fn build<H: UiHost, B>(build: B) -> EmptyMediaBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        EmptyMediaBuild {
            build: Some(build),
            variant: EmptyMediaVariant::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            _phantom: PhantomData,
        }
    }

    pub fn variant(mut self, variant: EmptyMediaVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();

        let mut layout = LayoutRefinement::default()
            .mb(Space::N2)
            .flex_shrink_0()
            .merge(self.layout);
        let mut chrome = ChromeRefinement::default().merge(self.chrome);

        if self.variant == EmptyMediaVariant::Icon {
            let bg = theme
                .color_by_key("muted")
                .unwrap_or_else(|| theme.color_token("muted.background"));
            let fg = theme.color_token("foreground");
            layout = layout
                .w_px(MetricRef::space(Space::N10))
                .h_px(MetricRef::space(Space::N10));
            chrome = ChromeRefinement::default()
                .rounded(Radius::Lg)
                .bg(ColorRef::Color(bg))
                .text_color(ColorRef::Color(fg))
                .merge(chrome);
        }

        let props = decl_style::container_props(&theme, chrome, layout);
        let children = self.children;
        cx.container(props, move |cx| {
            let layout = decl_style::layout_style(&theme, LayoutRefinement::default().size_full());
            vec![cx.flex(
                FlexProps {
                    layout,
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0).into(),
                    padding: Edges::all(Px(0.0)).into(),
                    justify: MainAlign::Center,
                    align: CrossAlign::Center,
                    wrap: false,
                },
                move |_cx| children,
            )]
        })
    }
}

#[derive(Debug, Clone)]
pub struct EmptyTitle {
    text: Arc<str>,
}

impl EmptyTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let fg = theme.color_token("foreground");
        let px = theme
            .metric_by_key("component.empty.title_px")
            .unwrap_or(Px(18.0));
        let line_height = theme
            .metric_by_key("component.empty.title_line_height")
            .unwrap_or(Px(28.0));

        ui::text(self.text)
            .text_size_px(px)
            .line_height_px(line_height)
            .font_medium()
            .text_align(TextAlign::Center)
            .text_balance()
            .w_full()
            .min_w_0()
            .text_color(ColorRef::Color(fg))
            .into_element(cx)
    }
}

#[derive(Debug, Clone)]
pub struct EmptyDescription {
    text: Arc<str>,
}

impl EmptyDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();

        scope_description_text(
            ui::raw_text(self.text)
                .text_balance()
                .overflow(TextOverflow::Clip)
                .text_align(TextAlign::Center)
                .w_full()
                .max_w(Px(384.0))
                .min_w_0()
                .into_element(cx),
            &theme,
            "component.empty.description",
        )
    }
}

#[derive(Debug)]
pub struct EmptyContent {
    layout: LayoutRefinement,
    children: Vec<AnyElement>,
}

impl EmptyContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            layout: LayoutRefinement::default(),
            children: children.into_iter().collect(),
        }
    }

    pub fn build<H: UiHost, B>(build: B) -> EmptyContentBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        EmptyContentBuild {
            build: Some(build),
            layout: LayoutRefinement::default(),
            _phantom: PhantomData,
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let gap = MetricRef::space(Space::N4).resolve(&theme);
        let max_w = MetricRef::Px(Px(384.0));
        let layout = decl_style::layout_style(
            &theme,
            LayoutRefinement::default()
                .w_full()
                .max_w(max_w)
                .min_w_0()
                .merge(self.layout),
        );
        let children = self.children;
        cx.flex(
            FlexProps {
                layout,
                direction: fret_core::Axis::Vertical,
                gap: gap.into(),
                padding: Edges::all(Px(0.0)).into(),
                justify: MainAlign::Start,
                align: CrossAlign::Center,
                wrap: false,
            },
            move |_cx| children,
        )
    }
}

pub fn empty<H: UiHost, I, F, T>(
    f: F,
) -> EmptyBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    Empty::build(move |cx, out| {
        let children = f(cx);
        extend_landed_empty_children(cx, out, children);
    })
}

pub fn empty_header<H: UiHost, I, F, T>(
    f: F,
) -> EmptyHeaderBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    EmptyHeader::build(move |cx, out| {
        let children = f(cx);
        extend_landed_empty_children(cx, out, children);
    })
}

pub fn empty_media<H: UiHost, I, F, T>(
    f: F,
) -> EmptyMediaBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    EmptyMedia::build(move |cx, out| {
        let children = f(cx);
        extend_landed_empty_children(cx, out, children);
    })
}

pub fn empty_title<T>(text: T) -> EmptyTitle
where
    T: Into<Arc<str>>,
{
    EmptyTitle::new(text)
}

pub fn empty_description<T>(text: T) -> EmptyDescription
where
    T: Into<Arc<str>>,
{
    EmptyDescription::new(text)
}

pub fn empty_content<H: UiHost, I, F, T>(
    f: F,
) -> EmptyContentBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    EmptyContent::build(move |cx, out| {
        let children = f(cx);
        extend_landed_empty_children(cx, out, children);
    })
}

pub struct EmptyBuild<H, B> {
    build: Option<B>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> EmptyBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children =
            collect_built_empty_children(cx, self.build.expect("expected empty build closure"));
        Empty::new(children)
            .refine_style(self.chrome)
            .refine_layout(self.layout)
            .into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for EmptyBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_style(patch.chrome).refine_layout(patch.layout)
    }
}

impl<H: UiHost, B> UiSupportsChrome for EmptyBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> UiSupportsLayout for EmptyBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> IntoUiElement<H> for EmptyBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        EmptyBuild::into_element(self, cx)
    }
}

pub struct EmptyHeaderBuild<H, B> {
    build: Option<B>,
    layout: LayoutRefinement,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> EmptyHeaderBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = collect_built_empty_children(
            cx,
            self.build.expect("expected empty-header build closure"),
        );
        EmptyHeader::new(children)
            .refine_layout(self.layout)
            .into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for EmptyHeaderBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_layout(patch.layout)
    }
}

impl<H: UiHost, B> UiSupportsLayout for EmptyHeaderBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> IntoUiElement<H> for EmptyHeaderBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        EmptyHeaderBuild::into_element(self, cx)
    }
}

pub struct EmptyMediaBuild<H, B> {
    build: Option<B>,
    variant: EmptyMediaVariant,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> EmptyMediaBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    pub fn variant(mut self, variant: EmptyMediaVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = collect_built_empty_children(
            cx,
            self.build.expect("expected empty-media build closure"),
        );
        EmptyMedia::new(children)
            .variant(self.variant)
            .refine_style(self.chrome)
            .refine_layout(self.layout)
            .into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for EmptyMediaBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_style(patch.chrome).refine_layout(patch.layout)
    }
}

impl<H: UiHost, B> UiSupportsChrome for EmptyMediaBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> UiSupportsLayout for EmptyMediaBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> IntoUiElement<H> for EmptyMediaBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        EmptyMediaBuild::into_element(self, cx)
    }
}

pub struct EmptyContentBuild<H, B> {
    build: Option<B>,
    layout: LayoutRefinement,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> EmptyContentBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = collect_built_empty_children(
            cx,
            self.build.expect("expected empty-content build closure"),
        );
        EmptyContent::new(children)
            .refine_layout(self.layout)
            .into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for EmptyContentBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_layout(patch.layout)
    }
}

impl<H: UiHost, B> UiSupportsLayout for EmptyContentBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> IntoUiElement<H> for EmptyContentBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        EmptyContentBuild::into_element(self, cx)
    }
}

fn collect_built_empty_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build: impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
) -> Vec<AnyElement> {
    let mut out = Vec::new();
    build(cx, &mut out);
    out
}

fn extend_landed_empty_children<H: UiHost, I, T>(
    cx: &mut ElementContext<'_, H>,
    out: &mut Vec<AnyElement>,
    children: I,
) where
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    for child in children {
        out.push(child.into_element(cx));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::ElementKind;

    #[test]
    fn empty_description_scopes_inherited_text_style() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            EmptyDescription::new("Description").into_element(cx)
        });

        let ElementKind::Text(props) = &element.kind else {
            panic!("expected EmptyDescription to be a text element");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());

        let theme = fret_ui::Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_text_style.as_ref(),
            Some(&fret_ui_kit::typography::description_text_refinement(
                &theme,
                "component.empty.description",
            ))
        );
        assert_eq!(
            element.inherited_foreground,
            Some(fret_ui_kit::typography::muted_foreground_color(&theme))
        );
    }
}
