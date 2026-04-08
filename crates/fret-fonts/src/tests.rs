use crate::*;
use fret_assets::{AssetLocator, AssetResolver, InMemoryAssetResolver};
use read_fonts::tables::name::NameId;
use read_fonts::{FontRef, TableProvider as _};
use std::collections::BTreeSet;

#[test]
fn supported_user_font_bytes_accept_supported_sfnt_signatures_only() {
    let true_type = [0x00, 0x01, 0x00, 0x00, 0x10];
    let open_type = *b"OTTOx";
    let collection = *b"ttcfx";
    let woff = *b"wOFFx";
    let junk = *b"plain";

    assert!(is_supported_user_font_bytes(true_type.as_slice()));
    assert!(is_supported_user_font_bytes(open_type.as_slice()));
    assert!(is_supported_user_font_bytes(collection.as_slice()));
    assert!(!is_supported_user_font_bytes(woff.as_slice()));
    assert!(!is_supported_user_font_bytes(junk.as_slice()));
}

#[test]
fn imported_font_asset_batch_builds_memory_font_requests() {
    let true_type = [0x00, 0x01, 0x00, 0x00, 0x10];
    let open_type = *b"OTTOx";
    let junk = *b"plain";

    let batch = build_imported_font_asset_batch([
        ("Acme Sans.ttf", true_type.as_slice()),
        ("Acme Serif.otf", open_type.as_slice()),
        ("README.txt", junk.as_slice()),
    ]);

    assert_eq!(batch.requests.len(), 2);
    assert_eq!(batch.resolved.len(), 2);
    assert_eq!(batch.rejected_files, 1);

    for request in &batch.requests {
        assert_eq!(request.kind_hint, Some(fret_assets::AssetKindHint::Font));
        assert!(
            matches!(request.locator, AssetLocator::Memory(_)),
            "expected imported font requests to stay on the memory asset lane"
        );
    }

    assert_eq!(batch.requests[0].locator, batch.resolved[0].locator);
    assert_eq!(batch.requests[1].locator, batch.resolved[1].locator);
    assert_ne!(batch.requests[0].locator, batch.requests[1].locator);
}

#[test]
fn imported_font_asset_batch_is_stable_for_same_input() {
    let true_type = [0x00, 0x01, 0x00, 0x00, 0x10];

    let first = build_imported_font_asset_batch([("Acme Sans.ttf", true_type.as_slice())]);
    let second = build_imported_font_asset_batch([("Acme Sans.ttf", true_type.as_slice())]);

    assert_eq!(first, second);
}

#[test]
fn imported_font_asset_resolver_round_trips_memory_requests() {
    let true_type = [0x00, 0x01, 0x00, 0x00, 0x10];
    let collection = *b"ttcfx";
    let batch = build_imported_font_asset_batch([
        ("Acme Sans.ttf", true_type.as_slice()),
        ("Acme Collection.ttc", collection.as_slice()),
    ]);
    let resolver = ImportedFontAssetResolver::default();
    resolver.replace_batch(&batch);

    for (request, resolved) in batch.requests.iter().zip(batch.resolved.iter()) {
        let actual = resolver
            .resolve_bytes(request)
            .expect("expected imported font asset to resolve");
        assert_eq!(actual, *resolved);
    }
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

fn assert_profile_manifest_consistency(profile: &'static BundledFontProfile) {
    assert_eq!(
        profile.expected_family_names,
        profile_face_families(profile).as_slice(),
        "expected family manifest to match bundled faces for profile {}",
        profile.name
    );
    assert_eq!(
        profile.ui_sans_families,
        families_for_role(profile, BundledFontRole::UiSans).as_slice(),
        "expected ui_sans_families to match UiSans role families for profile {}",
        profile.name
    );
    assert_eq!(
        profile.ui_serif_families,
        families_for_role(profile, BundledFontRole::UiSerif).as_slice(),
        "expected ui_serif_families to match UiSerif role families for profile {}",
        profile.name
    );
    assert_eq!(
        profile.ui_mono_families,
        families_for_role(profile, BundledFontRole::UiMonospace).as_slice(),
        "expected ui_mono_families to match UiMonospace role families for profile {}",
        profile.name
    );
}

#[test]
fn bootstrap_profile_manifest_is_consistent() {
    assert_profile_manifest_consistency(crate::bootstrap_profile());
}

#[test]
fn default_profile_manifest_is_consistent() {
    assert_profile_manifest_consistency(crate::default_profile());
}

#[test]
fn default_profile_matches_framework_baseline_surface() {
    let bootstrap = crate::bootstrap_profile();
    let default = crate::default_profile();

    assert_eq!(default.faces, bootstrap.faces);
    assert_eq!(default.provided_roles, bootstrap.provided_roles);
    assert_eq!(
        default.guaranteed_generic_families,
        bootstrap.guaranteed_generic_families
    );
    assert!(default.common_fallback_families.is_empty());
    assert!(!default.supports_role(BundledFontRole::EmojiFallback));
    assert!(!default.supports_role(BundledFontRole::CjkFallback));
}

#[test]
fn baseline_profile_guarantees_three_ui_generic_families() {
    let profile = crate::default_profile();

    assert!(profile.guarantees_generic_family(BundledGenericFamily::Sans));
    assert!(profile.guarantees_generic_family(BundledGenericFamily::Serif));
    assert!(profile.guarantees_generic_family(BundledGenericFamily::Monospace));
}

#[test]
fn bundled_face_asset_identity_stays_on_fret_fonts_bundle() {
    for profile in [crate::bootstrap_profile(), crate::default_profile()] {
        for face in profile.faces {
            assert_eq!(face.bundle_name, env!("CARGO_PKG_NAME"));
            assert_eq!(
                face.asset_locator(),
                AssetLocator::bundle(crate::bundled_asset_bundle(), face.asset_key)
            );
        }
    }
}

#[test]
fn bundled_asset_entries_round_trip_through_in_memory_resolver() {
    let mut resolver = InMemoryAssetResolver::new();
    resolver.insert_bundle_entries(
        crate::bundled_asset_bundle(),
        crate::default_profile().asset_entries(),
    );

    for face in crate::default_profile().faces {
        let resolved = resolver
            .resolve_bytes(&face.asset_request())
            .expect("expected bundled face request to resolve");
        assert_eq!(resolved.locator, face.asset_locator());
        assert_eq!(resolved.bytes.as_ref(), face.bytes);
        assert_eq!(
            resolved
                .media_type
                .as_ref()
                .map(|media_type| media_type.as_str()),
            Some(face.media_type)
        );
    }
}

#[test]
fn bundled_face_family_names_match_font_name_tables() {
    for face in crate::default_profile().faces {
        let family_names = decoded_family_names(face.bytes);
        assert!(
            family_names.iter().any(|name| name == face.family),
            "expected face family {:?} to appear in the font name table {:?}",
            face.family,
            family_names
        );
    }
}
