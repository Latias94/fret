use fret_icons::{IconId, ids};

use fret_core::{Edges, Px};
use fret_ui::element::InsetStyle;
use fret_ui_kit::LayoutRefinement;

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
}
