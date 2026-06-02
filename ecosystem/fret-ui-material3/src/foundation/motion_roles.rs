//! Semantic Material motion roles.
//!
//! Recipes should usually ask for a role such as "overlay scale" or "field chrome" instead of
//! choosing raw spring keys. The raw `MotionSchemeKey` mapping stays local to this Module.

use fret_ui::elements::ElementContext;
use fret_ui::{Theme, UiHost};

use crate::foundation::motion_scheme::{MotionSchemeKey, sys_spring_in_scope};
use crate::motion::SpringSpec;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MaterialMotionRole {
    ButtonPressedShape,
    IconButtonPressedShape,
    CheckboxMark,
    RadioDot,
    DropdownChevron,
    SelectionIndicator,
    FieldChrome,
    FieldFastEffects,
    FieldSlowEffects,
    OverlayScale,
    OverlayOpacity,
    SearchDockedExpand,
    SearchDockedCollapse,
    SearchFullScreenExpand,
    SearchFullScreenCollapse,
    SearchContentFadeIn,
    SearchContentFadeOut,
    ModalPanelSpatial,
    ModalPanelEffects,
}

impl MaterialMotionRole {
    pub(crate) fn scheme_key(self) -> MotionSchemeKey {
        match self {
            Self::ButtonPressedShape => MotionSchemeKey::DefaultEffects,
            Self::IconButtonPressedShape
            | Self::DropdownChevron
            | Self::RadioDot
            | Self::SelectionIndicator
            | Self::FieldChrome
            | Self::OverlayScale
            | Self::SearchDockedCollapse => MotionSchemeKey::FastSpatial,
            Self::CheckboxMark
            | Self::SearchDockedExpand
            | Self::SearchFullScreenCollapse
            | Self::ModalPanelSpatial => MotionSchemeKey::DefaultSpatial,
            Self::SearchFullScreenExpand => MotionSchemeKey::SlowSpatial,
            Self::FieldFastEffects
            | Self::OverlayOpacity
            | Self::SearchContentFadeIn
            | Self::SearchContentFadeOut => MotionSchemeKey::FastEffects,
            Self::FieldSlowEffects => MotionSchemeKey::SlowEffects,
            Self::ModalPanelEffects => MotionSchemeKey::DefaultEffects,
        }
    }
}

pub(crate) fn material_motion_spring_in_scope<H: UiHost>(
    cx: &ElementContext<'_, H>,
    theme: &Theme,
    role: MaterialMotionRole,
) -> SpringSpec {
    sys_spring_in_scope(cx, theme, role.scheme_key())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_motion_roles_preserve_existing_scheme_key_mapping() {
        let cases = [
            (
                MaterialMotionRole::ButtonPressedShape,
                MotionSchemeKey::DefaultEffects,
            ),
            (
                MaterialMotionRole::IconButtonPressedShape,
                MotionSchemeKey::FastSpatial,
            ),
            (
                MaterialMotionRole::CheckboxMark,
                MotionSchemeKey::DefaultSpatial,
            ),
            (MaterialMotionRole::RadioDot, MotionSchemeKey::FastSpatial),
            (
                MaterialMotionRole::DropdownChevron,
                MotionSchemeKey::FastSpatial,
            ),
            (
                MaterialMotionRole::SelectionIndicator,
                MotionSchemeKey::FastSpatial,
            ),
            (
                MaterialMotionRole::FieldChrome,
                MotionSchemeKey::FastSpatial,
            ),
            (
                MaterialMotionRole::FieldFastEffects,
                MotionSchemeKey::FastEffects,
            ),
            (
                MaterialMotionRole::FieldSlowEffects,
                MotionSchemeKey::SlowEffects,
            ),
            (
                MaterialMotionRole::OverlayScale,
                MotionSchemeKey::FastSpatial,
            ),
            (
                MaterialMotionRole::OverlayOpacity,
                MotionSchemeKey::FastEffects,
            ),
            (
                MaterialMotionRole::SearchDockedExpand,
                MotionSchemeKey::DefaultSpatial,
            ),
            (
                MaterialMotionRole::SearchDockedCollapse,
                MotionSchemeKey::FastSpatial,
            ),
            (
                MaterialMotionRole::SearchFullScreenExpand,
                MotionSchemeKey::SlowSpatial,
            ),
            (
                MaterialMotionRole::SearchFullScreenCollapse,
                MotionSchemeKey::DefaultSpatial,
            ),
            (
                MaterialMotionRole::SearchContentFadeIn,
                MotionSchemeKey::FastEffects,
            ),
            (
                MaterialMotionRole::SearchContentFadeOut,
                MotionSchemeKey::FastEffects,
            ),
            (
                MaterialMotionRole::ModalPanelSpatial,
                MotionSchemeKey::DefaultSpatial,
            ),
            (
                MaterialMotionRole::ModalPanelEffects,
                MotionSchemeKey::DefaultEffects,
            ),
        ];

        for (role, key) in cases {
            assert_eq!(role.scheme_key(), key, "{role:?}");
        }
    }
}
