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
        .py(Space::N6)
        .bg(ColorRef::Color(bg))
        .border_color(ColorRef::Color(border))
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let size = self.size;
        with_card_size_provider(cx, size, |cx| {
            let children = self.children;

            let props = {
                let theme = Theme::global(&*cx.app);
                let chrome = card_chrome(theme, size).merge(self.chrome);
                decl_style::container_props(theme, chrome, self.layout)
            };

            // Cards behave like block containers in shadcn/ui examples: their sections are expected to
            // stretch to the card width unless explicitly constrained.
            shadcn_layout::container_vstack(
                cx,
                props,
                stack::VStackProps::default()
                    .gap(Space::N6)
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
    chrome: ChromeRefinement,
    border_bottom: bool,
}

impl CardHeader {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            chrome: ChromeRefinement::default(),
            border_bottom: false,
        }
    }

    pub fn refine_style(mut self, style: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(style);
        self
    }

    pub fn border_bottom(mut self, value: bool) -> Self {
        self.border_bottom = value;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let size = card_size_in_scope(cx);
        let p = match size {
            CardSize::Default => Space::N6,
            CardSize::Sm => Space::N4,
        };
        let border_bottom = self.border_bottom;
        let props = {
            let theme = Theme::global(&*cx.app);
            let base = if border_bottom {
                // shadcn/ui v4: when the header has a bottom border it also adds `pb-6`.
                ChromeRefinement::default().px(p).pb(Space::N6)
            } else {
                // shadcn/ui v4: `px-6` (and `px-4` for smaller cards).
                ChromeRefinement::default().px(p)
            };
            decl_style::container_props(
                theme,
                base.merge(self.chrome),
                LayoutRefinement::default().w_full(),
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
            let left_col = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N1p5)
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
            let separator = crate::Separator::new().into_element(cx);
            shadcn_layout::container_vstack(
                cx,
                outer_props,
                stack::VStackProps::default()
                    .gap(Space::N0)
                    .layout(LayoutRefinement::default().w_full()),
                vec![content, separator],
            )
        } else {
            content
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let props = {
            let theme = Theme::global(&*cx.app);
            decl_style::container_props(
                theme,
                ChromeRefinement::default(),
                LayoutRefinement::default().merge(self.layout),
            )
        };

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

        let marker: Arc<str> = Arc::from(format!("{}:{}", CARD_ACTION_MARKER_PREFIX, el.id.0));
        attach_test_id(el, marker)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::{ContainerProps, Overflow, SemanticsProps};
    use fret_ui::elements::GlobalElementId;
    use fret_ui_kit::MetricRef;

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
            let py = fret_ui_kit::MetricRef::space(Space::N6).resolve(theme);
            let el = Card::new([cx.text("body")]).into_element(cx);

            let ElementKind::Container(ContainerProps {
                layout, padding, ..
            }) = &el.kind
            else {
                panic!("expected Card root to be a container element");
            };

            assert_eq!(layout.overflow, Overflow::Visible);
            assert_eq!(padding.top, py.into());
            assert_eq!(padding.right, Px(0.0).into());
            assert_eq!(padding.bottom, py.into());
            assert_eq!(padding.left, Px(0.0).into());
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
}

#[derive(Debug, Clone)]
pub struct CardContent {
    children: Vec<AnyElement>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl CardContent {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
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
        let size = card_size_in_scope(cx);
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
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
    border_top: bool,
    gap: Space,
    wrap: bool,
}

impl CardFooter {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        let children = children.into_iter().collect();
        Self {
            children,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
            border_top: false,
            gap: Space::N0.into(),
            wrap: false,
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

    pub fn border_top(mut self, value: bool) -> Self {
        self.border_top = value;
        self
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
        let size = card_size_in_scope(cx);
        let p = match size {
            CardSize::Default => Space::N6,
            CardSize::Sm => Space::N4,
        };
        let children = self.children;
        let chrome = self.chrome;
        let layout = self.layout;
        let gap = self.gap;
        let wrap = self.wrap;

        let el = if self.border_top {
            let outer_props = {
                let theme = Theme::global(&*cx.app);
                decl_style::container_props(
                    theme,
                    ChromeRefinement::default(),
                    LayoutRefinement::default().w_full(),
                )
            };

            let inner_props = {
                let theme = Theme::global(&*cx.app);
                // shadcn/ui v4: `flex items-center px-6` and `[.border-t]:pt-6` (vertical padding
                // lives on Card).
                decl_style::container_props(
                    theme,
                    ChromeRefinement::default()
                        .px(p)
                        .pt(Space::N6)
                        .merge(chrome),
                    LayoutRefinement::default().w_full().merge(layout),
                )
            };

            let inner = cx.container(inner_props, move |cx| {
                vec![if wrap {
                    ui::h_flex(cx, move |_cx| children)
                        .wrap()
                        .gap(gap)
                        .into_element(cx)
                } else {
                    ui::h_flex(cx, move |_cx| children)
                        .gap(gap)
                        .into_element(cx)
                }]
            });

            let separator = crate::Separator::new().into_element(cx);
            shadcn_layout::container_vstack(
                cx,
                outer_props,
                stack::VStackProps::default()
                    .gap(Space::N0)
                    .layout(LayoutRefinement::default().w_full()),
                vec![separator, inner],
            )
        } else {
            let props = {
                let theme = Theme::global(&*cx.app);
                decl_style::container_props(
                    theme,
                    // shadcn/ui v4: `flex items-center px-6` (vertical padding lives on Card).
                    ChromeRefinement::default().px(p).merge(chrome),
                    LayoutRefinement::default().w_full().merge(layout),
                )
            };

            cx.container(props, move |cx| {
                vec![if wrap {
                    ui::h_flex(cx, move |_cx| children)
                        .wrap()
                        .gap(gap)
                        .into_element(cx)
                } else {
                    ui::h_flex(cx, move |_cx| children)
                        .gap(gap)
                        .into_element(cx)
                }]
            })
        };

        let marker: Arc<str> = Arc::from(format!("{}:{}", CARD_FOOTER_MARKER_PREFIX, el.id.0));
        attach_test_id(el, marker)
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (fg, px, line_height) = {
            let theme = Theme::global(&*cx.app);
            let fg = theme.color_token("card-foreground");
            let px = theme
                .metric_by_key("component.card.title_px")
                .or_else(|| theme.metric_by_key("font.size"))
                .unwrap_or_else(|| theme.metric_token("font.size"));
            let line_height = theme
                .metric_by_key("component.card.title_line_height")
                .or_else(|| theme.metric_by_key("font.line_height"))
                .unwrap_or_else(|| theme.metric_token("font.line_height"));
            (fg, px, line_height)
        };

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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (fg, px, line_height) = {
            let theme = Theme::global(&*cx.app);
            let fg = theme.color_token("muted-foreground");
            let px = theme
                .metric_by_key("component.card.description_px")
                .or_else(|| theme.metric_by_key("font.size"))
                .unwrap_or_else(|| theme.metric_token("font.size"));
            let line_height = theme
                .metric_by_key("component.card.description_line_height")
                .or_else(|| theme.metric_by_key("font.line_height"))
                .unwrap_or_else(|| theme.metric_token("font.line_height"));
            (fg, px, line_height)
        };

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
