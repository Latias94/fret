use std::sync::Arc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorFallback, ColorRef, LayoutRefinement, Radius, Space, ui};

use crate::test_id::attach_test_id;

/// A small "announcement chip" block inspired by Kibo's shadcn blocks.
///
/// Upstream inspiration (MIT):
/// - `repo-ref/kibo/packages/announcement`
#[derive(Debug, Clone)]
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
        let theme = Theme::global(&*cx.app).clone();

        let border = theme.color_required("border");
        let bg = theme.color_required("background");
        let accent = theme.color_required("accent");

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
            vec![stack::hstack(
                cx,
                stack::HStackProps::default()
                    .items_center()
                    .gap_x(Space::N2),
                |_cx| children,
            )]
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
        let theme = Theme::global(&*cx.app).clone();
        let mut bg = theme.color_required("foreground");
        bg.a = (bg.a * 0.05).clamp(0.0, 1.0);
        let chrome = ChromeRefinement::default()
            .px(Space::N2p5)
            .py(Space::N1)
            .rounded(Radius::Full)
            .bg(ColorRef::Color(bg));

        let props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());
        let el = cx.container(props, |cx| {
            vec![ui::text(cx, self.label).text_xs().into_element(cx)]
        });
        attach_test_id(
            el,
            self.test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.announcement-tag")),
        )
    }
}

#[derive(Debug, Clone)]
pub struct AnnouncementTitle {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
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
        let theme = Theme::global(&*cx.app).clone();
        let props = decl_style::container_props(
            &theme,
            ChromeRefinement::default().py(Space::N1),
            LayoutRefinement::default(),
        );

        let test_id = self.test_id.clone();
        let children = self.children;

        let el = cx.container(props, move |cx| {
            vec![stack::hstack(
                cx,
                stack::HStackProps::default()
                    .items_center()
                    .gap_x(Space::N1),
                |_cx| children,
            )]
        });

        attach_test_id(
            el,
            test_id.unwrap_or_else(|| Arc::<str>::from("shadcn-extras.announcement-title")),
        )
    }
}
