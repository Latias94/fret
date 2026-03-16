use fret_icons::{IconId, IconRegistry, ids};

fn resolved_bytes(registry: &IconRegistry, id: IconId) -> Vec<u8> {
    registry
        .resolve_owned(&id)
        .unwrap_or_else(|err| panic!("expected `{}` to resolve: {err:?}", id.as_str()))
        .as_bytes()
        .to_vec()
}

#[test]
fn semantic_aliases_keep_first_registered_pack_when_lucide_installs_before_radix() {
    let mut registry = IconRegistry::default();

    fret_icons_lucide::register_icons(&mut registry);
    fret_icons_radix::register_icons(&mut registry);

    assert_eq!(
        resolved_bytes(&registry, ids::ui::SEARCH),
        resolved_bytes(&registry, IconId::new_static("lucide.search"))
    );
}

#[test]
fn semantic_aliases_keep_first_registered_pack_when_radix_installs_before_lucide() {
    let mut registry = IconRegistry::default();

    fret_icons_radix::register_icons(&mut registry);
    fret_icons_lucide::register_icons(&mut registry);

    assert_eq!(
        resolved_bytes(&registry, ids::ui::SEARCH),
        resolved_bytes(&registry, IconId::new_static("radix.magnifying-glass"))
    );
}

#[test]
fn app_code_can_explicitly_override_semantic_alias_after_pack_installation() {
    let mut registry = IconRegistry::default();

    fret_icons_lucide::register_icons(&mut registry);
    fret_icons_radix::register_icons(&mut registry);
    registry.alias(
        ids::ui::SEARCH,
        IconId::new_static("radix.magnifying-glass"),
    );

    assert_eq!(
        resolved_bytes(&registry, ids::ui::SEARCH),
        resolved_bytes(&registry, IconId::new_static("radix.magnifying-glass"))
    );
}
