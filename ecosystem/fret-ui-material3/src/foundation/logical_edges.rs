use fret_core::{Edges, LayoutDirection, Px};
use fret_ui::element::LayoutStyle;

pub(crate) fn horizontal_logical_edges(
    direction: LayoutDirection,
    inline_start: Px,
    inline_end: Px,
    block_start: Px,
    block_end: Px,
) -> Edges {
    let (left, right) = match direction {
        LayoutDirection::Ltr => (inline_start, inline_end),
        LayoutDirection::Rtl => (inline_end, inline_start),
    };

    Edges {
        top: block_start,
        right,
        bottom: block_end,
        left,
    }
}

pub(crate) fn set_inset_inline_end(
    layout: &mut LayoutStyle,
    direction: LayoutDirection,
    value: Px,
) {
    match direction {
        LayoutDirection::Ltr => layout.inset.right = Some(value).into(),
        LayoutDirection::Rtl => layout.inset.left = Some(value).into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use fret_ui::element::InsetEdge;

    #[test]
    fn logical_edges_map_inline_sides_to_physical_sides() {
        let edges =
            horizontal_logical_edges(LayoutDirection::Ltr, Px(1.0), Px(2.0), Px(3.0), Px(4.0));
        assert_eq!(edges.left, Px(1.0));
        assert_eq!(edges.right, Px(2.0));
        assert_eq!(edges.top, Px(3.0));
        assert_eq!(edges.bottom, Px(4.0));

        let edges =
            horizontal_logical_edges(LayoutDirection::Rtl, Px(1.0), Px(2.0), Px(3.0), Px(4.0));
        assert_eq!(edges.left, Px(2.0));
        assert_eq!(edges.right, Px(1.0));
        assert_eq!(edges.top, Px(3.0));
        assert_eq!(edges.bottom, Px(4.0));
    }

    #[test]
    fn inset_inline_end_maps_to_physical_edge() {
        let mut layout = LayoutStyle::default();
        set_inset_inline_end(&mut layout, LayoutDirection::Ltr, Px(-8.0));
        assert_eq!(layout.inset.right, InsetEdge::Px(Px(-8.0)));
        assert_eq!(layout.inset.left, InsetEdge::Auto);

        let mut layout = LayoutStyle::default();
        set_inset_inline_end(&mut layout, LayoutDirection::Rtl, Px(-8.0));
        assert_eq!(layout.inset.left, InsetEdge::Px(Px(-8.0)));
        assert_eq!(layout.inset.right, InsetEdge::Auto);
    }
}
