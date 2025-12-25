use fret_ui::element::{AnyElement, ColumnProps, CrossAlign, MainAlign, RowProps};
use fret_ui::{ElementCx, Theme, UiHost};

use crate::declarative::style;
use crate::{LayoutRefinement, Space};

#[derive(Debug, Clone)]
pub struct HStackProps {
    pub layout: LayoutRefinement,
    pub gap: Space,
    pub justify: MainAlign,
    pub align: CrossAlign,
}

impl Default for HStackProps {
    fn default() -> Self {
        Self {
            layout: LayoutRefinement::default(),
            gap: Space::N0,
            justify: MainAlign::Start,
            align: CrossAlign::Center,
        }
    }
}

impl HStackProps {
    pub fn gap(mut self, space: Space) -> Self {
        self.gap = space;
        self
    }

    pub fn layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = layout;
        self
    }

    pub fn justify(mut self, justify: MainAlign) -> Self {
        self.justify = justify;
        self
    }

    pub fn align(mut self, align: CrossAlign) -> Self {
        self.align = align;
        self
    }
}

/// Component-layer "hstack" helper.
///
/// This exists to express Tailwind-like `gap-*` ergonomically without reaching into runtime props.
pub fn hstack<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    props: HStackProps,
    f: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let gap = style::space(theme, props.gap);
    let layout = style::layout_style(theme, props.layout);
    cx.row(
        RowProps {
            layout,
            gap,
            justify: props.justify,
            align: props.align,
            ..Default::default()
        },
        f,
    )
}

#[derive(Debug, Clone)]
pub struct VStackProps {
    pub layout: LayoutRefinement,
    pub gap: Space,
    pub justify: MainAlign,
    pub align: CrossAlign,
}

impl Default for VStackProps {
    fn default() -> Self {
        Self {
            layout: LayoutRefinement::default(),
            gap: Space::N0,
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
        }
    }
}

impl VStackProps {
    pub fn gap(mut self, space: Space) -> Self {
        self.gap = space;
        self
    }

    pub fn layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = layout;
        self
    }

    pub fn justify(mut self, justify: MainAlign) -> Self {
        self.justify = justify;
        self
    }

    pub fn align(mut self, align: CrossAlign) -> Self {
        self.align = align;
        self
    }
}

/// Component-layer "vstack" helper.
///
/// This exists to express Tailwind-like `gap-*` ergonomically without reaching into runtime props.
pub fn vstack<H: UiHost>(
    cx: &mut ElementCx<'_, H>,
    props: VStackProps,
    f: impl FnOnce(&mut ElementCx<'_, H>) -> Vec<AnyElement>,
) -> AnyElement {
    let theme = Theme::global(&*cx.app);
    let gap = style::space(theme, props.gap);
    let layout = style::layout_style(theme, props.layout);
    cx.column(
        ColumnProps {
            layout,
            gap,
            justify: props.justify,
            align: props.align,
            ..Default::default()
        },
        f,
    )
}
