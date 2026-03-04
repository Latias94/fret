use fret_icons::{IconId, ids};

use crate::LayoutDirection;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_start_end_chevrons_flip_in_rtl() {
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
