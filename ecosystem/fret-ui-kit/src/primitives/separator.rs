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
    layout: LayoutRefinement,
}

impl Separator {
    pub fn new() -> Self {
        Self {
            orientation: SeparatorOrientation::default(),
            thickness: None,
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

    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
        self
    }

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let theme = Theme::global(&*cx.app).clone();

        let border = theme
            .color_by_key("border")
            .unwrap_or_else(|| theme.color_required("border"));
        let thickness = self.thickness.unwrap_or_else(|| {
            theme
                .metric_by_key("component.separator.px")
                .unwrap_or(Px(1.0))
        });

        let mut layout = decl_style::layout_style(&theme, self.layout);
        match self.orientation {
            SeparatorOrientation::Horizontal => {
                layout.size = SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(thickness),
                    ..layout.size
                };
            }
            SeparatorOrientation::Vertical => {
                layout.size = SizeStyle {
                    width: Length::Px(thickness),
                    height: Length::Fill,
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

pub fn separator<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    Separator::new().into_element(cx)
}
