use crate::profiles::{DEFAULT_EXPECTED_FAMILIES, DEFAULT_PROVIDED_ROLES};
use crate::*;
use fret_assets::{AssetLocator, AssetResolver, InMemoryAssetResolver};
use read_fonts::tables::name::NameId;
use read_fonts::{FontRef, TableProvider as _};
use std::collections::BTreeSet;

#[test]
fn imported_font_batch_accepts_supported_sfnt_signatures_only() {
    let true_type = [0x00, 0x01, 0x00, 0x00, 0x10];
    let open_type = *b"OTTOx";
    let collection = *b"ttcfx";
    let woff = *b"wOFFx";
    let junk = *b"plain";

    let batch = collect_supported_user_font_bytes([
        true_type.as_slice(),
        open_type.as_slice(),
        collection.as_slice(),
        woff.as_slice(),
        junk.as_slice(),
    ]);

    assert_eq!(batch.fonts.len(), 3);
    assert_eq!(batch.rejected_files, 2);
    assert_eq!(batch.fonts[0], true_type);
    assert_eq!(batch.fonts[1], open_type);
    assert_eq!(batch.fonts[2], collection);
}

#[test]
fn supported_user_font_import_extensions_lock_first_party_dialog_contract() {
    assert_eq!(
        SUPPORTED_USER_FONT_IMPORT_EXTENSIONS,
        &["ttf", "otf", "ttc"]
    );
}

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

fn profile_face_asset_keys(profile: &'static BundledFontProfile) -> Vec<&'static str> {
    dedupe_families(profile.faces.iter().map(|face| face.asset_key))
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
    let bytes_from_profile = profile
        .faces
        .iter()
        .map(|face| face.bytes)
        .collect::<Vec<_>>();

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
        "expected profile.faces to preserve bundled face byte ordering for profile {}",
        profile.name
    );
    assert_eq!(
        profile
            .face_for_asset_key(profile.faces[0].asset_key)
            .map(|face| face.family),
        Some(profile.faces[0].family),
        "expected face_for_asset_key to round-trip the first face for profile {}",
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
fn default_profile_font_bytes_are_non_empty() {
    for font in crate::default_profile().faces.iter().map(|face| face.bytes) {
        assert!(font.len() > 1024);
    }
}

#[test]
fn test_support_face_blobs_preserve_face_order_and_bytes() {
    let profile = crate::default_profile();
    let expected = profile
        .faces
        .iter()
        .map(|face| face.bytes.to_vec())
        .collect::<Vec<_>>();
    let actual = crate::test_support::face_blobs(profile.faces.iter()).collect::<Vec<_>>();

    assert_eq!(actual, expected);
}

#[test]
fn default_profile_matches_expected_manifest_contract() {
    let profile = crate::default_profile();
    assert!(!profile.faces.is_empty());
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
fn bundled_profile_asset_keys_are_unique_per_profile() {
    for profile in [crate::bootstrap_profile(), crate::default_profile()] {
        let asset_keys = profile_face_asset_keys(profile);
        assert_eq!(
            asset_keys.len(),
            profile.faces.len(),
            "expected asset keys to be unique for profile {}",
            profile.name
        );
    }
}

#[test]
fn bundled_face_asset_requests_are_font_bundle_requests() {
    let bundle = crate::bundled_asset_bundle();
    for profile in [crate::bootstrap_profile(), crate::default_profile()] {
        for face in profile.faces {
            let request = face.asset_request();
            assert_eq!(request.kind_hint, Some(fret_assets::AssetKindHint::Font));
            assert_eq!(
                request.locator,
                AssetLocator::bundle(bundle.clone(), face.asset_key),
                "expected face {} in profile {} to expose a package-scoped font locator",
                face.family,
                profile.name
            );
        }
    }
}

#[test]
fn bundled_profile_asset_entries_round_trip_through_bundle_resolver() {
    let bundle = crate::bundled_asset_bundle();
    for profile in [crate::bootstrap_profile(), crate::default_profile()] {
        let mut resolver = InMemoryAssetResolver::new();
        resolver.insert_bundle_entries(bundle.clone(), profile.asset_entries());

        for face in profile.faces {
            let resolved = resolver
                .resolve_bytes(&face.asset_request())
                .expect("expected bundled font face to resolve from profile asset entries");
            let expected_entry = face.asset_entry();
            assert_eq!(resolved.bytes.as_ref(), face.bytes);
            assert_eq!(resolved.revision, expected_entry.revision);
            assert_eq!(
                resolved.media_type.as_ref().map(|value| value.as_str()),
                Some(face.media_type)
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

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
#[test]
fn bundled_profile_matrix_covers_serif_contract() {
    for profile in [crate::bootstrap_profile(), crate::default_profile()] {
        assert_role_families_cover_codepoints(profile, BundledFontRole::UiSerif, &['A', 'm']);
    }
}

#[test]
fn bootstrap_profile_declares_expected_generic_guarantees() {
    let profile = crate::bootstrap_profile();
    assert!(profile.guarantees_generic_family(BundledGenericFamily::Monospace));
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    assert!(profile.guarantees_generic_family(BundledGenericFamily::Sans));
    #[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
    assert!(profile.guarantees_generic_family(BundledGenericFamily::Serif));
    #[cfg(not(any(feature = "bootstrap-subset", feature = "bootstrap-full")))]
    assert!(!profile.guarantees_generic_family(BundledGenericFamily::Sans));
    #[cfg(not(any(feature = "bootstrap-subset", feature = "bootstrap-full")))]
    assert!(!profile.guarantees_generic_family(BundledGenericFamily::Serif));
}

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
#[test]
fn default_profile_guarantees_serif_when_bootstrap_fonts_are_enabled() {
    let profile = crate::default_profile();
    assert_eq!(profile.ui_serif_families, &["Roboto Slab"]);
    assert!(profile.guarantees_generic_family(BundledGenericFamily::Serif));
}

#[cfg(any(feature = "bootstrap-subset", feature = "bootstrap-full"))]
#[test]
fn bundled_profile_matrix_guarantees_serif_when_bootstrap_fonts_are_enabled() {
    for profile in [crate::bootstrap_profile(), crate::default_profile()] {
        assert!(
            !profile.ui_serif_families.is_empty(),
            "expected bundled serif families in profile {}",
            profile.name
        );
        assert!(
            profile.guarantees_generic_family(BundledGenericFamily::Serif),
            "expected profile {} to guarantee serif when bootstrap fonts are enabled",
            profile.name
        );
    }
}

#[cfg(not(any(feature = "bootstrap-subset", feature = "bootstrap-full")))]
#[test]
fn bundled_profile_matrix_explicitly_omits_serif_contract_without_bootstrap_fonts() {
    for profile in [crate::bootstrap_profile(), crate::default_profile()] {
        assert!(
            profile.ui_serif_families.is_empty(),
            "expected no bundled serif families in profile {}",
            profile.name
        );
        assert!(
            !profile.guarantees_generic_family(BundledGenericFamily::Serif),
            "expected profile {} to avoid guaranteeing serif without bootstrap fonts",
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
    let emoji_count = crate::default_profile()
        .faces_for_role(crate::BundledFontRole::EmojiFallback)
        .count();
    assert_eq!(
        crate::default_profile().faces.len(),
        crate::bootstrap_profile().faces.len() + emoji_count
    );
    assert_eq!(emoji_count, 1);
}

#[cfg(all(feature = "cjk-lite", not(feature = "emoji")))]
#[test]
fn bundles_add_up_when_cjk_lite_is_enabled() {
    let cjk_count = crate::default_profile()
        .faces_for_role(crate::BundledFontRole::CjkFallback)
        .count();
    assert_eq!(
        crate::default_profile().faces.len(),
        crate::bootstrap_profile().faces.len() + cjk_count
    );
    assert_eq!(cjk_count, 1);
}

#[cfg(all(feature = "emoji", feature = "cjk-lite"))]
#[test]
fn bundles_add_up_when_emoji_and_cjk_lite_are_enabled() {
    let emoji_count = crate::default_profile()
        .faces_for_role(crate::BundledFontRole::EmojiFallback)
        .count();
    let cjk_count = crate::default_profile()
        .faces_for_role(crate::BundledFontRole::CjkFallback)
        .count();
    assert_eq!(
        crate::default_profile().faces.len(),
        crate::bootstrap_profile().faces.len() + emoji_count + cjk_count
    );
    assert_eq!(emoji_count, 1);
    assert_eq!(cjk_count, 1);
}

#[test]
fn default_profile_total_size_is_reasonable() {
    let total: usize = crate::default_profile()
        .faces
        .iter()
        .map(|face| face.bytes.len())
        .sum();

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
