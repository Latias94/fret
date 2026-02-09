use std::sync::Arc;

use fret_core::{Px, TextWrap};
use fret_ui::element::{AnyElement, ElementKind};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Space, ui};

use crate::layout as shadcn_layout;
use crate::surface_slot::{ShadcnSurfaceSlot, with_surface_slot_provider};
use crate::test_id::attach_test_id;

const CARD_ACTION_TEST_ID: &str = "fret-ui-shadcn.card-action";

fn is_card_action_marker(element: &AnyElement) -> bool {
    element
        .semantics_decoration
        .as_ref()
        .and_then(|d| d.test_id.as_deref())
        == Some(CARD_ACTION_TEST_ID)
        || match &element.kind {
            ElementKind::Semantics(props) => props.test_id.as_deref() == Some(CARD_ACTION_TEST_ID),
            _ => false,
        }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CardSize {
    #[default]
    Default,
    Sm,
}

#[derive(Debug, Default)]
struct CardSizeProviderState {
    current: Option<CardSize>,
}

fn card_size_in_scope<H: UiHost>(cx: &ElementContext<'_, H>) -> CardSize {
    cx.inherited_state_where::<CardSizeProviderState>(|st| st.current.is_some())
        .and_then(|st| st.current)
        .unwrap_or_default()
}

#[track_caller]
fn with_card_size_provider<H: UiHost, R>(
    cx: &mut ElementContext<'_, H>,
    size: CardSize,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> R,
) -> R {
    let prev = cx.with_state(CardSizeProviderState::default, |st| {
        let prev = st.current;
        st.current = Some(size);
        prev
    });
    let out = f(cx);
    cx.with_state(CardSizeProviderState::default, |st| {
        st.current = prev;
    });
    out
}

fn card_chrome(theme: &Theme, size: CardSize) -> ChromeRefinement {
    let bg = theme.color_required("card");
    let border = theme.color_required("border");

    // shadcn/ui v4: Card uses `rounded-xl`, which is computed from the base `--radius`.
    //
    // In the shadcn token model:
    // - `rounded-lg` ~= `--radius`
    // - `rounded-md` ~= `--radius - 2px`
    // - `rounded-xl` ~= `--radius + 4px`
    //
    // We model the base radius as `metric.radius.lg`, and derive `rounded-xl` from it to keep
    // behavior stable when the theme radius changes.
    let base_radius = theme.metric_required("metric.radius.lg");
    let rounded_xl = Px(base_radius.0 + 4.0);

    let py = match size {
        CardSize::Default => Space::N6,
        CardSize::Sm => Space::N4,
    };

    // shadcn/ui v4 card base:
    // - `rounded-xl border bg-card text-card-foreground shadow-sm`
    // - `flex flex-col gap-6 py-6` (gap handled by the inner vstack)
    ChromeRefinement::default()
        .radius(rounded_xl)
        .border_1()
        .bg(ColorRef::Color(bg))
        .border_color(ColorRef::Color(border))
        .py(py)
}

#[derive(Debug, Clone)]
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let size = self.size;
        with_card_size_provider(cx, size, |cx| {
            let theme = Theme::global(&*cx.app).clone();
            let chrome = card_chrome(&theme, size).merge(self.chrome);
            let mut props = decl_style::container_props(&theme, chrome, self.layout);
            let radius = props.corner_radii.top_left;
            props.shadow = Some(decl_style::shadow_sm(&theme, radius));
            let children = self.children;

            let gap = match size {
                CardSize::Default => Space::N6,
                CardSize::Sm => Space::N4,
            };

            // Cards behave like block containers in shadcn/ui examples: their sections are expected to
            // stretch to the card width unless explicitly constrained.
            shadcn_layout::container_vstack(
                cx,
                props,
                stack::VStackProps::default()
                    .gap(gap)
                    .layout(LayoutRefinement::default().w_full()),
                children,
            )
        })
    }
}

pub fn card<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    Card::new(f(cx)).into_element(cx)
}

pub fn card_content<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    with_surface_slot_provider(cx, ShadcnSurfaceSlot::CardContent, |cx| {
        CardContent::new(f(cx)).into_element(cx)
    })
}

#[derive(Debug, Clone)]
pub struct CardHeader {
    children: Vec<AnyElement>,
}

impl CardHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let size = card_size_in_scope(cx);
        let px = match size {
            CardSize::Default => Space::N6,
            CardSize::Sm => Space::N4,
        };
        let props = decl_style::container_props(
            &theme,
            // shadcn/ui v4: `px-6` (no default y padding; gap comes from the Card root).
            ChromeRefinement::default().px(px),
            LayoutRefinement::default().w_full(),
        );

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

        if let Some(action) = action {
            let left_col = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N2)
                    .layout(LayoutRefinement::default().flex_1().min_w_0()),
                move |_cx| left,
            );

            shadcn_layout::container_hstack(
                cx,
                props,
                stack::HStackProps::default()
                    .gap(Space::N2)
                    .layout(LayoutRefinement::default().w_full())
                    .justify_between()
                    .items_start(),
                vec![left_col, action],
            )
        } else {
            shadcn_layout::container_vstack(
                cx,
                props,
                stack::VStackProps::default()
                    .gap(Space::N2)
                    .layout(LayoutRefinement::default().w_full()),
                left,
            )
        }
    }
}

#[derive(Debug, Clone)]
pub struct CardAction {
    children: Vec<AnyElement>,
    layout: LayoutRefinement,
}

impl CardAction {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            layout: LayoutRefinement::default(),
        }
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default(),
            LayoutRefinement::default().merge(self.layout),
        );

        let children = self.children;
        let el = cx.container(props, move |cx| {
            if children.len() <= 1 {
                children
            } else {
                vec![stack::hstack(
                    cx,
                    stack::HStackProps::default().gap(Space::N2).items_center(),
                    move |_cx| children,
                )]
            }
        });

        attach_test_id(el, Arc::<str>::from(CARD_ACTION_TEST_ID))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_ui::element::{ContainerProps, SemanticsProps};
    use fret_ui::elements::GlobalElementId;

    #[test]
    fn card_action_marker_matches_semantics_decoration_test_id() {
        let el = AnyElement::new(
            GlobalElementId(1),
            ElementKind::Container(ContainerProps::default()),
            Vec::new(),
        )
        .test_id(CARD_ACTION_TEST_ID);

        assert!(is_card_action_marker(&el));
    }

    #[test]
    fn card_action_marker_matches_legacy_semantics_test_id() {
        let el = AnyElement::new(
            GlobalElementId(1),
            ElementKind::Semantics(SemanticsProps {
                test_id: Some(Arc::<str>::from(CARD_ACTION_TEST_ID)),
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
}

#[derive(Debug, Clone)]
pub struct CardContent {
    children: Vec<AnyElement>,
}

impl CardContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let size = card_size_in_scope(cx);
        let px = match size {
            CardSize::Default => Space::N6,
            CardSize::Sm => Space::N4,
        };
        let props = decl_style::container_props(
            &theme,
            // shadcn/ui v4: `px-6` (no default y padding; gap comes from the Card root).
            ChromeRefinement::default().px(px),
            LayoutRefinement::default().w_full(),
        );
        let children = self.children;
        with_surface_slot_provider(cx, ShadcnSurfaceSlot::CardContent, |cx| {
            shadcn_layout::container_vstack(
                cx,
                props,
                stack::VStackProps::default().layout(LayoutRefinement::default().w_full()),
                children,
            )
        })
    }
}

#[derive(Debug, Clone)]
pub struct CardFooter {
    children: Vec<AnyElement>,
}

impl CardFooter {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self { children }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let size = card_size_in_scope(cx);
        let px = match size {
            CardSize::Default => Space::N6,
            CardSize::Sm => Space::N4,
        };
        let props = decl_style::container_props(
            &theme,
            // shadcn/ui v4: `flex items-center px-6` (no default y padding; gap comes from the Card root).
            ChromeRefinement::default().px(px),
            LayoutRefinement::default().w_full(),
        );
        let children = self.children;
        shadcn_layout::container_hstack(
            cx,
            props,
            stack::HStackProps::default()
                .layout(LayoutRefinement::default().w_full())
                .items_center(),
            children,
        )
    }
}

#[derive(Debug, Clone)]
pub struct CardTitle {
    text: Arc<str>,
}

impl CardTitle {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme.color_required("card-foreground");

        let px = theme
            .metric_by_key("component.card.title_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.card.title_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        ui::text(cx, self.text)
            .w_full()
            .text_size_px(px)
            .line_height_px(line_height)
            .font_semibold()
            .letter_spacing_em(-0.02)
            .wrap(TextWrap::Word)
            .text_color(ColorRef::Color(fg))
            .into_element(cx)
    }
}

#[derive(Debug, Clone)]
pub struct CardDescription {
    text: Arc<str>,
}

impl CardDescription {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self { text: text.into() }
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = theme.color_required("muted-foreground");

        let px = theme
            .metric_by_key("component.card.description_px")
            .or_else(|| theme.metric_by_key("font.size"))
            .unwrap_or_else(|| theme.metric_required("font.size"));
        let line_height = theme
            .metric_by_key("component.card.description_line_height")
            .or_else(|| theme.metric_by_key("font.line_height"))
            .unwrap_or_else(|| theme.metric_required("font.line_height"));

        ui::text(cx, self.text)
            .w_full()
            .text_size_px(px)
            .line_height_px(line_height)
            .font_normal()
            .wrap(TextWrap::Word)
            .text_color(ColorRef::Color(fg))
            .into_element(cx)
    }
}
