use fret_icons::{IconId, ids};

use fret_core::{Edges, Px};
use fret_ui::element::InsetStyle;
use fret_ui_kit::{LayoutRefinement, Space};

use crate::LayoutDirection;

#[inline]
pub(crate) fn chevrons_inline_start(dir: LayoutDirection) -> IconId {
    match dir {
        LayoutDirection::Ltr => IconId::new_static("lucide.chevrons-left"),
        LayoutDirection::Rtl => IconId::new_static("lucide.chevrons-right"),
    }
}

#[inline]
pub(crate) fn chevrons_inline_end(dir: LayoutDirection) -> IconId {
    match dir {
        LayoutDirection::Ltr => IconId::new_static("lucide.chevrons-right"),
        LayoutDirection::Rtl => IconId::new_static("lucide.chevrons-left"),
    }
}

/// Returns a chevron that points toward the inline-start edge for `dir`.
#[inline]
pub(crate) fn chevron_inline_start(dir: LayoutDirection) -> IconId {
    match dir {
        LayoutDirection::Ltr => ids::ui::CHEVRON_LEFT,
        LayoutDirection::Rtl => ids::ui::CHEVRON_RIGHT,
    }
}

/// Returns a chevron that points toward the inline-end edge for `dir`.
#[inline]
pub(crate) fn chevron_inline_end(dir: LayoutDirection) -> IconId {
    match dir {
        LayoutDirection::Ltr => ids::ui::CHEVRON_RIGHT,
        LayoutDirection::Rtl => ids::ui::CHEVRON_LEFT,
    }
}

/// Applies an inline-start `auto` margin to `layout`.
///
/// This is the logical equivalent of CSS `margin-inline-start: auto`, which is commonly used to
/// push a trailing widget to the inline-end edge of a horizontal row.
#[inline]
pub(crate) fn layout_margin_inline_start_auto(
    layout: &mut fret_ui::element::LayoutStyle,
    dir: LayoutDirection,
) {
    match dir {
        LayoutDirection::Ltr => layout.margin.left = fret_ui::element::MarginEdge::Auto,
        LayoutDirection::Rtl => layout.margin.right = fret_ui::element::MarginEdge::Auto,
    }
}

#[inline]
pub(crate) fn layout_refinement_margin_inline_start_auto(dir: LayoutDirection) -> LayoutRefinement {
    match dir {
        LayoutDirection::Ltr => LayoutRefinement::default().ml_auto(),
        LayoutDirection::Rtl => LayoutRefinement::default().mr_auto(),
    }
}

#[inline]
pub(crate) fn physical_inline_start_end(
    dir: LayoutDirection,
    inline_start: Px,
    inline_end: Px,
) -> (Px, Px) {
    match dir {
        LayoutDirection::Ltr => (inline_start, inline_end),
        LayoutDirection::Rtl => (inline_end, inline_start),
    }
}

/// When a style surface only supports a symmetric `padding_x`, pick a value that will not under-pad
/// any physical edge in the presence of asymmetric token resolution.
#[inline]
pub(crate) fn padding_x_from_physical_edges_max(left: Px, right: Px) -> Px {
    Px(left.0.max(right.0))
}

#[inline]
pub(crate) fn inline_start_end_pair<T>(
    dir: LayoutDirection,
    inline_start: T,
    inline_end: T,
) -> (T, T) {
    match dir {
        LayoutDirection::Ltr => (inline_start, inline_end),
        LayoutDirection::Rtl => (inline_end, inline_start),
    }
}

#[inline]
pub(crate) fn vec_main_with_inline_end<T>(
    dir: LayoutDirection,
    main: T,
    inline_end: Option<T>,
) -> Vec<T> {
    match dir {
        LayoutDirection::Ltr => {
            let mut out = vec![main];
            if let Some(inline_end) = inline_end {
                out.push(inline_end);
            }
            out
        }
        LayoutDirection::Rtl => {
            let mut out = Vec::new();
            if let Some(inline_end) = inline_end {
                out.push(inline_end);
            }
            out.push(main);
            out
        }
    }
}

#[inline]
pub(crate) fn vec_main_with_inline_start<T>(
    dir: LayoutDirection,
    main: T,
    inline_start: Option<T>,
) -> Vec<T> {
    match dir {
        LayoutDirection::Ltr => {
            let mut out = Vec::new();
            if let Some(inline_start) = inline_start {
                out.push(inline_start);
            }
            out.push(main);
            out
        }
        LayoutDirection::Rtl => {
            let mut out = vec![main];
            if let Some(inline_start) = inline_start {
                out.push(inline_start);
            }
            out
        }
    }
}

#[inline]
pub(crate) fn concat_inline_start_end<T>(
    dir: LayoutDirection,
    mut inline_start: Vec<T>,
    mut inline_end: Vec<T>,
) -> Vec<T> {
    match dir {
        LayoutDirection::Ltr => {
            inline_start.extend(inline_end);
            inline_start
        }
        LayoutDirection::Rtl => {
            inline_end.reverse();
            inline_end.extend(inline_start);
            inline_end
        }
    }
}

#[inline]
pub(crate) fn reverse_in_rtl<T>(dir: LayoutDirection, mut items: Vec<T>) -> Vec<T> {
    if dir == LayoutDirection::Rtl {
        items.reverse();
    }
    items
}

#[inline]
pub(crate) fn concat_main_with_inline_start_vec<T>(
    dir: LayoutDirection,
    main: T,
    mut inline_start: Vec<T>,
) -> Vec<T> {
    match dir {
        LayoutDirection::Ltr => {
            inline_start.push(main);
            inline_start
        }
        LayoutDirection::Rtl => {
            inline_start.reverse();
            let mut out = vec![main];
            out.extend(inline_start);
            out
        }
    }
}

#[inline]
pub(crate) fn padding_edges_with_inline_start_end(
    dir: LayoutDirection,
    pad_top: Px,
    pad_bottom: Px,
    pad_inline_start: Px,
    pad_inline_end: Px,
) -> Edges {
    let (left, right) = physical_inline_start_end(dir, pad_inline_start, pad_inline_end);
    Edges {
        top: pad_top,
        right,
        bottom: pad_bottom,
        left,
    }
}

#[inline]
pub(crate) fn inset_style_set_inline_start(inset: &mut InsetStyle, dir: LayoutDirection, px: Px) {
    match dir {
        LayoutDirection::Ltr => {
            inset.left = Some(px).into();
            inset.right = None.into();
        }
        LayoutDirection::Rtl => {
            inset.right = Some(px).into();
            inset.left = None.into();
        }
    }
}

#[inline]
pub(crate) fn layout_margin_inline_start_px(
    layout: &mut fret_ui::element::LayoutStyle,
    dir: LayoutDirection,
    px: Px,
) {
    match dir {
        LayoutDirection::Ltr => {
            layout.margin.left = fret_ui::element::MarginEdge::Px(px);
        }
        LayoutDirection::Rtl => {
            layout.margin.right = fret_ui::element::MarginEdge::Px(px);
        }
    }
}

#[inline]
pub(crate) fn layout_refinement_apply_margin_inline_start_neg(
    layout: LayoutRefinement,
    dir: LayoutDirection,
    space: Space,
) -> LayoutRefinement {
    match dir {
        LayoutDirection::Ltr => layout.ml_neg(space),
        LayoutDirection::Rtl => layout.mr_neg(space),
    }
}

#[inline]
pub(crate) fn layout_refinement_apply_margin_inline_end_neg(
    layout: LayoutRefinement,
    dir: LayoutDirection,
    space: Space,
) -> LayoutRefinement {
    match dir {
        LayoutDirection::Ltr => layout.mr_neg(space),
        LayoutDirection::Rtl => layout.ml_neg(space),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_start_end_chevrons_flip_in_rtl() {
        assert_eq!(
            chevrons_inline_start(LayoutDirection::Ltr),
            IconId::new_static("lucide.chevrons-left")
        );
        assert_eq!(
            chevrons_inline_start(LayoutDirection::Rtl),
            IconId::new_static("lucide.chevrons-right")
        );
        assert_eq!(
            chevrons_inline_end(LayoutDirection::Ltr),
            IconId::new_static("lucide.chevrons-right")
        );
        assert_eq!(
            chevrons_inline_end(LayoutDirection::Rtl),
            IconId::new_static("lucide.chevrons-left")
        );

        assert_eq!(
            chevron_inline_start(LayoutDirection::Ltr),
            ids::ui::CHEVRON_LEFT
        );
        assert_eq!(
            chevron_inline_start(LayoutDirection::Rtl),
            ids::ui::CHEVRON_RIGHT
        );

        assert_eq!(
            chevron_inline_end(LayoutDirection::Ltr),
            ids::ui::CHEVRON_RIGHT
        );
        assert_eq!(
            chevron_inline_end(LayoutDirection::Rtl),
            ids::ui::CHEVRON_LEFT
        );
    }

    #[test]
    fn inline_start_end_pair_swaps_in_rtl() {
        let (a, b) = inline_start_end_pair(LayoutDirection::Ltr, 1, 2);
        assert_eq!((a, b), (1, 2));

        let (a, b) = inline_start_end_pair(LayoutDirection::Rtl, 1, 2);
        assert_eq!((a, b), (2, 1));
    }

    #[test]
    fn padding_x_from_physical_edges_max_picks_larger_edge() {
        assert_eq!(
            padding_x_from_physical_edges_max(Px(10.0), Px(12.0)),
            Px(12.0)
        );
        assert_eq!(
            padding_x_from_physical_edges_max(Px(12.0), Px(10.0)),
            Px(12.0)
        );
    }

    #[test]
    fn vec_main_with_inline_start_places_widget_at_inline_start() {
        assert_eq!(
            vec_main_with_inline_start(LayoutDirection::Ltr, 1, Some(2)),
            vec![2, 1]
        );
        assert_eq!(
            vec_main_with_inline_start(LayoutDirection::Rtl, 1, Some(2)),
            vec![1, 2]
        );
    }

    #[test]
    fn vec_main_with_inline_end_places_widget_at_inline_end() {
        assert_eq!(
            vec_main_with_inline_end(LayoutDirection::Ltr, 1, Some(2)),
            vec![1, 2]
        );
        assert_eq!(
            vec_main_with_inline_end(LayoutDirection::Rtl, 1, Some(2)),
            vec![2, 1]
        );
    }

    #[test]
    fn concat_inline_start_end_reverses_inline_end_in_rtl() {
        assert_eq!(
            concat_inline_start_end(LayoutDirection::Ltr, vec![1, 2], vec![3, 4]),
            vec![1, 2, 3, 4]
        );
        assert_eq!(
            concat_inline_start_end(LayoutDirection::Rtl, vec![1, 2], vec![3, 4]),
            vec![4, 3, 1, 2]
        );
    }

    #[test]
    fn reverse_in_rtl_only_reverses_in_rtl() {
        assert_eq!(
            reverse_in_rtl(LayoutDirection::Ltr, vec![1, 2, 3]),
            vec![1, 2, 3]
        );
        assert_eq!(
            reverse_in_rtl(LayoutDirection::Rtl, vec![1, 2, 3]),
            vec![3, 2, 1]
        );
    }

    #[test]
    fn concat_main_with_inline_start_vec_places_vec_at_inline_start() {
        assert_eq!(
            concat_main_with_inline_start_vec(LayoutDirection::Ltr, 1, vec![2, 3]),
            vec![2, 3, 1]
        );
        assert_eq!(
            concat_main_with_inline_start_vec(LayoutDirection::Rtl, 1, vec![2, 3]),
            vec![1, 3, 2]
        );
    }
}
