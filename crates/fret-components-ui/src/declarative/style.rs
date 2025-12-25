use fret_core::{Color, Corners, Edges, Px};
use fret_ui::Theme;
use fret_ui::element::ContainerProps;

use crate::{ColorRef, MetricRef, Radius, Space, StyleRefinement};

pub fn space(theme: &Theme, space: Space) -> Px {
    MetricRef::space(space).resolve(theme)
}

pub fn radius(theme: &Theme, radius: Radius) -> Px {
    MetricRef::radius(radius).resolve(theme)
}

pub fn color(theme: &Theme, color: ColorRef) -> Color {
    color.resolve(theme)
}

pub fn container_props(theme: &Theme, refinement: StyleRefinement) -> ContainerProps {
    let padding_x = refinement
        .padding_x
        .as_ref()
        .map(|m| m.resolve(theme))
        .unwrap_or(Px(0.0));
    let padding_y = refinement
        .padding_y
        .as_ref()
        .map(|m| m.resolve(theme))
        .unwrap_or(Px(0.0));

    let background = refinement.background.as_ref().map(|c| c.resolve(theme));

    let border_width = refinement
        .border_width
        .as_ref()
        .map(|m| m.resolve(theme))
        .unwrap_or(Px(0.0));
    let border_color = refinement.border_color.as_ref().map(|c| c.resolve(theme));

    let radius = refinement
        .radius
        .as_ref()
        .map(|m| m.resolve(theme))
        .unwrap_or(Px(0.0));

    ContainerProps {
        layout: Default::default(),
        padding_x,
        padding_y,
        background,
        border: Edges::all(border_width),
        border_color,
        corner_radii: Corners::all(radius),
    }
}
