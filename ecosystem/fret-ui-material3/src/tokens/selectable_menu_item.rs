//! Shared Material selectable menu item token outcomes.
//!
//! Select, Autocomplete, and ExposedDropdown all render Material selectable menu rows. Component
//! token modules keep their variant-specific normal colors, while this module owns the shared
//! selected/disabled content outcomes and density constants.

use fret_core::{Color, Corners, Px};
use fret_ui::Theme;

use crate::foundation::token_resolver::MaterialTokenResolver;
use crate::tokens::list as list_tokens;

pub(crate) fn outer_horizontal_padding(_theme: &Theme) -> Px {
    Px(4.0)
}

pub(crate) fn outer_vertical_padding(_theme: &Theme, has_secondary_text: bool) -> Px {
    if has_secondary_text { Px(2.0) } else { Px(0.0) }
}

pub(crate) fn content_horizontal_padding(_theme: &Theme) -> Px {
    Px(12.0)
}

pub(crate) fn icon_text_gap(_theme: &Theme) -> Px {
    Px(8.0)
}

pub(crate) fn container_shape(theme: &Theme, selected: bool) -> Corners {
    let tokens = MaterialTokenResolver::new(theme);
    if selected {
        tokens.corners_chain_or(
            &[
                "md.comp.menu.list-item.selected.container.shape",
                "md.sys.shape.corner.medium",
            ],
            Corners::all(Px(12.0)),
        )
    } else {
        tokens.corners_chain_or(
            &[
                "md.comp.menu.list-item.container.shape",
                "md.sys.shape.corner.extra-small",
            ],
            Corners::all(Px(4.0)),
        )
    }
}

pub(crate) fn selected_or_disabled_label_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
) -> Option<Color> {
    (selected || !enabled).then(|| {
        let (label, _, _, _) = list_tokens::item_outcomes(
            theme,
            selected,
            enabled,
            list_tokens::ListItemInteraction::Default,
        );
        label
    })
}

pub(crate) fn selected_or_disabled_icon_color(
    theme: &Theme,
    selected: bool,
    enabled: bool,
) -> Option<Color> {
    (selected || !enabled).then(|| {
        let (_, icon, _, _) = list_tokens::item_outcomes(
            theme,
            selected,
            enabled,
            list_tokens::ListItemInteraction::Default,
        );
        icon
    })
}
