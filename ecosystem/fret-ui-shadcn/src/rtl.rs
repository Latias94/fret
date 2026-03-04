use fret_icons::{IconId, ids};

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
