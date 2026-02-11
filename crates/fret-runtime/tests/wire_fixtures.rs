use fret_core::{KeyCode, Modifiers};
use fret_runtime::{
    InputContext, KeyChord, Keymap, MenuBar, MenuBarConfig, MenuBarError, Platform,
};

fn fixture_bytes(path: &str) -> &'static [u8] {
    match path {
        "keymap/v1-basic.json" => include_bytes!("fixtures/keymap/v1-basic.json"),
        "keymap/v2-sequence.json" => include_bytes!("fixtures/keymap/v2-sequence.json"),
        "keymap/v2-empty-keys.json" => include_bytes!("fixtures/keymap/v2-empty-keys.json"),
        "menubar/v2-replace.json" => include_bytes!("fixtures/menubar/v2-replace.json"),
        "menubar/v2-patch.json" => include_bytes!("fixtures/menubar/v2-patch.json"),
        "menubar/v2-invalid-both.json" => include_bytes!("fixtures/menubar/v2-invalid-both.json"),
        _ => panic!("unknown fixture path: {path}"),
    }
}

#[test]
fn keymap_v1_basic_fixture_parses() {
    let km = Keymap::from_bytes(fixture_bytes("keymap/v1-basic.json")).expect("keymap parses");

    let ctx = InputContext {
        platform: Platform::Windows,
        ..Default::default()
    };

    let chord = KeyChord::new(
        KeyCode::KeyO,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    assert!(km.resolve(&ctx, chord).is_some());
}

#[test]
fn keymap_v2_sequence_fixture_parses_and_exposes_continuations() {
    let km = Keymap::from_bytes(fixture_bytes("keymap/v2-sequence.json")).expect("keymap parses");

    let ctx = InputContext {
        platform: Platform::Windows,
        ..Default::default()
    };

    let ctrl_k = KeyChord::new(
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    let continuations = km.continuations(&ctx, &[ctrl_k]);
    assert!(!continuations.is_empty());
}

#[test]
fn keymap_v2_empty_keys_fixture_rejects_with_structured_error() {
    let err = Keymap::from_bytes(fixture_bytes("keymap/v2-empty-keys.json")).unwrap_err();
    assert!(matches!(
        err,
        fret_runtime::KeymapError::EmptyKeys { index: 0 }
    ));
}

#[test]
fn menubar_v2_replace_fixture_parses() {
    let bar =
        MenuBar::from_bytes(fixture_bytes("menubar/v2-replace.json")).expect("menubar parses");
    assert!(!bar.menus.is_empty());
}

#[test]
fn menubar_v2_patch_fixture_parses_into_config() {
    let cfg =
        MenuBarConfig::from_bytes(fixture_bytes("menubar/v2-patch.json")).expect("config parses");

    match cfg {
        MenuBarConfig::Patch(patch) => assert!(!patch.ops.is_empty()),
        other => panic!("expected patch config, got {other:?}"),
    }
}

#[test]
fn menubar_v2_invalid_both_fixture_is_rejected() {
    let err = MenuBarConfig::from_bytes(fixture_bytes("menubar/v2-invalid-both.json")).unwrap_err();
    assert!(matches!(err, MenuBarError::PatchFailed { .. }));
}
