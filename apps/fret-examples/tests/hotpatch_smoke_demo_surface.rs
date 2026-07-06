#[test]
fn hotpatch_smoke_demo_routes_model_writes_through_owner() {
    let source = include_str!("../../fret-demo/src/bin/hotpatch_smoke_demo.rs");
    let compact_source = source
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();

    for needle in [
        "struct HotpatchSmokeModelOwner<'a>",
        "models: &'a mut ModelStore",
        "fn increment_counter(",
        "fn set_debug(",
        "HotpatchSmokeModelOwner::new(app.models_mut())",
        ".increment_counter(&state.counter)",
        ".set_debug(&state.debug, &msg)",
    ] {
        assert!(
            source.contains(needle),
            "hotpatch smoke demo should route writable model state through its demo-local owner; missing `{needle}`"
        );
    }

    for forbidden in [
        "app.models_mut().update(",
        "app.models_mut().update::<",
        "app.models_mut().update_any(",
        "app.models_mut().update_any::<",
        "ModelStore::update(",
        "ModelStore::update::<",
        "ModelStore::update_any(",
        "ModelStore::update_any::<",
    ] {
        assert!(
            !compact_source.contains(forbidden),
            "hotpatch smoke event/command paths should not bypass HotpatchSmokeModelOwner; unexpected `{forbidden}`"
        );
    }
}
