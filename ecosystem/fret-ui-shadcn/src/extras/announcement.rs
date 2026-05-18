use std::sync::Arc;

use crate::test_id::attach_test_id;
use fret_core::{FontWeight, Px, TextOverflow, TextStyleRefinement, TextWrap};
use fret_ui::element::{AnyElement, ElementKind, LayoutStyle, Length};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{
    ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, Radius, Space, typography, ui,
};

/// A small "announcement chip" block inspired by Kibo's shadcn blocks.
///
/// Upstream inspiration (MIT):
/// - `repo-ref/kibo/packages/announcement`
#[derive(Debug)]
pub struct Announcement {
    children: Vec<AnyElement>,
    themed: bool,
    test_id: Option<Arc<str>>,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl Announcement {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            themed: false,
            test_id: None,
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
        }
    }

    /// Enables a slightly stronger "themed" treatment (subtle border/background tweaks).
    pub fn themed(mut self, themed: bool) -> Self {
        self.themed = themed;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
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

        let border = theme.color_token("border");
        let bg = theme.color_token("background");
        let accent = theme.color_token("accent");

        let mut chrome = ChromeRefinement::default()
            .px(Space::N3)
            .py(Space::N0p5)
            .rounded(Radius::Full)
            .border_1()
            .bg(ColorRef::Color(bg))
            .border_color(ColorRef::Color(border))
            .shadow_sm();

        if self.themed {
            chrome = chrome
                .border_color(ColorRef::Token {
                    key: "foreground",
                    fallback: ColorFallback::ThemeTextPrimary,
                })
                .bg(ColorRef::Color(accent));
        }

        chrome = chrome.merge(self.chrome);

        let props = decl_style::container_props(&theme, chrome, self.layout);
        let test_id = self.test_id.clone();
        let children = self.children;

        let el = cx.container(props, move |cx| {
            vec![
                ui::h_row(|_cx| children)
                    .items_center()
                    .gap(Space::N2)
                    .into_element(cx),
            ]
        });

        attach_test_id(
            el,
            test_id.unwrap_or_else(|| Arc::<str>::from("shadcn-extras.announcement")),
        )
    }
}

#[derive(Debug, Clone)]
pub struct AnnouncementTag {
    label: Arc<str>,
    test_id: Option<Arc<str>>,
}

impl AnnouncementTag {
    pub fn new(label: impl Into<Arc<str>>) -> Self {
        Self {
            label: label.into(),
            test_id: None,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).snapshot();
        let mut bg = theme.color_token("foreground");
        bg.a = (bg.a * 0.05).clamp(0.0, 1.0);
        let chrome = ChromeRefinement::default()
            .px(Space::N2p5)
            .py(Space::N1)
            .rounded(Radius::Full)
            .bg(ColorRef::Color(bg));

        let props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());
        let el = cx.container(props, |cx| {
            vec![ui::text(self.label).text_xs().into_element(cx)]
        });
        attach_test_id(
            el,
            self.test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.announcement-tag")),
        )
    }
}

#[derive(Debug)]
pub struct AnnouncementTitle {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
}

fn announcement_title_text_refinement(theme: &Theme) -> TextStyleRefinement {
    let mut refinement = typography::composable_refinement_from_style(
        &typography::control_text_style(theme, typography::UiTextSize::Sm),
    );
    refinement.weight = Some(FontWeight::MEDIUM);
    refinement
}

fn apply_announcement_title_text_contract_recursive(el: &mut AnyElement) {
    fn apply_text_layout(
        layout: &mut LayoutStyle,
        wrap: &mut TextWrap,
        overflow: &mut TextOverflow,
    ) {
        layout.flex.shrink = 1.0;
        layout.size.min_width = Some(Length::Px(Px(0.0)));
        *wrap = TextWrap::None;
        *overflow = TextOverflow::Ellipsis;
    }

    match &mut el.kind {
        ElementKind::Text(props) => {
            apply_text_layout(&mut props.layout, &mut props.wrap, &mut props.overflow);
        }
        ElementKind::StyledText(props) => {
            apply_text_layout(&mut props.layout, &mut props.wrap, &mut props.overflow);
        }
        ElementKind::SelectableText(props) => {
            apply_text_layout(&mut props.layout, &mut props.wrap, &mut props.overflow);
        }
        _ => {}
    }

    for child in &mut el.children {
        apply_announcement_title_text_contract_recursive(child);
    }
}

impl AnnouncementTitle {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme_handle = Theme::global(&*cx.app);
        let title_refinement = announcement_title_text_refinement(theme_handle);
        let theme = theme_handle.snapshot();
        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default().py(Space::N1),
            LayoutRefinement::default()
                .flex_shrink(1.0)
                .min_w_0()
                .overflow_hidden(),
        );

        let test_id = self.test_id.clone();
        let mut children = self.children;
        for child in &mut children {
            apply_announcement_title_text_contract_recursive(child);
        }

        let el = cx.container(props, move |cx| {
            vec![
                ui::h_row(|_cx| children)
                    .items_center()
                    .gap(Space::N1)
                    .layout(
                        LayoutRefinement::default()
                            .w_full()
                            .min_w_0()
                            .flex_shrink(1.0)
                            .overflow_hidden(),
                    )
                    .into_element(cx),
            ]
        });
        let el = typography::scope_text_style(el, title_refinement);

        attach_test_id(
            el,
            test_id.unwrap_or_else(|| Arc::<str>::from("shadcn-extras.announcement-title")),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, FontWeight, Point, Rect, Size, TextOverflow, TextWrap};
    use fret_ui::Theme;
    use fret_ui::element::{ElementKind, Length, Overflow};

    fn bounds_240x120() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(240.0), Px(120.0)),
        )
    }

    fn find_text<'a>(element: &'a AnyElement, text: &str) -> Option<&'a AnyElement> {
        match &element.kind {
            ElementKind::Text(props) if props.text.as_ref() == text => Some(element),
            _ => element
                .children
                .iter()
                .find_map(|child| find_text(child, text)),
        }
    }

    #[test]
    fn announcement_title_keeps_composable_children_on_truncated_title_contract() {
        let window = AppWindowId::default();
        let mut app = App::new();
        let title = "Shadcn Extras landed in Fret with a very long title";

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds_240x120(), "test", |cx| {
                AnnouncementTitle::new([cx.text(title)]).into_element(cx)
            });

        let ElementKind::Container(props) = &element.kind else {
            panic!("expected AnnouncementTitle root to be a Container");
        };
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.layout.overflow, Overflow::Clip);

        let theme = Theme::global(&app);
        let inherited = element
            .inherited_text_style
            .as_ref()
            .expect("expected AnnouncementTitle to scope title text style");
        let expected = fret_ui_kit::typography::composable_refinement_from_style(
            &fret_ui_kit::typography::control_text_style(
                theme,
                fret_ui_kit::typography::UiTextSize::Sm,
            ),
        );
        assert_eq!(inherited.size, expected.size);
        assert_eq!(inherited.line_height, expected.line_height);
        assert_eq!(inherited.weight, Some(FontWeight::MEDIUM));

        let text = find_text(&element, title).expect("expected nested composable title text");
        let ElementKind::Text(props) = &text.kind else {
            panic!("expected nested AnnouncementTitle child to stay a Text element");
        };
        assert!(props.style.is_none());
        assert_eq!(props.layout.flex.shrink, 1.0);
        assert_eq!(props.layout.size.min_width, Some(Length::Px(Px(0.0))));
        assert_eq!(props.wrap, TextWrap::None);
        assert_eq!(props.overflow, TextOverflow::Ellipsis);
    }
}
