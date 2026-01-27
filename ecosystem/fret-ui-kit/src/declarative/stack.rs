use fret_ui::element::{AnyElement, ColumnProps, RowProps};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::declarative::style;
use crate::{Items, Justify, LayoutRefinement, Space};

#[derive(Debug, Clone)]
pub struct HStackProps {
    pub layout: LayoutRefinement,
    pub gap: Space,
    pub justify: Justify,
    pub items: Items,
}

impl Default for HStackProps {
    fn default() -> Self {
        Self {
            layout: LayoutRefinement::default(),
            gap: Space::N0,
            justify: Justify::Start,
            items: Items::Center,
        }
    }
}

impl HStackProps {
    pub fn gap(mut self, space: Space) -> Self {
        self.gap = space;
        self
    }

    pub fn gap_x(self, space: Space) -> Self {
        self.gap(space)
    }

    pub fn layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = layout;
        self
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    pub fn items(mut self, items: Items) -> Self {
        self.items = items;
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

    pub fn items_start(self) -> Self {
        self.items(Items::Start)
    }

    pub fn items_center(self) -> Self {
        self.items(Items::Center)
    }

    pub fn items_end(self) -> Self {
        self.items(Items::End)
    }

    pub fn items_stretch(self) -> Self {
        self.items(Items::Stretch)
    }
}

/// Component-layer "hstack" helper.
///
/// This exists to express Tailwind-like `gap-*` ergonomically without reaching into runtime props.
pub fn hstack<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: HStackProps,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let theme = Theme::global(&*cx.app);
    let gap = style::space(theme, props.gap);
    let layout = style::layout_style(theme, props.layout);
    cx.row(
        RowProps {
            layout,
            gap,
            justify: props.justify.to_main_align(),
            align: props.items.to_cross_align(),
            ..Default::default()
        },
        f,
    )
}

/// Variant of [`hstack`] that avoids iterator borrow pitfalls by collecting into a sink.
///
/// Use this when the natural authoring form is an iterator that captures `&mut cx` (e.g.
/// `items.iter().map(|it| cx.keyed(...))`), which cannot be returned directly.
pub fn hstack_build<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: HStackProps,
    build: impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
) -> AnyElement {
    hstack(cx, props, |cx| {
        let mut out = Vec::new();
        build(cx, &mut out);
        out
    })
}

#[derive(Debug, Clone)]
pub struct VStackProps {
    pub layout: LayoutRefinement,
    pub gap: Space,
    pub justify: Justify,
    pub items: Items,
}

impl Default for VStackProps {
    fn default() -> Self {
        Self {
            layout: LayoutRefinement::default(),
            gap: Space::N0,
            justify: Justify::Start,
            items: Items::Stretch,
        }
    }
}

impl VStackProps {
    pub fn gap(mut self, space: Space) -> Self {
        self.gap = space;
        self
    }

    pub fn gap_y(self, space: Space) -> Self {
        self.gap(space)
    }

    pub fn layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = layout;
        self
    }

    pub fn justify(mut self, justify: Justify) -> Self {
        self.justify = justify;
        self
    }

    pub fn items(mut self, items: Items) -> Self {
        self.items = items;
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

    pub fn items_start(self) -> Self {
        self.items(Items::Start)
    }

    pub fn items_center(self) -> Self {
        self.items(Items::Center)
    }

    pub fn items_end(self) -> Self {
        self.items(Items::End)
    }

    pub fn items_stretch(self) -> Self {
        self.items(Items::Stretch)
    }
}

/// Component-layer "vstack" helper.
///
/// This exists to express Tailwind-like `gap-*` ergonomically without reaching into runtime props.
pub fn vstack<H: UiHost, I>(
    cx: &mut ElementContext<'_, H>,
    props: VStackProps,
    f: impl FnOnce(&mut ElementContext<'_, H>) -> I,
) -> AnyElement
where
    I: IntoIterator<Item = AnyElement>,
{
    let theme = Theme::global(&*cx.app);
    let gap = style::space(theme, props.gap);
    let layout = style::layout_style(theme, props.layout);
    cx.column(
        ColumnProps {
            layout,
            gap,
            justify: props.justify.to_main_align(),
            align: props.items.to_cross_align(),
            ..Default::default()
        },
        f,
    )
}

/// Variant of [`vstack`] that avoids iterator borrow pitfalls by collecting into a sink.
///
/// Use this when the natural authoring form is an iterator that captures `&mut cx` (e.g.
/// `items.iter().map(|it| cx.keyed(...))`), which cannot be returned directly.
pub fn vstack_build<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    props: VStackProps,
    build: impl FnOnce(&mut ElementContext<'_, H>, &mut Vec<AnyElement>),
) -> AnyElement {
    vstack(cx, props, |cx| {
        let mut out = Vec::new();
        build(cx, &mut out);
        out
    })
}
