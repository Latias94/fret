use fret_ui::element::{AnyElement, Overflow};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, LayoutRefinement};

/// shadcn/ui `AspectRatio`.
///
/// This is a small declarative helper that applies an aspect ratio constraint to its child.
///
/// Note: Fret’s aspect ratio contract (ADR 0057) only derives the missing dimension when exactly
/// one axis is `Auto`. By default, `AspectRatio` sets `width=Fill` and keeps `height=Auto`.
#[derive(Debug, Clone)]
pub struct AspectRatio {
    ratio: f32,
    child: AnyElement,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
}

impl AspectRatio {
    pub fn new(ratio: f32, child: AnyElement) -> Self {
        Self {
            ratio,
            child,
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let base_layout = LayoutRefinement::default().w_full();
        let mut props = decl_style::container_props(
            &theme,
            self.chrome,
            base_layout.aspect_ratio(self.ratio).merge(self.layout),
        );
        props.layout.overflow = Overflow::Clip;

        let child = self.child;
        cx.container(props, move |_cx| vec![child])
    }
}
