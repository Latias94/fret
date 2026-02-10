use fret_core::Px;
use fret_ui::element::{AnyElement, ContainerProps, Length, SizeStyle};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::LayoutRefinement;
use crate::declarative::style as decl_style;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SeparatorOrientation {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone)]
pub struct Separator {
    orientation: SeparatorOrientation,
    thickness: Option<Px>,
    flex_stretch_cross_axis: bool,
    layout: LayoutRefinement,
}

impl Separator {
    pub fn new() -> Self {
        Self {
            orientation: SeparatorOrientation::default(),
            thickness: None,
            flex_stretch_cross_axis: false,
            layout: LayoutRefinement::default(),
        }
    }

    pub fn orientation(mut self, orientation: SeparatorOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn thickness(mut self, thickness: Px) -> Self {
        self.thickness = Some(thickness);
        self
    }

    pub fn flex_stretch_cross_axis(mut self, stretch: bool) -> Self {
        self.flex_stretch_cross_axis = stretch;
        self
    }

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let (border, thickness, mut layout) = {
            let theme = Theme::global(&*cx.app);

            let border = theme
                .color_by_key("border")
                .unwrap_or_else(|| theme.color_required("border"));
            let thickness = self.thickness.unwrap_or_else(|| {
                theme
                    .metric_by_key("component.separator.px")
                    .unwrap_or(Px(1.0))
            });
            let layout = decl_style::layout_style(theme, self.layout);

            (border, thickness, layout)
        };
        match self.orientation {
            SeparatorOrientation::Horizontal => {
                layout.size = SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(thickness),
                    min_height: Some(thickness),
                    max_height: Some(thickness),
                    ..layout.size
                };
            }
            SeparatorOrientation::Vertical => {
                layout.size = SizeStyle {
                    width: Length::Px(thickness),
                    // In shadcn/radix recipes the vertical separator is typically `self-stretch`
                    // inside a flex row. Using `Fill` maps to `height: 100%`, which does not
                    // resolve in an auto-height containing block. Keeping the height `Auto`
                    // allows `align-items: stretch` to produce the desired outcome.
                    height: if self.flex_stretch_cross_axis {
                        Length::Auto
                    } else {
                        Length::Fill
                    },
                    min_width: Some(thickness),
                    max_width: Some(thickness),
                    ..layout.size
                };
            }
        }

        cx.container(
            ContainerProps {
                layout,
                background: Some(border),
                ..Default::default()
            },
            |_cx| Vec::new(),
        )
    }
}

#[track_caller]
pub fn separator<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    Separator::new().into_element(cx)
}
