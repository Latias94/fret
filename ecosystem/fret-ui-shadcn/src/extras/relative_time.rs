use std::sync::Arc;

use fret_core::Px;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::stack;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius, Space, ui};

use crate::test_id::attach_test_id;

/// A small world-clock-like display block inspired by Kibo's "RelativeTime" shadcn block.
///
/// Notes:
/// - This is display-only in M1 (no timers / no scheduling).
/// - Callers provide already-formatted strings for date/time.
///
/// Upstream inspiration (MIT):
/// - `repo-ref/kibo/packages/relative-time`
#[derive(Debug, Clone)]
pub struct RelativeTime {
    children: Vec<AnyElement>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl RelativeTime {
    pub fn new(children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            children: children.into_iter().collect(),
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let children = self.children;
        let el = stack::vstack(
            cx,
            stack::VStackProps::default()
                .gap_y(Space::N2)
                .layout(self.layout),
            |_cx| children,
        );
        attach_test_id(
            el,
            self.test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.relative-time")),
        )
    }
}

#[derive(Debug, Clone)]
pub struct RelativeTimeZone {
    label: Arc<str>,
    date: Arc<str>,
    time: Arc<str>,
    test_id: Option<Arc<str>>,
    layout: LayoutRefinement,
}

impl RelativeTimeZone {
    pub fn new(
        label: impl Into<Arc<str>>,
        date: impl Into<Arc<str>>,
        time: impl Into<Arc<str>>,
    ) -> Self {
        Self {
            label: label.into(),
            date: date.into(),
            time: time.into(),
            test_id: None,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let left = stack::hstack(
            cx,
            stack::HStackProps::default()
                .items_center()
                .gap_x(Space::N1p5)
                .layout(LayoutRefinement::default().min_w_0()),
            |cx| {
                vec![
                    RelativeTimeZoneLabel::new(self.label.clone()).into_element(cx),
                    RelativeTimeZoneDate::new(self.date.clone()).into_element(cx),
                ]
            },
        );
        let right = RelativeTimeZoneDisplay::new(self.time)
            .muted(true)
            .into_element(cx);

        let el = stack::hstack(
            cx,
            stack::HStackProps::default()
                .justify_between()
                .items_center()
                .gap_x(Space::N1p5)
                .layout(LayoutRefinement::default().min_w_0().merge(self.layout)),
            |_cx| vec![left, right],
        );

        attach_test_id(
            el,
            self.test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.relative-time-zone")),
        )
    }
}

#[derive(Debug, Clone)]
pub struct RelativeTimeZoneLabel {
    label: Arc<str>,
    test_id: Option<Arc<str>>,
}

impl RelativeTimeZoneLabel {
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let bg = theme.color_required("secondary");
        let fg = theme.color_required("secondary-foreground");

        let chrome = ChromeRefinement::default()
            .px(Space::N1p5)
            .rounded(Radius::Sm)
            .bg(ColorRef::Color(bg))
            .text_color(ColorRef::Color(fg));
        let props =
            decl_style::container_props(&theme, chrome, LayoutRefinement::default().h_px(Px(16.0)));

        let label = self.label;
        let el = cx.container(props, move |cx| {
            let layout = decl_style::layout_style(
                &theme,
                LayoutRefinement::default().w_full().h_full().min_w_0(),
            );
            vec![cx.flex(
                fret_ui::element::FlexProps {
                    layout,
                    direction: fret_core::Axis::Horizontal,
                    gap: Px(0.0),
                    padding: fret_core::Edges::all(Px(0.0)),
                    justify: fret_ui::element::MainAlign::Center,
                    align: fret_ui::element::CrossAlign::Center,
                    wrap: false,
                },
                move |cx| vec![ui::text(cx, label.clone()).text_xs().into_element(cx)],
            )]
        });
        attach_test_id(
            el,
            self.test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.relative-time-zone-label")),
        )
    }
}

#[derive(Debug, Clone)]
pub struct RelativeTimeZoneDate {
    text: Arc<str>,
    test_id: Option<Arc<str>>,
}

impl RelativeTimeZoneDate {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            test_id: None,
        }
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let el = ui::text(cx, self.text).text_xs().into_element(cx);
        attach_test_id(
            el,
            self.test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.relative-time-zone-date")),
        )
    }
}

#[derive(Debug, Clone)]
pub struct RelativeTimeZoneDisplay {
    text: Arc<str>,
    muted: bool,
    test_id: Option<Arc<str>>,
}

impl RelativeTimeZoneDisplay {
    pub fn new(text: impl Into<Arc<str>>) -> Self {
        Self {
            text: text.into(),
            muted: true,
            test_id: None,
        }
    }

    pub fn muted(mut self, muted: bool) -> Self {
        self.muted = muted;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();
        let fg = if self.muted {
            theme.color_required("muted-foreground")
        } else {
            theme.color_required("foreground")
        };

        let chrome = ChromeRefinement::default().pl(Space::N8);
        let props = decl_style::container_props(&theme, chrome, LayoutRefinement::default());
        let text = self.text;

        let el = cx.container(props, move |cx| {
            vec![
                ui::text(cx, text.clone())
                    .text_xs()
                    .text_color(ColorRef::Color(fg))
                    .into_element(cx),
            ]
        });

        attach_test_id(
            el,
            self.test_id
                .unwrap_or_else(|| Arc::<str>::from("shadcn-extras.relative-time-zone-display")),
        )
    }
}
