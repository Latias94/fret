use crate::profiles::{DEFAULT_EXPECTED_FAMILIES, DEFAULT_PROVIDED_ROLES};
use crate::*;
use read_fonts::tables::name::NameId;
use read_fonts::{FontRef, TableProvider as _};
use std::collections::BTreeSet;

fn dedupe_families(values: impl IntoIterator<Item = &'static str>) -> Vec<&'static str> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for family in values {
        if seen.insert(family) {
            out.push(family);
        }
    }
    out
}

fn profile_face_families(profile: &'static BundledFontProfile) -> Vec<&'static str> {
    dedupe_families(profile.faces.iter().map(|face| face.family))
}

fn families_for_role(
    profile: &'static BundledFontProfile,
    role: BundledFontRole,
) -> Vec<&'static str> {
    dedupe_families(
        profile
            .faces
            .iter()
            .filter(|face| face.roles.contains(&role))
            .map(|face| face.family),
    )
}

fn faces_for_family_and_role(
    profile: &'static BundledFontProfile,
    family: &'static str,
    role: BundledFontRole,
) -> Vec<&'static BundledFontFaceSpec> {
    profile
        .faces
        .iter()
        .filter(|face| face.family == family && face.roles.contains(&role))
        .collect()
}

fn expected_provided_roles(profile: &'static BundledFontProfile) -> Vec<BundledFontRole> {
    [
        BundledFontRole::UiSans,
        BundledFontRole::UiSerif,
        BundledFontRole::UiMonospace,
        BundledFontRole::EmojiFallback,
        BundledFontRole::CjkFallback,
    ]
    .into_iter()
    .filter(|role| profile.supports_role(*role))
    .collect()
}

fn expected_guaranteed_generic_families(
    profile: &'static BundledFontProfile,
) -> Vec<BundledGenericFamily> {
    [
        BundledGenericFamily::Sans,
        BundledGenericFamily::Serif,
        BundledGenericFamily::Monospace,
    ]
    .into_iter()
    .filter(|family| match family {
        BundledGenericFamily::Sans => !profile.ui_sans_families.is_empty(),
        BundledGenericFamily::Serif => !profile.ui_serif_families.is_empty(),
        BundledGenericFamily::Monospace => !profile.ui_mono_families.is_empty(),
    })
    .collect()
}

fn expected_common_fallback_families(profile: &'static BundledFontProfile) -> Vec<&'static str> {
    let mut out = families_for_role(profile, BundledFontRole::CjkFallback);
    out.extend(families_for_role(profile, BundledFontRole::EmojiFallback));
    out
}

fn decoded_family_names(bytes: &'static [u8]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for font in FontRef::fonts(bytes) {
        let font = font.expect("expected bundled font bytes to parse");
        let name = font
            .name()
            .expect("expected bundled font to have a name table");
        let strings = name.string_data();
        for record in name.name_record() {
            if record.name_id() != NameId::new(1) && record.name_id() != NameId::new(16) {
                continue;
            }
            let value = record
                .string(strings)
                .expect("expected name record to decode")
                .to_string();
            let value = value.trim();
            if !value.is_empty() {
                out.insert(value.to_string());
            }
        }
    }
    out
}

fn face_covers_codepoint(face: &'static BundledFontFaceSpec, ch: char) -> bool {
    FontRef::fonts(face.bytes).any(|font| {
        let font = font.expect("expected bundled font bytes to parse");
        font.cmap()
            .expect("expected bundled font to have a cmap")
            .map_codepoint(ch)
            .is_some_and(|glyph_id| glyph_id.to_u32() != 0)
    })
}

fn assert_profile_manifest_consistency(profile: &'static BundledFontProfile) {
    let face_families = profile_face_families(profile);
    let bytes_from_profile = profile.font_bytes().collect::<Vec<_>>();

    assert_eq!(
        profile.expected_family_names,
        face_families.as_slice(),
        "expected family manifest to match the bundled faces for profile {}",
        profile.name
    );
    assert_eq!(
        profile.provided_roles,
        expected_provided_roles(profile).as_slice(),
        "expected provided roles to match the face-role union for profile {}",
        profile.name
    );
    assert_eq!(
        profile.guaranteed_generic_families,
        expected_guaranteed_generic_families(profile).as_slice(),
        "expected guaranteed generic families to follow the declared UI family slots for profile {}",
        profile.name
    );
    assert_eq!(
        profile.ui_sans_families,
        families_for_role(profile, BundledFontRole::UiSans).as_slice(),
        "expected ui_sans_families to match the UiSans role families for profile {}",
        profile.name
    );
    assert_eq!(
        profile.ui_serif_families,
        families_for_role(profile, BundledFontRole::UiSerif).as_slice(),
        "expected ui_serif_families to match the UiSerif role families for profile {}",
        profile.name
    );
    assert_eq!(
        profile.ui_mono_families,
        families_for_role(profile, BundledFontRole::UiMonospace).as_slice(),
        "expected ui_mono_families to match the UiMonospace role families for profile {}",
        profile.name
    );
    assert_eq!(
        profile.common_fallback_families,
        expected_common_fallback_families(profile).as_slice(),
        "expected common fallback families to follow the CJK-then-emoji role order for profile {}",
        profile.name
    );
    assert_eq!(
        bytes_from_profile.as_slice(),
        profile
            .faces
            .iter()
            .map(|face| face.bytes)
            .collect::<Vec<_>>(),
        "expected font_bytes() to preserve face ordering for profile {}",
        profile.name
    );
}

fn assert_role_families_cover_codepoints(
    profile: &'static BundledFontProfile,
    role: BundledFontRole,
    samples: &[char],
) {
    for family in families_for_role(profile, role) {
        let faces = faces_for_family_and_role(profile, family, role);
        assert!(
            !faces.is_empty(),
            "expected at least one face for role {:?} family {} in profile {}",
            role,
            family,
            profile.name
        );
        for ch in samples {
            assert!(
                faces.iter().any(|face| face_covers_codepoint(face, *ch)),
                "expected role {:?} family {} in profile {} to cover {:?}",
                role,
                family,
                profile.name,
                ch
            );
        }
    }
}

#[test]
fn default_fonts_are_non_empty() {
    for font in crate::default_fonts() {
        assert!(font.len() > 1024);
    }
}

#[test]
fn default_profile_matches_default_fonts() {
    let profile = crate::default_profile();
    let bytes_from_profile = profile.font_bytes().collect::<Vec<_>>();
    assert_eq!(crate::default_fonts(), bytes_from_profile.as_slice());
    assert_eq!(profile.faces.len(), crate::default_fonts().len());
    assert_eq!(profile.expected_family_names, DEFAULT_EXPECTED_FAMILIES);
    assert_eq!(profile.provided_roles, DEFAULT_PROVIDED_ROLES);
}

#[test]
fn bundled_profiles_are_manifest_consistent() {
    assert_profile_manifest_consistency(crate::bootstrap_profile());
    assert_profile_manifest_consistency(crate::default_profile());
}

#[test]
fn bundled_face_family_names_match_name_tables() {
    for profile in [crate::bootstrap_profile(), crate::default_profile()] {
        for face in profile.faces {
            let family_names = decoded_family_names(face.bytes);
            assert!(
                family_names.contains(face.family),
                "expected bundled face family {} to match decoded name-table families {:?} in profile {}",
                face.family,
                family_names,
                profile.name
            );
        }
    }
}

#[test]
fn bundled_profile_matrix_covers_ui_and_monospace_contracts() {
    for profile in [crate::bootstrap_profile(), crate::default_profile()] {
        assert_role_families_cover_codepoints(profile, BundledFontRole::UiSans, &['A', 'm']);
        assert_role_families_cover_codepoints(profile, BundledFontRole::UiMonospace, &['0', 'm']);
    }
}

#[test]
fn bootstrap_profile_declares_expected_generic_guarantees() {
    let profile = crate::bootstrap_profile();
    assert!(profile.guarantees_generic_family(BundledGenericFamily::Monospace));
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    assert!(profile.guarantees_generic_family(BundledGenericFamily::Sans));
    #[cfg(not(any(feature = "bootstrap-subset", feature = "bootstrap-full")))]
    assert!(!profile.guarantees_generic_family(BundledGenericFamily::Sans));
    assert!(!profile.guarantees_generic_family(BundledGenericFamily::Serif));
}

#[test]
fn default_profile_explicitly_does_not_guarantee_serif() {
    let profile = crate::default_profile();
    assert!(profile.ui_serif_families.is_empty());
    assert!(!profile.guarantees_generic_family(BundledGenericFamily::Serif));
}

#[test]
fn bundled_profile_matrix_explicitly_omits_serif_contract() {
    for profile in [crate::bootstrap_profile(), crate::default_profile()] {
        assert!(
            profile.ui_serif_families.is_empty(),
            "expected no bundled serif families in profile {}",
            profile.name
        );
        assert!(
            !profile.guarantees_generic_family(BundledGenericFamily::Serif),
            "expected profile {} to explicitly avoid guaranteeing serif",
            profile.name
        );
    }
}

#[cfg(feature = "emoji")]
#[test]
fn default_profile_declares_emoji_role_when_enabled() {
    let profile = crate::default_profile();
    assert!(profile.supports_role(BundledFontRole::EmojiFallback));
    assert!(
        profile
            .common_fallback_families
            .contains(&"Noto Color Emoji")
    );
}

#[cfg(feature = "emoji")]
#[test]
fn bundled_profile_matrix_covers_emoji_fallback_contract() {
    let profile = crate::default_profile();
    assert_role_families_cover_codepoints(profile, BundledFontRole::EmojiFallback, &['\u{1F600}']);
}

#[cfg(not(feature = "emoji"))]
#[test]
fn bundled_profile_matrix_explicitly_omits_emoji_fallback_when_disabled() {
    let profile = crate::default_profile();
    assert!(!profile.supports_role(BundledFontRole::EmojiFallback));
    assert!(
        !profile
            .common_fallback_families
            .contains(&"Noto Color Emoji")
    );
}

#[cfg(feature = "cjk-lite")]
#[test]
fn default_profile_declares_cjk_role_when_enabled() {
    let profile = crate::default_profile();
    assert!(profile.supports_role(BundledFontRole::CjkFallback));
    assert!(
        profile
            .common_fallback_families
            .contains(&"Noto Sans CJK SC")
    );
}

#[cfg(feature = "cjk-lite")]
#[test]
fn bundled_profile_matrix_covers_cjk_fallback_contract() {
    let profile = crate::default_profile();
    assert_role_families_cover_codepoints(profile, BundledFontRole::CjkFallback, &['你', '界']);
}

#[cfg(not(feature = "cjk-lite"))]
#[test]
fn bundled_profile_matrix_explicitly_omits_cjk_fallback_when_disabled() {
    let profile = crate::default_profile();
    assert!(!profile.supports_role(BundledFontRole::CjkFallback));
    assert!(
        !profile
            .common_fallback_families
            .contains(&"Noto Sans CJK SC")
    );
}

#[cfg(all(feature = "emoji", not(feature = "cjk-lite")))]
#[test]
fn bundles_add_up_when_emoji_is_enabled() {
    assert_eq!(
        crate::default_fonts().len(),
        crate::bootstrap_fonts().len() + crate::emoji_fonts().len()
    );
    assert_eq!(crate::emoji_fonts().len(), 1);
}

#[cfg(all(feature = "cjk-lite", not(feature = "emoji")))]
#[test]
fn bundles_add_up_when_cjk_lite_is_enabled() {
    assert_eq!(
        crate::default_fonts().len(),
        crate::bootstrap_fonts().len() + crate::cjk_lite_fonts().len()
    );
    assert_eq!(crate::cjk_lite_fonts().len(), 1);
}

#[cfg(all(feature = "emoji", feature = "cjk-lite"))]
#[test]
fn bundles_add_up_when_emoji_and_cjk_lite_are_enabled() {
    assert_eq!(
        crate::default_fonts().len(),
        crate::bootstrap_fonts().len() + crate::emoji_fonts().len() + crate::cjk_lite_fonts().len()
    );
    assert_eq!(crate::emoji_fonts().len(), 1);
    assert_eq!(crate::cjk_lite_fonts().len(), 1);
}

#[test]
fn default_fonts_total_size_is_reasonable() {
    let total: usize = crate::default_fonts().iter().map(|b| b.len()).sum();

    #[cfg(all(
        not(feature = "emoji"),
        not(feature = "cjk-lite"),
        feature = "bootstrap-full"
    ))]
    assert!(
        total < 8_000_000,
        "default fonts too large (expected full bootstrap): {total}"
    );

    #[cfg(all(
        not(feature = "emoji"),
        not(feature = "cjk-lite"),
        not(feature = "bootstrap-full")
    ))]
    assert!(
        total < 2_000_000,
        "default fonts too large (expected subset bootstrap): {total}"
    );

    #[cfg(all(
        feature = "emoji",
        not(feature = "cjk-lite"),
        feature = "bootstrap-full"
    ))]
    assert!(
        total < 20_000_000,
        "default fonts too large (emoji + full bootstrap enabled): {total}"
    );

    #[cfg(all(
        feature = "emoji",
        not(feature = "cjk-lite"),
        not(feature = "bootstrap-full")
    ))]
    assert!(
        total < 15_000_000,
        "default fonts too large (emoji bundle enabled): {total}"
    );

    #[cfg(all(
        not(feature = "emoji"),
        feature = "cjk-lite",
        feature = "bootstrap-full"
    ))]
    assert!(
        total < 15_000_000,
        "default fonts too large (cjk-lite + full bootstrap enabled): {total}"
    );

    #[cfg(all(
        not(feature = "emoji"),
        feature = "cjk-lite",
        not(feature = "bootstrap-full")
    ))]
    assert!(
        total < 12_000_000,
        "default fonts too large (cjk-lite bundle enabled): {total}"
    );

    #[cfg(all(feature = "emoji", feature = "cjk-lite", feature = "bootstrap-full"))]
    assert!(
        total < 30_000_000,
        "default fonts too large (emoji + cjk-lite + full bootstrap enabled): {total}"
    );

    #[cfg(all(
        feature = "emoji",
        feature = "cjk-lite",
        not(feature = "bootstrap-full")
    ))]
    assert!(
        total < 25_000_000,
        "default fonts too large (emoji + cjk-lite bundles enabled): {total}"
    );
}
