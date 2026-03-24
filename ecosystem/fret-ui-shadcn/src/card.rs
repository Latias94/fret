use std::marker::PhantomData;
use std::sync::Arc;

use fret_core::{FontWeight, Px, TextOverflow, TextWrap};
use fret_ui::element::{AnyElement, ElementKind, Length};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::{current_color, style as decl_style};
use fret_ui_kit::{
    ChromeRefinement, ColorRef, IntoUiElement, Justify, LayoutRefinement, Space, UiPatch,
    UiPatchTarget, UiSupportsChrome, UiSupportsLayout, ui,
};

use crate::layout as shadcn_layout;
use crate::surface_slot::{ShadcnSurfaceSlot, with_surface_slot_provider};
use crate::test_id::attach_test_id;
use fret_ui_kit::typography::scope_description_text;

const CARD_ACTION_MARKER_PREFIX: &str = "fret-ui-shadcn.card-action";
const CARD_FOOTER_MARKER_PREFIX: &str = "fret-ui-shadcn.card-footer";

fn matches_marker(test_id: &str, prefix: &str) -> bool {
    test_id == prefix
        || (test_id.starts_with(prefix)
            && test_id
                .as_bytes()
                .get(prefix.len())
                .is_some_and(|b| *b == b':'))
}

fn is_card_action_marker(element: &AnyElement) -> bool {
    element
        .semantics_decoration
        .as_ref()
        .and_then(|d| d.test_id.as_deref())
        .is_some_and(|id| matches_marker(id, CARD_ACTION_MARKER_PREFIX))
        || match &element.kind {
            ElementKind::Semantics(props) => props
                .test_id
                .as_deref()
                .is_some_and(|id| matches_marker(id, CARD_ACTION_MARKER_PREFIX)),
            _ => false,
        }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CardSize {
    #[default]
    Default,
    Sm,
}

fn card_size_in_scope<H: UiHost>(cx: &ElementContext<'_, H>) -> CardSize {
    cx.provided::<CardSize>().copied().unwrap_or_default()
}

#[track_caller]
fn with_card_size_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    size: CardSize,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    cx.provide(size, f)
}

fn card_chrome(theme: &Theme, _size: CardSize) -> ChromeRefinement {
    let bg = theme.color_token("card");
    let border = theme.color_token("border");

    // shadcn/ui v4: Card uses `rounded-xl`, which is computed from the base `--radius`.
    //
    // In the shadcn token model:
    // - `rounded-lg` ~= `--radius`
    // - `rounded-md` ~= `--radius - 2px`
    // - `rounded-xl` ~= `--radius + 4px`
    //
    // We model the base radius as `metric.radius.lg`, and derive `rounded-xl` from it to keep
    // behavior stable when the theme radius changes.
    let base_radius = theme.metric_token("metric.radius.lg");
    let rounded_xl = Px(base_radius.0 + 4.0);

    // shadcn/ui v4 card base:
    // - `rounded-xl border bg-card text-card-foreground shadow-sm`
    // - `flex flex-col gap-6 py-6` (horizontal padding comes from sections)
    ChromeRefinement::default()
        .radius(rounded_xl)
        .border_1()
        .shadow_sm()
        .py(match _size {
            CardSize::Default => Space::N6,
            CardSize::Sm => Space::N4,
        })
        .bg(ColorRef::Color(bg))
        .border_color(ColorRef::Color(border))
}

#[derive(Debug)]
pub struct Card {
    children: Vec<AnyElement>,
    size: CardSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Card {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            size: CardSize::Default,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Builder-first variant that collects children inside the card size provider.
    pub fn build<H: UiHost, B>(build: B) -> CardBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        CardBuild {
            build: Some(build),
            size: CardSize::Default,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            _phantom: PhantomData,
        }
    }

    pub fn size(mut self, size: CardSize) -> Self {
        self.size = size;
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
        let size = self.size;
        with_card_size_provider(cx, size, |cx| {
            let children = self.children;
            let gap = match size {
                CardSize::Default => Space::N6,
                CardSize::Sm => Space::N4,
            };

            let fg = {
                let theme = Theme::global(&*cx.app);
                theme.color_token("card-foreground")
            };

            let props = {
                let theme = Theme::global(&*cx.app);
                let chrome = card_chrome(theme, size).merge(self.chrome);
                // Keep the root width caller-owned so the recipe matches upstream shadcn/ui
                // semantics more closely: examples opt into widths like `w-full max-w-sm` at the
                // call site, while the card sections themselves still fill the card's resolved
                // inner width.
                let layout = LayoutRefinement::default().merge(self.layout);
                decl_style::container_props(theme, chrome, layout)
            };

            // Cards behave like block containers in shadcn/ui examples: their sections are expected to
            // stretch to the card width unless explicitly constrained.
            shadcn_layout::container_vstack_fill_width(
                cx,
                props,
                shadcn_layout::VStackProps::default()
                    .gap(gap)
                    .layout(LayoutRefinement::default().w_full()),
                children,
            )
            .inherit_foreground(fg)
        })
    }
}

pub fn card<H: UiHost, I, F, T>(
    f: F,
) -> CardBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    Card::build(move |cx, out| {
        let children = f(cx);
        extend_landed_children(cx, out, children);
    })
}

/// Build a card and its sections inside a size provider.
///
/// This avoids footguns where callers construct `CardHeader` / `CardContent` / `CardFooter`
/// elements outside the `Card` subtree and accidentally miss inherited size defaults.
pub fn card_sized<H: UiHost, I, F, T>(
    size: CardSize,
    f: F,
) -> CardBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    Card::build(move |cx, out| {
        let children = f(cx);
        extend_landed_children(cx, out, children);
    })
    .size(size)
}

pub struct CardBuild<H, B> {
    build: Option<B>,
    size: CardSize,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> CardBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    pub fn size(mut self, size: CardSize) -> Self {
        self.size = size;
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
        let size = self.size;
        let children = with_card_size_provider(cx, size, |cx| {
            collect_built_card_children(cx, self.build.expect("expected card build closure"))
        });
        Card::new(children)
            .size(size)
            .refine_style(self.chrome)
            .refine_layout(self.layout)
            .into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for CardBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_style(patch.chrome).refine_layout(patch.layout)
    }
}

impl<H: UiHost, B> UiSupportsChrome for CardBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> UiSupportsLayout for CardBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> IntoUiElement<H> for CardBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        CardBuild::into_element(self, cx)
    }
}

fn extend_landed_children<H: UiHost, I, T>(
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

pub fn card_header<H: UiHost, I, F, T>(
    f: F,
) -> CardHeaderBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    CardHeader::build(move |cx, out| {
        let children = f(cx);
        extend_landed_children(cx, out, children);
    })
}

pub fn card_action<H: UiHost, I, F, T>(
    f: F,
) -> CardActionBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    CardAction::build(move |cx, out| {
        let children = f(cx);
        extend_landed_children(cx, out, children);
    })
}

pub fn card_title<T>(text: T) -> CardTitle
where
    T: Into<Arc<str>>,
{
    CardTitle::new(text)
}

pub fn card_title_children<H: UiHost, I, F, T>(
    f: F,
) -> CardTitleBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    CardTitle::build(move |cx, out| {
        let children = f(cx);
        extend_landed_children(cx, out, children);
    })
}

pub fn card_description<T>(text: T) -> CardDescription
where
    T: Into<Arc<str>>,
{
    CardDescription::new(text)
}

pub fn card_description_children<H: UiHost, I, F, T>(
    f: F,
) -> CardDescriptionBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    CardDescription::build(move |cx, out| {
        let children = f(cx);
        extend_landed_children(cx, out, children);
    })
}

pub fn card_content<H: UiHost, I, F, T>(
    f: F,
) -> CardContentBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    CardContent::build(move |cx, out| {
        let children = f(cx);
        extend_landed_children(cx, out, children);
    })
}

pub fn card_footer<H: UiHost, I, F, T>(
    f: F,
) -> CardFooterBuild<H, impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)>
where
    F: FnOnce(&mut ElementContext<'_, H>) -> I,
    I: IntoIterator<Item = T>,
    T: IntoUiElement<H>,
{
    CardFooter::build(move |cx, out| {
        let children = f(cx);
        extend_landed_children(cx, out, children);
    })
}

fn collect_built_card_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    build: impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
) -> Vec<AnyElement> {
    let mut out = Vec::new();
    build(cx, &mut out);
    out
}

#[derive(Debug)]
pub struct CardHeader {
    children: Vec<AnyElement>,
    size: Option<CardSize>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    border_bottom: bool,
}

impl CardHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            size: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            border_bottom: false,
        }
    }

    /// Builder-first variant that collects children at `into_element(cx)` time.
    pub fn build<H: UiHost, B>(build: B) -> CardHeaderBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        CardHeaderBuild {
            build: Some(build),
            size: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            border_bottom: false,
            _phantom: PhantomData,
        }
    }

    /// Explicitly set the card size for this section.
    ///
    /// Most compositions rely on `Card` installing a size provider; however, some callers build
    /// `CardHeader` / `CardContent` / `CardFooter` elements before they are inserted into a
    /// `Card` subtree. In those cases, inherited size is unavailable, so callers can pass an
    /// explicit size to match upstream shadcn behavior.
    pub fn size(mut self, size: CardSize) -> Self {
        self.size = Some(size);
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

    pub fn border_bottom(mut self, value: bool) -> Self {
        self.border_bottom = value;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let size = self.size.unwrap_or_else(|| card_size_in_scope(cx));
        let p = match size {
            CardSize::Default => Space::N6,
            CardSize::Sm => Space::N4,
        };
        let pb = match size {
            CardSize::Default => Space::N6,
            CardSize::Sm => Space::N4,
        };
        let border_bottom = self.border_bottom;
        let layout = self.layout;
        let props = {
            let theme = Theme::global(&*cx.app);
            let base = if border_bottom {
                // shadcn/ui v4: when the header has a bottom border it also adds `pb-6`, and uses a
                // smaller `pb-4` on `size=sm`.
                ChromeRefinement::default().px(p).pb(pb)
            } else {
                // shadcn/ui v4: `px-6` (and `px-4` for smaller cards).
                ChromeRefinement::default().px(p)
            };
            decl_style::container_props(
                theme,
                base.merge(self.chrome),
                LayoutRefinement::default().w_full().merge(layout),
            )
        };

        let mut action: Option<AnyElement> = None;
        let mut left: Vec<AnyElement> = Vec::with_capacity(self.children.len());

        for child in self.children {
            let is_action = is_card_action_marker(&child);
            if is_action && action.is_none() {
                action = Some(child);
            } else {
                left.push(child);
            }
        }

        let content = if let Some(action) = action {
            let left_col = ui::v_stack(move |_cx| left)
                // shadcn/ui v4 CardHeader uses `gap-2` between title and description, even
                // when an action slot is present.
                .gap(Space::N2)
                .layout(LayoutRefinement::default().flex_1().min_w_0())
                .into_element(cx);

            shadcn_layout::container_hstack(
                cx,
                props,
                shadcn_layout::HStackProps::default()
                    .gap(Space::N2)
                    .layout(LayoutRefinement::default().w_full())
                    .justify_between()
                    .items_start(),
                vec![left_col, action],
            )
        } else {
            shadcn_layout::container_vstack_fill_width(
                cx,
                props,
                shadcn_layout::VStackProps::default()
                    .gap(Space::N2)
                    .items_start(),
                left,
            )
        };

        if border_bottom {
            let outer_props = {
                let theme = Theme::global(&*cx.app);
                decl_style::container_props(
                    theme,
                    ChromeRefinement::default(),
                    LayoutRefinement::default().w_full(),
                )
            };
            let separator = crate::separator::Separator::new().into_element(cx);
            shadcn_layout::container_vstack(
                cx,
                outer_props,
                shadcn_layout::VStackProps::default()
                    .gap(Space::N0)
                    .layout(LayoutRefinement::default().w_full()),
                vec![content, separator],
            )
        } else {
            content
        }
    }
}

pub struct CardHeaderBuild<H, B> {
    build: Option<B>,
    size: Option<CardSize>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    border_bottom: bool,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> CardHeaderBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    pub fn size(mut self, size: CardSize) -> Self {
        self.size = Some(size);
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

    pub fn border_bottom(mut self, value: bool) -> Self {
        self.border_bottom = value;
        self
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = collect_built_card_children(
            cx,
            self.build.expect("expected card header build closure"),
        );
        let mut header = CardHeader::new(children)
            .refine_style(self.chrome)
            .refine_layout(self.layout)
            .border_bottom(self.border_bottom);
        if let Some(size) = self.size {
            header = header.size(size);
        }
        header.into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for CardHeaderBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_style(patch.chrome).refine_layout(patch.layout)
    }
}

impl<H: UiHost, B> UiSupportsChrome for CardHeaderBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> UiSupportsLayout for CardHeaderBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> IntoUiElement<H> for CardHeaderBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        CardHeaderBuild::into_element(self, cx)
    }
}

#[derive(Debug)]
pub struct CardAction {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl CardAction {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Builder-first variant that collects children at `into_element(cx)` time.
    pub fn build<H: UiHost, B>(build: B) -> CardActionBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        CardActionBuild {
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
        let props = {
            let theme = Theme::global(&*cx.app);
            decl_style::container_props(
                theme,
                ChromeRefinement::default().merge(self.chrome),
                LayoutRefinement::default().merge(self.layout),
            )
        };

        let children = self.children;
        let el = cx.container(props, move |cx| {
            if children.len() <= 1 {
                children
            } else {
                vec![
                    ui::h_row(move |_cx| children)
                        .gap(Space::N2)
                        .items_center()
                        .into_element(cx),
                ]
            }
        });

        let marker: Arc<str> = Arc::from(format!("{}:{}", CARD_ACTION_MARKER_PREFIX, el.id.0));
        attach_test_id(el, marker)
    }
}

pub struct CardActionBuild<H, B> {
    build: Option<B>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> CardActionBuild<H, B>
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
        let children = collect_built_card_children(
            cx,
            self.build.expect("expected card action build closure"),
        );
        CardAction::new(children)
            .refine_style(self.chrome)
            .refine_layout(self.layout)
            .into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for CardActionBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_style(patch.chrome).refine_layout(patch.layout)
    }
}

impl<H: UiHost, B> UiSupportsChrome for CardActionBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> UiSupportsLayout for CardActionBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> IntoUiElement<H> for CardActionBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        CardActionBuild::into_element(self, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, AttributedText, Axis, Point, Rect, Size, TextSpan};
    use fret_ui::element::{
        ContainerProps, CrossAlign, FlexProps, Length, MainAlign, Overflow, SemanticsProps,
    };
    use fret_ui::elements::GlobalElementId;
    use fret_ui_kit::ui::UiElementSinkExt as _;
    use fret_ui_kit::{MetricRef, UiExt as _};

    #[test]
    fn card_action_marker_matches_semantics_decoration_test_id() {
        let el = AnyElement::new(
            GlobalElementId(1),
            ElementKind::Container(ContainerProps::default()),
            Vec::new(),
        )
        .test_id(format!("{CARD_ACTION_MARKER_PREFIX}:1"));

        assert!(is_card_action_marker(&el));
    }

    #[test]
    fn card_action_marker_matches_legacy_semantics_test_id() {
        let el = AnyElement::new(
            GlobalElementId(1),
            ElementKind::Semantics(SemanticsProps {
                test_id: Some(Arc::<str>::from(format!("{CARD_ACTION_MARKER_PREFIX}:1"))),
                ..Default::default()
            }),
            Vec::new(),
        );

        assert!(is_card_action_marker(&el));
    }

    #[test]
    fn card_action_marker_ignores_other_test_ids() {
        let el = AnyElement::new(
            GlobalElementId(1),
            ElementKind::Container(ContainerProps::default()),
            Vec::new(),
        )
        .test_id("not-a-card-action");

        assert!(!is_card_action_marker(&el));
    }

    fn find_text<'a>(
        element: &'a AnyElement,
        needle: &str,
    ) -> Option<&'a fret_ui::element::TextProps> {
        match &element.kind {
            ElementKind::Text(props) if props.text.as_ref() == needle => Some(props),
            _ => element
                .children
                .iter()
                .find_map(|child| find_text(child, needle)),
        }
    }

    fn find_first_styled_text(element: &AnyElement) -> Option<&fret_ui::element::StyledTextProps> {
        match &element.kind {
            ElementKind::StyledText(props) => Some(props),
            _ => element.children.iter().find_map(find_first_styled_text),
        }
    }

    #[test]
    fn card_free_helpers_render_expected_structure() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            card(|cx| {
                ui::children![
                    cx;
                    card_header(|cx| {
                        ui::children![
                            cx;
                            card_title("Card Title"),
                            card_description("Card Description"),
                            card_action(|cx| ui::children![cx; cx.text("Card Action")]),
                        ]
                    }),
                    card_content(|cx| ui::children![cx; cx.text("Card Content")]),
                    card_footer(|cx| ui::children![cx; cx.text("Card Footer")]),
                ]
            })
            .into_element(cx)
        });

        assert!(find_text(&element, "Card Title").is_some());
        assert!(find_text(&element, "Card Description").is_some());
        assert!(find_text(&element, "Card Action").is_some());
        assert!(find_text(&element, "Card Content").is_some());
        assert!(find_text(&element, "Card Footer").is_some());
    }

    #[test]
    fn card_description_scopes_inherited_text_style() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            CardDescription::new("Description").into_element(cx)
        });

        let ElementKind::Text(props) = &element.kind else {
            panic!("expected CardDescription to be a text element");
        };
        assert!(props.style.is_none());
        assert!(props.color.is_none());

        let theme = fret_ui::Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_text_style.as_ref(),
            Some(&fret_ui_kit::typography::description_text_refinement(
                &theme,
                "component.card.description",
            ))
        );
        assert_eq!(
            element.inherited_foreground,
            Some(fret_ui_kit::typography::muted_foreground_color(&theme))
        );
    }

    #[test]
    fn card_description_children_scope_inherited_text_style() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            CardDescription::new_children([cx.text("Nested description")]).into_element(cx)
        });

        let nested = find_text(&element, "Nested description").expect("expected nested text child");
        assert!(nested.style.is_none());
        assert!(nested.color.is_none());

        let theme = fret_ui::Theme::global(&app).snapshot();
        assert_eq!(
            element.inherited_text_style.as_ref(),
            Some(&fret_ui_kit::typography::description_text_refinement(
                &theme,
                "component.card.description",
            ))
        );
        assert_eq!(
            element.inherited_foreground,
            Some(fret_ui_kit::typography::muted_foreground_color(&theme))
        );
    }

    #[test]
    fn card_title_children_patch_rich_text_with_title_typography() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(120.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let rich = AttributedText::new(
                Arc::<str>::from("Card title rendered from a rich text child"),
                Arc::<[TextSpan]>::from([TextSpan::new(
                    "Card title rendered from a rich text child".len(),
                )]),
            );

            CardTitle::new_children([cx.styled_text(rich)]).into_element(cx)
        });

        let ElementKind::StyledText(props) = &element.kind else {
            panic!(
                "expected CardTitle::new_children(single child) to keep the rich text node, got {:?}",
                element.kind
            );
        };

        let style = props
            .style
            .as_ref()
            .expect("expected CardTitle children to receive explicit title text style");
        let theme = Theme::global(&app);
        let expected_px = theme
            .metric_by_key("component.card.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_token("font.size"));
        let expected_line_height = theme
            .metric_by_key("component.card.title_line_height")
            .unwrap_or(expected_px);

        assert_eq!(style.size, expected_px);
        assert_eq!(style.weight, FontWeight::SEMIBOLD);
        assert_eq!(style.line_height, Some(expected_line_height));
        assert_eq!(props.wrap, TextWrap::Word);
        assert_eq!(props.overflow, TextOverflow::Clip);
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(
            element.inherited_foreground,
            Some(theme.color_token("card-foreground"))
        );
    }

    #[test]
    fn card_title_children_helper_accepts_late_landed_children() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(320.0), Px(180.0)),
        );

        let element = fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            card(|cx| {
                ui::children![
                    cx;
                    card_header(|cx| {
                        ui::children![
                            cx;
                            card_title_children(|cx| ui::children![
                                cx;
                                cx.styled_text(AttributedText::new(
                                    Arc::<str>::from("Nested title"),
                                    Arc::<[TextSpan]>::from([TextSpan::new("Nested title".len())]),
                                )),
                            ]),
                        ]
                    }),
                ]
            })
            .into_element(cx)
        });

        let title = find_first_styled_text(&element)
            .expect("expected card_title_children(...) to keep the styled text node");
        let style = title
            .style
            .as_ref()
            .expect("expected card_title_children(...) to patch styled text typography");
        assert_eq!(style.weight, FontWeight::SEMIBOLD);
        assert_eq!(title.wrap, TextWrap::Word);
    }

    #[test]
    fn card_root_has_default_vertical_padding_and_visible_overflow() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let theme = Theme::global(&*cx.app);
            let fg = theme.color_token("card-foreground");
            let py = fret_ui_kit::MetricRef::space(Space::N6).resolve(theme);
            let el = Card::new([cx.text("body")]).into_element(cx);

            assert_eq!(
                el.inherited_foreground,
                Some(fg),
                "expected Card to install `text-card-foreground` behavior on the existing root"
            );

            let card = &el;
            let ElementKind::Container(ContainerProps {
                layout, padding, ..
            }) = &card.kind
            else {
                panic!("expected Card surface to be a container element");
            };

            assert_eq!(layout.overflow, Overflow::Visible);
            assert_eq!(layout.size.width, Length::Auto);
            assert_eq!(padding.top, py.into());
            assert_eq!(padding.right, Px(0.0).into());
            assert_eq!(padding.bottom, py.into());
            assert_eq!(padding.left, Px(0.0).into());

            let inner = card
                .children
                .first()
                .unwrap_or_else(|| panic!("expected Card surface to contain an inner stack"));
            let ElementKind::Container(ContainerProps {
                layout: inner_layout,
                ..
            }) = &inner.kind
            else {
                panic!("expected Card inner stack wrapper to be a container element");
            };
            assert_eq!(
                inner_layout.size.width,
                Length::Fill,
                "expected Card inner stack wrapper to request fill width so nested sections inherit a definite inline-size budget"
            );
        });
    }

    #[test]
    fn card_content_uses_px_only() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = CardContent::new(Vec::<AnyElement>::new()).into_element(cx);

            let ElementKind::Container(ContainerProps { padding, .. }) = &el.kind else {
                panic!("expected CardContent to be a container element");
            };

            assert_eq!(padding.top, Px(0.0).into());
            assert_eq!(padding.bottom, Px(0.0).into());
            assert_eq!(padding.left, padding.right);
            assert!(
                matches!(
                    padding.left,
                    fret_ui::element::SpacingLength::Px(px) if px.0 > 0.0
                ),
                "expected CardContent horizontal padding to be px-only and positive, got {:?}",
                padding.left
            );
        });
    }

    #[test]
    fn card_content_does_not_stretch_children_by_default() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let button = crate::button::Button::new("Inline").into_element(cx);
            let el = CardContent::new([button]).into_element(cx);

            let child = el
                .children
                .first()
                .unwrap_or_else(|| panic!("expected CardContent to have a single vstack child"));

            let ElementKind::Container(ContainerProps {
                layout: wrapper_layout,
                ..
            }) = &child.kind
            else {
                panic!("expected CardContent child to be a fill-width wrapper container");
            };

            let inner = child.children.first().unwrap_or_else(|| {
                panic!("expected CardContent fill-width wrapper to contain an inner flex root")
            });

            let ElementKind::Flex(FlexProps {
                align,
                direction,
                layout,
                ..
            }) = &inner.kind
            else {
                panic!("expected CardContent wrapper child to be a flex element");
            };

            assert_eq!(
                wrapper_layout.size.width,
                Length::Fill,
                "expected CardContent inner wrapper to request fill width so wrapped text resolves against the card's inner width"
            );
            assert_eq!(
                wrapper_layout.size.min_width,
                Some(Length::Px(Px(0.0))),
                "expected CardContent inner wrapper to opt into min-w-0 so nested flex/text content can shrink and wrap"
            );
            assert_eq!(
                *direction,
                Axis::Vertical,
                "expected CardContent inner flow root to stay vertical"
            );
            assert_eq!(
                *align,
                CrossAlign::Start,
                "expected CardContent to avoid cross-axis stretch so inline-sized children (e.g. buttons) do not fill the card width"
            );
            assert_eq!(
                layout.size.width,
                Length::Fill,
                "expected CardContent inner flow root to request fill width so wrapped text resolves against the card's inner width"
            );
        });
    }

    #[test]
    fn card_header_without_action_uses_fill_width_flow_root() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = CardHeader::new([
                CardTitle::new("Overview").into_element(cx),
                CardDescription::new(
                    "Window / event / UiTree / renderer contracts (mechanisms & boundaries)",
                )
                .into_element(cx),
            ])
            .into_element(cx);

            let child = el
                .children
                .first()
                .unwrap_or_else(|| panic!("expected CardHeader to have a single inner flow child"));

            let ElementKind::Container(ContainerProps {
                layout: wrapper_layout,
                ..
            }) = &child.kind
            else {
                panic!("expected CardHeader child to be a fill-width wrapper container");
            };

            let inner = child.children.first().unwrap_or_else(|| {
                panic!("expected CardHeader fill-width wrapper to contain an inner flex root")
            });

            let ElementKind::Flex(FlexProps {
                align,
                direction,
                layout,
                ..
            }) = &inner.kind
            else {
                panic!("expected CardHeader wrapper child to be a flex element");
            };

            assert_eq!(
                wrapper_layout.size.width,
                Length::Fill,
                "expected CardHeader inner wrapper to request fill width so wrapped title/description text resolves against the card width"
            );
            assert_eq!(
                wrapper_layout.size.min_width,
                Some(Length::Px(Px(0.0))),
                "expected CardHeader inner wrapper to opt into min-w-0 so nested text can shrink and wrap in narrow cards"
            );
            assert_eq!(
                *direction,
                Axis::Vertical,
                "expected CardHeader inner flow root to stay vertical"
            );
            assert_eq!(
                *align,
                CrossAlign::Start,
                "expected CardHeader without an action slot to avoid cross-axis stretch on the inner flow root"
            );
            assert_eq!(
                layout.size.width,
                Length::Fill,
                "expected CardHeader inner flow root to request fill width so wrapped title/description text resolves against the card width"
            );
        });
    }

    #[test]
    fn card_sections_can_inherit_or_override_size() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let theme = Theme::global(&*cx.app);
            let px_default = MetricRef::space(Space::N6).resolve(theme);
            let px_sm = MetricRef::space(Space::N4).resolve(theme);

            let default_header_el = CardHeader::new(Vec::<AnyElement>::new()).into_element(cx);
            let ElementKind::Container(ContainerProps {
                padding: default_header_padding,
                ..
            }) = &default_header_el.kind
            else {
                panic!("expected CardHeader to be a container element");
            };
            assert_eq!(default_header_padding.left, px_default.into());
            assert_eq!(default_header_padding.right, px_default.into());

            let explicit_header_el = CardHeader::new(Vec::<AnyElement>::new())
                .size(CardSize::Sm)
                .into_element(cx);
            let ElementKind::Container(ContainerProps {
                padding: explicit_header_padding,
                ..
            }) = &explicit_header_el.kind
            else {
                panic!("expected CardHeader(size=Sm) to be a container element");
            };
            assert_eq!(explicit_header_padding.left, px_sm.into());
            assert_eq!(explicit_header_padding.right, px_sm.into());

            let inherited_header_el = with_card_size_provider(cx, CardSize::Sm, |cx| {
                CardHeader::new(Vec::<AnyElement>::new()).into_element(cx)
            });
            let ElementKind::Container(ContainerProps {
                padding: inherited_header_padding,
                ..
            }) = &inherited_header_el.kind
            else {
                panic!("expected CardHeader(inherited Sm) to be a container element");
            };
            assert_eq!(inherited_header_padding.left, px_sm.into());
            assert_eq!(inherited_header_padding.right, px_sm.into());

            let default_el = CardContent::new(Vec::<AnyElement>::new()).into_element(cx);
            let ElementKind::Container(ContainerProps {
                padding: default_padding,
                ..
            }) = &default_el.kind
            else {
                panic!("expected CardContent to be a container element");
            };
            assert_eq!(default_padding.left, px_default.into());
            assert_eq!(default_padding.right, px_default.into());

            let explicit_el = CardContent::new(Vec::<AnyElement>::new())
                .size(CardSize::Sm)
                .into_element(cx);
            let ElementKind::Container(ContainerProps {
                padding: explicit_padding,
                ..
            }) = &explicit_el.kind
            else {
                panic!("expected CardContent(size=Sm) to be a container element");
            };
            assert_eq!(explicit_padding.left, px_sm.into());
            assert_eq!(explicit_padding.right, px_sm.into());

            let inherited_el = with_card_size_provider(cx, CardSize::Sm, |cx| {
                CardContent::new(Vec::<AnyElement>::new()).into_element(cx)
            });
            let ElementKind::Container(ContainerProps {
                padding: inherited_padding,
                ..
            }) = &inherited_el.kind
            else {
                panic!("expected CardContent(inherited Sm) to be a container element");
            };
            assert_eq!(inherited_padding.left, px_sm.into());
            assert_eq!(inherited_padding.right, px_sm.into());

            let default_footer_el = CardFooter::new(Vec::<AnyElement>::new()).into_element(cx);
            let ElementKind::Container(ContainerProps {
                padding: default_footer_padding,
                ..
            }) = &default_footer_el.kind
            else {
                panic!("expected CardFooter to be a container element");
            };
            assert_eq!(default_footer_padding.left, px_default.into());
            assert_eq!(default_footer_padding.right, px_default.into());

            let explicit_footer_el = CardFooter::new(Vec::<AnyElement>::new())
                .size(CardSize::Sm)
                .into_element(cx);
            let ElementKind::Container(ContainerProps {
                padding: explicit_footer_padding,
                ..
            }) = &explicit_footer_el.kind
            else {
                panic!("expected CardFooter(size=Sm) to be a container element");
            };
            assert_eq!(explicit_footer_padding.left, px_sm.into());
            assert_eq!(explicit_footer_padding.right, px_sm.into());

            let inherited_footer_el = with_card_size_provider(cx, CardSize::Sm, |cx| {
                CardFooter::new(Vec::<AnyElement>::new()).into_element(cx)
            });
            let ElementKind::Container(ContainerProps {
                padding: inherited_footer_padding,
                ..
            }) = &inherited_footer_el.kind
            else {
                panic!("expected CardFooter(inherited Sm) to be a container element");
            };
            assert_eq!(inherited_footer_padding.left, px_sm.into());
            assert_eq!(inherited_footer_padding.right, px_sm.into());
        });
    }

    #[test]
    fn card_header_build_matches_eager_defaults() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let eager = CardHeader::new(Vec::<AnyElement>::new()).into_element(cx);
            let built = CardHeader::build(|_cx, _out| {}).into_element(cx);

            let ElementKind::Container(ContainerProps {
                padding: eager_padding,
                ..
            }) = &eager.kind
            else {
                panic!("expected eager CardHeader to be a container element");
            };
            let ElementKind::Container(ContainerProps {
                padding: built_padding,
                ..
            }) = &built.kind
            else {
                panic!("expected built CardHeader to be a container element");
            };

            assert_eq!(built_padding.top, eager_padding.top);
            assert_eq!(built_padding.right, eager_padding.right);
            assert_eq!(built_padding.bottom, eager_padding.bottom);
            assert_eq!(built_padding.left, eager_padding.left);
        });
    }

    #[test]
    fn card_build_matches_eager_defaults() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let eager = Card::new(Vec::<AnyElement>::new()).into_element(cx);
            let built = Card::build(|_cx, _out| {}).into_element(cx);

            let eager_foreground = eager.inherited_foreground;
            let built_foreground = built.inherited_foreground;
            assert_eq!(built_foreground, eager_foreground);

            let ElementKind::Container(ContainerProps {
                layout: eager_layout,
                padding: eager_padding,
                ..
            }) = &eager.kind
            else {
                panic!("expected eager Card root to be a container element");
            };
            let ElementKind::Container(ContainerProps {
                layout: built_layout,
                padding: built_padding,
                ..
            }) = &built.kind
            else {
                panic!("expected built Card root to be a container element");
            };

            assert_eq!(built_layout.overflow, eager_layout.overflow);
            assert_eq!(built_layout.size.width, eager_layout.size.width);
            assert_eq!(built_padding.top, eager_padding.top);
            assert_eq!(built_padding.right, eager_padding.right);
            assert_eq!(built_padding.bottom, eager_padding.bottom);
            assert_eq!(built_padding.left, eager_padding.left);
        });
    }

    #[test]
    fn card_build_children_macro_accepts_host_bound_builders() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let children = ui::children![cx;
                Card::build(|_cx, _out| {}).ui().w_full(),
                CardHeader::build(|_cx, _out| {}).ui().w_full(),
                CardContent::build(|_cx, _out| {}).ui().w_full(),
            ];

            assert_eq!(children.len(), 3);
            assert!(matches!(children[0].kind, ElementKind::Container(_)));
            assert!(children[0].inherited_foreground.is_some());
            assert!(matches!(children[1].kind, ElementKind::Container(_)));
            assert!(matches!(children[2].kind, ElementKind::Container(_)));
        });
    }

    #[test]
    fn card_build_push_ui_accepts_host_bound_builders() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let mut out = Vec::new();
            out.push_ui(cx, Card::build(|_cx, _out| {}));
            out.push_ui(cx, CardHeader::build(|_cx, _out| {}));
            out.push_ui(cx, CardContent::build(|_cx, _out| {}));

            assert_eq!(out.len(), 3);
            assert!(matches!(out[0].kind, ElementKind::Container(_)));
            assert!(out[0].inherited_foreground.is_some());
            assert!(matches!(out[1].kind, ElementKind::Container(_)));
            assert!(matches!(out[2].kind, ElementKind::Container(_)));
        });
    }

    #[test]
    fn card_build_ui_builder_path_applies_layout_patches() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let theme = Theme::global(&*cx.app);
            let background = theme.color_token("background");
            let border = theme.color_token("border");

            let card = Card::build(|_cx, _out| {})
                .ui()
                .bg(ColorRef::Color(background))
                .border_1()
                .border_color(ColorRef::Color(border))
                .max_w(Px(320.0))
                .into_element(cx);
            let header = CardHeader::build(|_cx, _out| {})
                .ui()
                .max_w(Px(240.0))
                .into_element(cx);
            let content = CardContent::build(|_cx, _out| {})
                .ui()
                .max_w(Px(200.0))
                .into_element(cx);

            let ElementKind::Container(ContainerProps {
                layout: card_layout,
                background: card_background,
                border_color: card_border_color,
                ..
            }) = &card.kind
            else {
                panic!("expected ui()-patched Card root to be a container element");
            };
            let ElementKind::Container(ContainerProps {
                layout: header_layout,
                ..
            }) = &header.kind
            else {
                panic!("expected ui()-patched CardHeader to be a container element");
            };
            let ElementKind::Container(ContainerProps {
                layout: content_layout,
                ..
            }) = &content.kind
            else {
                panic!("expected ui()-patched CardContent to be a container element");
            };

            assert_eq!(card_background, &Some(background));
            assert_eq!(card_border_color, &Some(border));
            assert_eq!(card_layout.size.max_width, Some(Length::Px(Px(320.0))));
            assert_eq!(header_layout.size.max_width, Some(Length::Px(Px(240.0))));
            assert_eq!(content_layout.size.max_width, Some(Length::Px(Px(200.0))));
        });
    }

    #[test]
    fn card_build_provides_inherited_sm_size_to_sections() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            fn find_descendant_with_children<'a>(
                root: &'a AnyElement,
                child_count: usize,
            ) -> &'a AnyElement {
                let mut stack = vec![root];
                while let Some(node) = stack.pop() {
                    if node.children.len() >= child_count {
                        return node;
                    }
                    for child in node.children.iter().rev() {
                        stack.push(child);
                    }
                }
                panic!("expected descendant with at least {child_count} children");
            }

            let theme = Theme::global(&*cx.app);
            let px_sm = MetricRef::space(Space::N4).resolve(theme);
            let built = Card::build(|cx, out| {
                out.push(CardHeader::build(|_cx, _out| {}).into_element(cx));
                out.push(CardContent::build(|_cx, _out| {}).into_element(cx));
            })
            .size(CardSize::Sm)
            .into_element(cx);

            let stack = find_descendant_with_children(&built, 2);
            let header = stack
                .children
                .first()
                .unwrap_or_else(|| panic!("expected built Card header child"));
            let content = stack
                .children
                .get(1)
                .unwrap_or_else(|| panic!("expected built Card content child"));

            let ElementKind::Container(ContainerProps {
                padding: header_padding,
                ..
            }) = &header.kind
            else {
                panic!("expected built Card header to be a container element");
            };
            let ElementKind::Container(ContainerProps {
                padding: content_padding,
                ..
            }) = &content.kind
            else {
                panic!("expected built Card content to be a container element");
            };

            assert_eq!(header_padding.left, px_sm.into());
            assert_eq!(header_padding.right, px_sm.into());
            assert_eq!(content_padding.left, px_sm.into());
            assert_eq!(content_padding.right, px_sm.into());
        });
    }

    #[test]
    fn card_content_build_matches_eager_defaults() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let eager = CardContent::new(Vec::<AnyElement>::new()).into_element(cx);
            let built = CardContent::build(|_cx, _out| {}).into_element(cx);

            let ElementKind::Container(ContainerProps {
                padding: eager_padding,
                ..
            }) = &eager.kind
            else {
                panic!("expected eager CardContent to be a container element");
            };
            let ElementKind::Container(ContainerProps {
                padding: built_padding,
                ..
            }) = &built.kind
            else {
                panic!("expected built CardContent to be a container element");
            };

            assert_eq!(built_padding.top, eager_padding.top);
            assert_eq!(built_padding.right, eager_padding.right);
            assert_eq!(built_padding.bottom, eager_padding.bottom);
            assert_eq!(built_padding.left, eager_padding.left);
        });
    }

    #[test]
    fn card_content_build_children_see_card_content_surface_slot() {
        use std::cell::Cell;

        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );
        let seen_slot = Cell::new(None::<crate::surface_slot::ShadcnSurfaceSlot>);

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let seen_slot = &seen_slot;
            let _ = CardContent::build(move |cx, _out| {
                seen_slot.set(crate::surface_slot::surface_slot_in_scope(cx));
            })
            .into_element(cx);
        });

        assert_eq!(
            seen_slot.get(),
            Some(crate::surface_slot::ShadcnSurfaceSlot::CardContent)
        );
    }

    #[test]
    fn card_header_border_bottom_adds_pb_6() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let theme = Theme::global(&*cx.app);
            let pb = MetricRef::space(Space::N6).resolve(theme);

            let el = CardHeader::new(Vec::<AnyElement>::new())
                .border_bottom(true)
                .into_element(cx);

            fn has_header_padding(el: &AnyElement, pb: Px) -> bool {
                let mut stack = vec![el];
                while let Some(node) = stack.pop() {
                    if let ElementKind::Container(ContainerProps { padding, .. }) = &node.kind {
                        if padding.bottom == pb.into()
                            && padding.left == padding.right
                            && matches!(
                                padding.left,
                                fret_ui::element::SpacingLength::Px(px) if px.0 > 0.0
                            )
                        {
                            return true;
                        }
                    }
                    for child in &node.children {
                        stack.push(child);
                    }
                }
                false
            }

            assert!(
                has_header_padding(&el, pb),
                "expected CardHeader(border_bottom=true) to apply pb-6 to the padded header container"
            );
        });
    }

    #[test]
    fn card_header_border_bottom_uses_smaller_pb_for_sm_cards() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let theme = Theme::global(&*cx.app);
            let pb_sm = MetricRef::space(Space::N4).resolve(theme);

            let el = with_card_size_provider(cx, CardSize::Sm, |cx| {
                CardHeader::new(Vec::<AnyElement>::new())
                    .border_bottom(true)
                    .into_element(cx)
            });

            fn has_header_padding(el: &AnyElement, pb: Px) -> bool {
                let mut stack = vec![el];
                while let Some(node) = stack.pop() {
                    if let ElementKind::Container(ContainerProps { padding, .. }) = &node.kind {
                        if padding.bottom == pb.into()
                            && padding.left == padding.right
                            && matches!(
                                padding.left,
                                fret_ui::element::SpacingLength::Px(px) if px.0 > 0.0
                            )
                        {
                            return true;
                        }
                    }
                    for child in &node.children {
                        stack.push(child);
                    }
                }
                false
            }

            assert!(
                has_header_padding(&el, pb_sm),
                "expected CardHeader(border_bottom=true,size=sm) to apply pb-4 to the padded header container"
            );
        });
    }

    #[test]
    fn card_footer_border_top_uses_smaller_pt_for_sm_cards() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let theme = Theme::global(&*cx.app);
            let pt_sm = MetricRef::space(Space::N4).resolve(theme);

            let el = with_card_size_provider(cx, CardSize::Sm, |cx| {
                CardFooter::new(Vec::<AnyElement>::new())
                    .border_top(true)
                    .into_element(cx)
            });

            fn has_footer_padding(el: &AnyElement, pt: Px) -> bool {
                let mut stack = vec![el];
                while let Some(node) = stack.pop() {
                    if let ElementKind::Container(ContainerProps { padding, .. }) = &node.kind {
                        if padding.top == pt.into()
                            && padding.left == padding.right
                            && matches!(
                                padding.left,
                                fret_ui::element::SpacingLength::Px(px) if px.0 > 0.0
                            )
                        {
                            return true;
                        }
                    }
                    for child in &node.children {
                        stack.push(child);
                    }
                }
                false
            }

            assert!(
                has_footer_padding(&el, pt_sm),
                "expected CardFooter(border_top=true,size=sm) to apply pt-4 to the padded footer container"
            );
        });
    }

    #[test]
    fn card_footer_column_uses_vertical_flex() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = CardFooter::new([cx.text("A"), cx.text("B")])
                .direction(CardFooterDirection::Column)
                .gap(Space::N2)
                .into_element(cx);

            fn find_flex_direction(el: &AnyElement) -> Option<Axis> {
                let mut stack = vec![el];
                while let Some(node) = stack.pop() {
                    if let ElementKind::Flex(props) = &node.kind {
                        return Some(props.direction);
                    }
                    for child in &node.children {
                        stack.push(child);
                    }
                }
                None
            }

            assert_eq!(
                find_flex_direction(&el),
                Some(Axis::Vertical),
                "expected CardFooter(direction=Column) to emit a vertical flex node"
            );
        });
    }

    #[test]
    fn card_footer_row_requests_fill_width_and_min_w_0() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = CardFooter::new([cx.text("Footer only.")]).into_element(cx);

            let ElementKind::Container(ContainerProps {
                layout: root_layout,
                ..
            }) = &el.kind
            else {
                panic!("expected CardFooter root to be a container element");
            };

            fn find_flex<'a>(el: &'a AnyElement) -> &'a FlexProps {
                let mut stack = vec![el];
                while let Some(node) = stack.pop() {
                    if let ElementKind::Flex(props) = &node.kind {
                        return props;
                    }
                    for child in node.children.iter().rev() {
                        stack.push(child);
                    }
                }
                panic!("expected CardFooter subtree to contain a flex root");
            }

            let FlexProps {
                align,
                direction,
                justify,
                layout,
                ..
            } = find_flex(&el);

            assert_eq!(
                *direction,
                Axis::Horizontal,
                "expected CardFooter(direction=Row) to emit a horizontal flex node"
            );
            assert_eq!(
                *align,
                CrossAlign::Center,
                "expected CardFooter row to keep the upstream `items-center` outcome"
            );
            assert_eq!(
                *justify,
                MainAlign::Start,
                "expected CardFooter row to keep start main-axis alignment by default"
            );
            assert_eq!(
                layout.size.width,
                Length::Fill,
                "expected CardFooter row to request fill width so footer-only text resolves against the card's inner width"
            );
            assert_eq!(
                root_layout.size.min_width,
                Some(Length::Px(Px(0.0))),
                "expected CardFooter root to opt into min-w-0 so wrapped text can shrink without collapsing to per-word lines"
            );
        });
    }

    #[test]
    fn card_footer_justify_end_aligns_row_content_to_main_end() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let bounds = Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(400.0), Px(300.0)),
        );

        fret_ui::elements::with_element_cx(&mut app, window, bounds, "test", |cx| {
            let el = CardFooter::new([cx.text("Footer only.")])
                .justify_end()
                .into_element(cx);

            fn find_flex<'a>(el: &'a AnyElement) -> &'a FlexProps {
                let mut stack = vec![el];
                while let Some(node) = stack.pop() {
                    if let ElementKind::Flex(props) = &node.kind {
                        return props;
                    }
                    for child in node.children.iter().rev() {
                        stack.push(child);
                    }
                }
                panic!("expected CardFooter subtree to contain a flex root");
            }

            let FlexProps {
                direction, justify, ..
            } = find_flex(&el);

            assert_eq!(
                *direction,
                Axis::Horizontal,
                "expected CardFooter(justify_end) to keep the default row direction"
            );
            assert_eq!(
                *justify,
                MainAlign::End,
                "expected CardFooter(justify_end) to align row content to the main-axis end"
            );
        });
    }
}

#[derive(Debug)]
pub struct CardContent {
    children: Vec<AnyElement>,
    size: Option<CardSize>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl CardContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            size: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Builder-first variant that collects children at `into_element(cx)` time.
    pub fn build<H: UiHost, B>(build: B) -> CardContentBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        CardContentBuild {
            build: Some(build),
            size: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            _phantom: PhantomData,
        }
    }

    /// Explicitly set the card size for this section.
    ///
    /// Most compositions rely on `Card` installing a size provider; however, some callers build
    /// `CardHeader` / `CardContent` / `CardFooter` elements before they are inserted into a
    /// `Card` subtree. In those cases, inherited size is unavailable, so callers can pass an
    /// explicit size to match upstream shadcn behavior.
    pub fn size(mut self, size: CardSize) -> Self {
        self.size = Some(size);
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
        let size = self.size.unwrap_or_else(|| card_size_in_scope(cx));
        let p = match size {
            CardSize::Default => Space::N6,
            CardSize::Sm => Space::N4,
        };
        let props = {
            let theme = Theme::global(&*cx.app);
            decl_style::container_props(
                theme,
                // shadcn/ui v4: `px-6` (horizontal padding only; vertical padding lives on Card).
                ChromeRefinement::default().px(p).merge(self.chrome),
                LayoutRefinement::default().w_full().merge(self.layout),
            )
        };
        let children = self.children;
        with_surface_slot_provider(cx, ShadcnSurfaceSlot::CardContent, |cx| {
            shadcn_layout::container_vstack_fill_width(
                cx,
                props,
                // Upstream shadcn/ui `CardContent` is a plain `div` (`px-6`) rather than a flex
                // container, so avoid the default `items: stretch` behavior that would expand
                // inline-sized children (e.g. buttons) to fill the card width. Still request a
                // fill-width flow root so wrapped text resolves against the section's inner width
                // instead of shrink-wrapping to an intrinsic/min-content width.
                shadcn_layout::VStackProps::default().items_start(),
                children,
            )
        })
    }
}

pub struct CardContentBuild<H, B> {
    build: Option<B>,
    size: Option<CardSize>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> CardContentBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    pub fn size(mut self, size: CardSize) -> Self {
        self.size = Some(size);
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
        let size = self.size.unwrap_or_else(|| card_size_in_scope(cx));
        let p = match size {
            CardSize::Default => Space::N6,
            CardSize::Sm => Space::N4,
        };
        let props = {
            let theme = Theme::global(&*cx.app);
            decl_style::container_props(
                theme,
                // Keep the build-path parity with `CardContent::into_element`: children built
                // through `raw::card::card_content(...)` must observe the CardContent surface slot
                // while they are rendered (e.g. Calendar -> bg-transparent inside CardContent).
                ChromeRefinement::default().px(p).merge(self.chrome),
                LayoutRefinement::default().w_full().merge(self.layout),
            )
        };
        let build = self.build.expect("expected card content build closure");

        with_surface_slot_provider(cx, ShadcnSurfaceSlot::CardContent, |cx| {
            let children = collect_built_card_children(cx, build);
            shadcn_layout::container_vstack_fill_width(
                cx,
                props,
                shadcn_layout::VStackProps::default().items_start(),
                children,
            )
        })
    }
}

impl<H: UiHost, B> UiPatchTarget for CardContentBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_style(patch.chrome).refine_layout(patch.layout)
    }
}

impl<H: UiHost, B> UiSupportsChrome for CardContentBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> UiSupportsLayout for CardContentBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> IntoUiElement<H> for CardContentBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        CardContentBuild::into_element(self, cx)
    }
}

#[derive(Debug)]
pub struct CardFooter {
    children: Vec<AnyElement>,
    size: Option<CardSize>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    border_top: bool,
    direction: CardFooterDirection,
    justify: Justify,
    gap: Space,
    wrap: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardFooterDirection {
    Row,
    Column,
}

impl Default for CardFooterDirection {
    fn default() -> Self {
        Self::Row
    }
}

impl CardFooter {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            size: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            border_top: false,
            direction: CardFooterDirection::Row,
            justify: Justify::Start,
            gap: Space::N0.into(),
            wrap: false,
        }
    }

    /// Builder-first variant that collects children at `into_element(cx)` time.
    pub fn build<H: UiHost, B>(build: B) -> CardFooterBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        CardFooterBuild {
            build: Some(build),
            size: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            border_top: false,
            direction: CardFooterDirection::Row,
            justify: Justify::Start,
            gap: Space::N0.into(),
            wrap: false,
            _phantom: PhantomData,
        }
    }

    /// Explicitly set the card size for this section.
    ///
    /// See `CardContent::size(...)` for why some call sites need this.
    pub fn size(mut self, size: CardSize) -> Self {
        self.size = Some(size);
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

    pub fn border_top(mut self, value: bool) -> Self {
        self.border_top = value;
        self
    }

    pub fn direction(mut self, direction: CardFooterDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    pub fn justify_start(self) -> Self {
        self.justify(Justify::Start)
    }

    pub fn justify_center(self) -> Self {
        self.justify(Justify::Center)
    }

    pub fn justify_end(self) -> Self {
        self.justify(Justify::End)
    }

    pub fn justify_between(self) -> Self {
        self.justify(Justify::Between)
    }

    pub fn gap(mut self, gap: Space) -> Self {
        self.gap = gap;
        self
    }

    pub fn wrap(mut self, value: bool) -> Self {
        self.wrap = value;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let size = self.size.unwrap_or_else(|| card_size_in_scope(cx));
        let p = match size {
            CardSize::Default => Space::N6,
            CardSize::Sm => Space::N4,
        };
        let pt = match size {
            CardSize::Default => Space::N6,
            CardSize::Sm => Space::N4,
        };
        let border_top = self.border_top;
        let children = self.children;
        let chrome = self.chrome;
        let layout = self.layout;
        let direction = self.direction;
        let justify = self.justify;
        let gap = self.gap;
        let wrap = self.wrap;

        let inner_props = {
            let theme = Theme::global(&*cx.app);
            let base = if border_top {
                // shadcn/ui v4: `flex items-center px-6` and `[.border-t]:pt-6` (vertical padding
                // lives on Card).
                ChromeRefinement::default().px(p).pt(pt)
            } else {
                // shadcn/ui v4: `flex items-center px-6` (vertical padding lives on Card).
                ChromeRefinement::default().px(p)
            };
            decl_style::container_props(
                theme,
                base.merge(chrome),
                LayoutRefinement::default().w_full().min_w_0().merge(layout),
            )
        };

        let inner = cx.container(inner_props, move |cx| {
            let mut children = Some(children);
            vec![match direction {
                CardFooterDirection::Row => {
                    let children = children
                        .take()
                        .unwrap_or_else(|| panic!("expected CardFooter children to be available"));
                    if wrap {
                        ui::h_flex(move |_cx| children)
                            .wrap()
                            .gap(gap)
                            .items_center()
                            .justify(justify)
                            .layout(LayoutRefinement::default().w_full())
                            .into_element(cx)
                    } else {
                        ui::h_flex(move |_cx| children)
                            .gap(gap)
                            .items_center()
                            .justify(justify)
                            .layout(LayoutRefinement::default().w_full())
                            .into_element(cx)
                    }
                }
                CardFooterDirection::Column => {
                    let children = children
                        .take()
                        .unwrap_or_else(|| panic!("expected CardFooter children to be available"));
                    // shadcn/ui v4: `flex-col` uses the default `items-stretch` behavior.
                    ui::v_flex(move |_cx| children)
                        .gap(gap)
                        .justify(justify)
                        .layout(LayoutRefinement::default().w_full())
                        .into_element(cx)
                }
            }]
        });

        let el = if border_top {
            let outer_props = {
                let theme = Theme::global(&*cx.app);
                decl_style::container_props(
                    theme,
                    ChromeRefinement::default(),
                    LayoutRefinement::default().w_full(),
                )
            };

            let separator = crate::separator::Separator::new().into_element(cx);
            shadcn_layout::container_vstack(
                cx,
                outer_props,
                shadcn_layout::VStackProps::default()
                    .gap(Space::N0)
                    .layout(LayoutRefinement::default().w_full()),
                vec![separator, inner],
            )
        } else {
            inner
        };

        let marker: Arc<str> = Arc::from(format!("{}:{}", CARD_FOOTER_MARKER_PREFIX, el.id.0));
        attach_test_id(el, marker)
    }
}

pub struct CardFooterBuild<H, B> {
    build: Option<B>,
    size: Option<CardSize>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    border_top: bool,
    direction: CardFooterDirection,
    justify: Justify,
    gap: Space,
    wrap: bool,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> CardFooterBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    pub fn size(mut self, size: CardSize) -> Self {
        self.size = Some(size);
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

    pub fn border_top(mut self, value: bool) -> Self {
        self.border_top = value;
        self
    }

    pub fn direction(mut self, direction: CardFooterDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    pub fn justify_start(self) -> Self {
        self.justify(Justify::Start)
    }

    pub fn justify_center(self) -> Self {
        self.justify(Justify::Center)
    }

    pub fn justify_end(self) -> Self {
        self.justify(Justify::End)
    }

    pub fn justify_between(self) -> Self {
        self.justify(Justify::Between)
    }

    pub fn gap(mut self, gap: Space) -> Self {
        self.gap = gap;
        self
    }

    pub fn wrap(mut self, value: bool) -> Self {
        self.wrap = value;
        self
    }

    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = collect_built_card_children(
            cx,
            self.build.expect("expected card footer build closure"),
        );
        let mut footer = CardFooter::new(children)
            .refine_style(self.chrome)
            .refine_layout(self.layout)
            .border_top(self.border_top)
            .direction(self.direction)
            .justify(self.justify)
            .gap(self.gap)
            .wrap(self.wrap);
        if let Some(size) = self.size {
            footer = footer.size(size);
        }
        footer.into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for CardFooterBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_style(patch.chrome).refine_layout(patch.layout)
    }
}

impl<H: UiHost, B> UiSupportsChrome for CardFooterBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> UiSupportsLayout for CardFooterBuild<H, B> where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>)
{
}

impl<H: UiHost, B> IntoUiElement<H> for CardFooterBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        CardFooterBuild::into_element(self, cx)
    }
}

#[derive(Debug)]
pub struct CardTitle {
    content: CardTitleContent,
}

#[derive(Debug)]
enum CardTitleContent {
    Text(Arc<str>),
    Children(Vec<AnyElement>),
}

impl CardTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            content: CardTitleContent::Text(text.into()),
        }
    }

    pub fn new_children(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            content: CardTitleContent::Children(children.into_iter().collect()),
        }
    }

    /// Builder-first variant that collects children at `into_element(cx)` time.
    pub fn build<H: UiHost, B>(build: B) -> CardTitleBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        CardTitleBuild {
            build: Some(build),
            _phantom: PhantomData,
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (fg, px, line_height) = {
            let theme = Theme::global(&*cx.app);
            let fg = theme.color_token("card-foreground");
            let px = theme
                .metric_by_key("component.card.title_px")
                .or_else(|| theme.metric_by_key("font.size"))
                .unwrap_or_else(|| theme.metric_token("font.size"));
            // shadcn/ui v4 CardTitle uses `leading-none` by default.
            //
            // Treat this as an outcome contract: if the theme doesn't override a specific card
            // title line-height, default to a tight line box that matches the font size.
            let line_height = theme
                .metric_by_key("component.card.title_line_height")
                .unwrap_or(px);
            (fg, px, line_height)
        };

        match self.content {
            CardTitleContent::Text(text) => ui::text(text)
                .w_full()
                .text_size_px(px)
                .line_height_px(line_height)
                .font_semibold()
                .wrap(TextWrap::Word)
                .text_color(ColorRef::Color(fg))
                .into_element(cx),
            CardTitleContent::Children(mut children) => {
                for child in &mut children {
                    patch_card_title_text_style_recursive(child, px, line_height);
                }

                let mut children =
                    current_color::scope_children(cx, ColorRef::Color(fg), |_cx| children);

                match children.len() {
                    0 => ui::text("")
                        .w_full()
                        .text_size_px(px)
                        .line_height_px(line_height)
                        .font_semibold()
                        .wrap(TextWrap::Word)
                        .text_color(ColorRef::Color(fg))
                        .into_element(cx),
                    1 => children.pop().expect("children.len() == 1"),
                    _ => ui::v_flex(move |_cx| children)
                        .gap(Space::N0)
                        .items_start()
                        .layout(LayoutRefinement::default().w_full().min_w_0())
                        .into_element(cx),
                }
            }
        }
    }
}

fn patch_card_title_text_style_recursive(el: &mut AnyElement, px: Px, line_height: Px) {
    fn patch_text_style(style: &mut Option<fret_core::TextStyle>, px: Px, line_height: Px) {
        let mut style_value = style.take().unwrap_or_default();
        style_value.size = px;
        style_value.weight = FontWeight::SEMIBOLD;
        style_value.line_height = Some(line_height);
        style_value.line_height_em = None;
        *style = Some(style_value);
    }

    fn ensure_fill_width(layout: &mut fret_ui::element::LayoutStyle) {
        if matches!(layout.size.width, Length::Auto) {
            layout.size.width = Length::Fill;
        }
        if layout.size.min_width.is_none() {
            layout.size.min_width = Some(Length::Px(Px(0.0)));
        }
    }

    match &mut el.kind {
        ElementKind::Text(props) => {
            patch_text_style(&mut props.style, px, line_height);
            ensure_fill_width(&mut props.layout);
            props.wrap = TextWrap::Word;
            props.overflow = TextOverflow::Clip;
        }
        ElementKind::StyledText(props) => {
            patch_text_style(&mut props.style, px, line_height);
            ensure_fill_width(&mut props.layout);
            props.wrap = TextWrap::Word;
            props.overflow = TextOverflow::Clip;
        }
        ElementKind::SelectableText(props) => {
            patch_text_style(&mut props.style, px, line_height);
            ensure_fill_width(&mut props.layout);
            props.wrap = TextWrap::Word;
            props.overflow = TextOverflow::Clip;
        }
        _ => {}
    }

    for child in &mut el.children {
        patch_card_title_text_style_recursive(child, px, line_height);
    }
}

pub struct CardTitleBuild<H, B> {
    build: Option<B>,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> CardTitleBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children =
            collect_built_card_children(cx, self.build.expect("expected card title build closure"));
        CardTitle::new_children(children).into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for CardTitleBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, _patch: UiPatch) -> Self {
        self
    }
}

impl<H: UiHost, B> IntoUiElement<H> for CardTitleBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        CardTitleBuild::into_element(self, cx)
    }
}

#[derive(Debug)]
pub struct CardDescription {
    content: CardDescriptionContent,
}

#[derive(Debug)]
enum CardDescriptionContent {
    Text(Arc<str>),
    Children(Vec<AnyElement>),
}

impl CardDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            content: CardDescriptionContent::Text(text.into()),
        }
    }

    pub fn new_children(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            content: CardDescriptionContent::Children(children.into_iter().collect()),
        }
    }

    /// Builder-first variant that collects children at `into_element(cx)` time.
    pub fn build<H: UiHost, B>(build: B) -> CardDescriptionBuild<H, B>
    where
        B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
    {
        CardDescriptionBuild {
            build: Some(build),
            _phantom: PhantomData,
        }
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();

        match self.content {
            CardDescriptionContent::Text(text) => scope_description_text(
                ui::raw_text(text)
                    .w_full()
                    .wrap(TextWrap::Word)
                    .into_element(cx),
                &theme,
                "component.card.description",
            ),
            CardDescriptionContent::Children(children) => scope_description_text(
                ui::v_flex(move |_cx| children)
                    .gap(Space::N1)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().min_w_0())
                    .into_element(cx),
                &theme,
                "component.card.description",
            ),
        }
    }
}

pub struct CardDescriptionBuild<H, B> {
    build: Option<B>,
    _phantom: PhantomData<fn() -> H>,
}

impl<H: UiHost, B> CardDescriptionBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    pub fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = collect_built_card_children(
            cx,
            self.build.expect("expected card description build closure"),
        );
        CardDescription::new_children(children).into_element(cx)
    }
}

impl<H: UiHost, B> UiPatchTarget for CardDescriptionBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    fn apply_ui_patch(self, _patch: UiPatch) -> Self {
        self
    }
}

impl<H: UiHost, B> IntoUiElement<H> for CardDescriptionBuild<H, B>
where
    B: FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
{
    #[track_caller]
    fn into_element(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        CardDescriptionBuild::into_element(self, cx)
    }
}
