use fret_ui::element::{AnyElement, ColumnProps, CrossAlign, LayoutStyle, MainAlign, RowProps};
use fret_ui::{ElementCx, Theme, UiHost};

use crate::Space;
use crate::declarative::style;

#[derive(Debug, Clone, Copy)]
pub struct HStackProps {
    pub layout: LayoutStyle,
    pub gap: Space,
    pub justify: MainAlign,
    pub align: CrossAlign,
}

impl Default for HStackProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
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

    pub fn layout(mut self, layout: LayoutStyle) -> Self {
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
    cx.row(
        RowProps {
            layout: props.layout,
            gap,
            justify: props.justify,
            align: props.align,
            ..Default::default()
        },
        f,
    )
}

#[derive(Debug, Clone, Copy)]
pub struct VStackProps {
    pub layout: LayoutStyle,
    pub gap: Space,
    pub justify: MainAlign,
    pub align: CrossAlign,
}

impl Default for VStackProps {
    fn default() -> Self {
        Self {
            layout: LayoutStyle::default(),
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

    pub fn layout(mut self, layout: LayoutStyle) -> Self {
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
    cx.column(
        ColumnProps {
            layout: props.layout,
            gap,
            justify: props.justify,
            align: props.align,
            ..Default::default()
        },
        f,
    )
}
