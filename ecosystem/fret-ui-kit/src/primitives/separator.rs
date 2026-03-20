use fret_core::{Px, SemanticsOrientation, SemanticsRole};
use fret_ui::element::{AnyElement, ContainerProps, Length, SemanticsDecoration, SizeStyle};
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
    decorative: bool,
    layout: LayoutRefinement,
}

impl Separator {
    pub fn new() -> Self {
        Self {
            orientation: SeparatorOrientation::default(),
            thickness: None,
            flex_stretch_cross_axis: false,
            decorative: false,
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

    pub fn decorative(mut self, decorative: bool) -> Self {
        self.decorative = decorative;
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
                .unwrap_or_else(|| theme.color_token("border"));
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
                    min_height: Some(Length::Px(thickness)),
                    max_height: Some(Length::Px(thickness)),
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
                    min_width: Some(Length::Px(thickness)),
                    max_width: Some(Length::Px(thickness)),
                    ..layout.size
                };
            }
        }

        let mut element = cx.container(
            ContainerProps {
                layout,
                background: Some(border),
                ..Default::default()
            },
            |_cx| Vec::new(),
        );

        let decoration = if self.decorative {
            SemanticsDecoration::default().hidden(true)
        } else {
            let mut semantics = SemanticsDecoration::default().role(SemanticsRole::Separator);
            if self.orientation == SeparatorOrientation::Vertical {
                semantics = semantics.orientation(SemanticsOrientation::Vertical);
            }
            semantics
        };
        element = element.attach_semantics(decoration);

        element
    }
}

#[track_caller]
pub fn separator<H: UiHost>(cx: &mut ElementContext<'_, H>) -> AnyElement {
    Separator::new().into_element(cx)
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_app::App;
    use fret_core::{AppWindowId, Point, Rect, Size};
    use fret_ui::element::{ElementKind, Length};

    fn bounds_200x100() -> Rect {
        Rect::new(
            Point::new(Px(0.0), Px(0.0)),
            Size::new(Px(200.0), Px(100.0)),
        )
    }

    #[test]
    fn separator_defaults_to_semantic_horizontal_rule() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds_200x100(), "test", |cx| {
                Separator::new().into_element(cx)
            });

        let ElementKind::Container(props) = &element.kind else {
            panic!("expected Separator to render a container");
        };
        assert_eq!(props.layout.size.width, Length::Fill);
        assert_eq!(props.layout.size.height, Length::Px(Px(1.0)));

        let decoration = element
            .semantics_decoration
            .as_ref()
            .expect("expected separator semantics decoration");
        assert_eq!(decoration.role, Some(SemanticsRole::Separator));
        assert_eq!(decoration.hidden, None);
        assert_eq!(decoration.orientation, None);
    }

    #[test]
    fn decorative_separator_hides_from_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds_200x100(), "test", |cx| {
                Separator::new().decorative(true).into_element(cx)
            });

        let decoration = element
            .semantics_decoration
            .as_ref()
            .expect("expected decorative separator semantics decoration");
        assert_eq!(decoration.hidden, Some(true));
        assert_eq!(decoration.role, None);
        assert_eq!(decoration.orientation, None);
    }

    #[test]
    fn vertical_separator_can_expose_vertical_semantics() {
        let window = AppWindowId::default();
        let mut app = App::new();

        let element =
            fret_ui::elements::with_element_cx(&mut app, window, bounds_200x100(), "test", |cx| {
                Separator::new()
                    .orientation(SeparatorOrientation::Vertical)
                    .flex_stretch_cross_axis(true)
                    .into_element(cx)
            });

        let ElementKind::Container(props) = &element.kind else {
            panic!("expected vertical Separator to render a container");
        };
        assert_eq!(props.layout.size.width, Length::Px(Px(1.0)));
        assert_eq!(props.layout.size.height, Length::Auto);

        let decoration = element
            .semantics_decoration
            .as_ref()
            .expect("expected vertical separator semantics decoration");
        assert_eq!(decoration.role, Some(SemanticsRole::Separator));
        assert_eq!(decoration.orientation, Some(SemanticsOrientation::Vertical));
        assert_eq!(decoration.hidden, None);
    }
}
