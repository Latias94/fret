use super::*;
use fret_assets::{AssetLocator, AssetResolver, InMemoryAssetResolver};

#[test]
fn cjk_profile_manifest_is_consistent() {
    let profile = default_profile();

    assert_eq!(profile.name, "cjk-lite");
    assert_eq!(profile.expected_family_names, &["Noto Sans CJK SC"]);
    assert_eq!(profile.provided_roles, &[BundledFontRole::CjkFallback]);
    assert_eq!(profile.common_fallback_families, &["Noto Sans CJK SC"]);
    assert!(profile.supports_role(BundledFontRole::CjkFallback));
    assert_eq!(profile.faces.len(), 1);
}

#[test]
fn cjk_face_asset_identity_uses_extension_bundle() {
    let face = default_profile().faces[0];

    assert_eq!(face.bundle_name, env!("CARGO_PKG_NAME"));
    assert_eq!(
        face.asset_locator(),
        AssetLocator::bundle(bundled_asset_bundle(), face.asset_key)
    );

    let mut resolver = InMemoryAssetResolver::new();
    resolver.insert_bundle_entries(bundled_asset_bundle(), default_profile().asset_entries());

    let resolved = resolver
        .resolve_bytes(&face.asset_request())
        .expect("expected cjk bundled face to resolve");
    assert_eq!(resolved.locator, face.asset_locator());
    assert_eq!(resolved.bytes.as_ref(), face.bytes);
}
