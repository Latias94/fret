use super::*;
use crate::{CommandId, InputContext, KeyChord, Platform, WhenExpr};
use fret_core::{KeyCode, Modifiers};
use std::sync::Arc;

#[test]
fn keymap_rejects_unknown_when_identifiers() {
    let bytes = br#"{
            "keymap_version": 1,
            "bindings": [
                {
                    "command": "test.command",
                    "keys": { "mods": [], "key": "KeyA" },
                    "when": "ui.multi_windo"
                }
            ]
        }"#;

    let err = Keymap::from_bytes(bytes).unwrap_err();
    assert!(matches!(
        err,
        KeymapError::WhenValidationFailed { index: 0, .. }
    ));
}

#[test]
fn keymap_accepts_modifier_tokens_case_insensitive_and_aliases() {
    let bytes = br#"{
            "keymap_version": 1,
            "bindings": [
                { "command": "test.shift", "keys": { "mods": ["Shift"], "key": "Tab" } },
                { "command": "test.ctrl", "keys": { "mods": ["Control"], "key": "KeyA" } },
                { "command": "test.alt", "keys": { "mods": ["Option"], "key": "KeyB" } },
                { "command": "test.meta", "keys": { "mods": ["Command"], "key": "KeyC" } },
                { "command": "test.alt_gr", "keys": { "mods": ["Alt_Gr"], "key": "KeyD" } }
            ]
        }"#;

    Keymap::from_bytes(bytes).expect("keymap parses");
}

#[test]
fn keymap_rejects_string_keys_used_as_boolean_when() {
    let bytes = br#"{
            "keymap_version": 1,
            "bindings": [
                {
                    "command": "test.command",
                    "keys": { "mods": [], "key": "KeyA" },
                    "when": "dnd.external_payload"
                }
            ]
        }"#;

    let err = Keymap::from_bytes(bytes).unwrap_err();
    assert!(matches!(
        err,
        KeymapError::WhenValidationFailed { index: 0, .. }
    ));
}

#[test]
fn keymap_conflicts_detects_last_wins_overrides() {
    let bytes = br#"{
            "keymap_version": 1,
            "bindings": [
                { "command": "test.a", "keys": { "mods": ["ctrl"], "key": "KeyP" } },
                { "command": "test.b", "keys": { "mods": ["ctrl"], "key": "KeyP" } }
            ]
        }"#;

    let km = Keymap::from_bytes(bytes).unwrap();
    let conflicts = km.conflicts();
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].kind, KeymapConflictKind::Override);
    assert_eq!(conflicts[0].entries.len(), 2);
    assert_eq!(
        conflicts[0].entries[0].command.as_ref().unwrap().as_str(),
        "test.a"
    );
    assert_eq!(
        conflicts[0].entries[1].command.as_ref().unwrap().as_str(),
        "test.b"
    );
}

#[test]
fn keymap_continuations_list_valid_next_chords_and_filters_unbound() {
    let ctrl_k = KeyChord::new(
        KeyCode::KeyK,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    let up = KeyChord::new(KeyCode::ArrowUp, Modifiers::default());
    let down_shift = KeyChord::new(
        KeyCode::ArrowDown,
        Modifiers {
            shift: true,
            ..Default::default()
        },
    );

    let mut km = Keymap::empty();
    km.push_binding(Binding {
        platform: PlatformFilter::All,
        sequence: vec![ctrl_k, up],
        when: None,
        command: Some(CommandId::new(Arc::<str>::from("test.up"))),
    });
    km.push_binding(Binding {
        platform: PlatformFilter::All,
        sequence: vec![ctrl_k, down_shift],
        when: None,
        command: Some(CommandId::new(Arc::<str>::from("test.down_shift"))),
    });

    let ctx = InputContext {
        platform: Platform::Windows,
        ..Default::default()
    };

    let out = km.continuations(&ctx, &[ctrl_k]);
    assert_eq!(out.len(), 2);
    assert!(out.iter().any(|c| {
        c.next == up
            && c.matched
                .exact
                .as_ref()
                .is_some_and(|c| c.as_ref().is_some_and(|id| id.as_str() == "test.up"))
    }));
    assert!(out.iter().any(|c| c.next == down_shift
        && c.matched.exact.as_ref().is_some_and(|c| {
            c.as_ref()
                .is_some_and(|id| id.as_str() == "test.down_shift")
        })));

    // Explicitly unbind one of the continuations: it should no longer be listed.
    km.push_binding(Binding {
        platform: PlatformFilter::All,
        sequence: vec![ctrl_k, up],
        when: None,
        command: None,
    });

    let out = km.continuations(&ctx, &[ctrl_k]);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].next, down_shift);
}

#[test]
fn keymap_display_shortcut_prefers_non_modal_non_text_context() {
    let mut km = Keymap::empty();
    let cmd = CommandId::new(Arc::<str>::from("test.cmd"));

    let ctrl_p = KeyChord::new(
        KeyCode::KeyP,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    let ctrl_e = KeyChord::new(
        KeyCode::KeyE,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );

    km.push_binding(Binding {
        platform: PlatformFilter::All,
        sequence: vec![ctrl_e],
        when: Some(WhenExpr::parse("focus.is_text_input").unwrap()),
        command: Some(cmd.clone()),
    });
    km.push_binding(Binding {
        platform: PlatformFilter::All,
        sequence: vec![ctrl_p],
        when: None,
        command: Some(cmd.clone()),
    });

    let base = InputContext {
        platform: Platform::Windows,
        ui_has_modal: true,
        focus_is_text_input: true,
        ..Default::default()
    };

    let out = km
        .display_shortcut_for_command_sequence(&base, &cmd)
        .unwrap();
    assert_eq!(out, vec![ctrl_p]);
}

#[test]
fn keymap_display_shortcut_falls_back_to_modal_context_when_needed() {
    let mut km = Keymap::empty();
    let cmd = CommandId::new(Arc::<str>::from("test.modal_only"));

    let esc = KeyChord::new(KeyCode::Escape, Modifiers::default());
    km.push_binding(Binding {
        platform: PlatformFilter::All,
        sequence: vec![esc],
        when: Some(WhenExpr::parse("ui.has_modal").unwrap()),
        command: Some(cmd.clone()),
    });

    let base = InputContext {
        platform: Platform::Windows,
        ui_has_modal: false,
        focus_is_text_input: false,
        ..Default::default()
    };

    let out = km
        .display_shortcut_for_command_sequence(&base, &cmd)
        .unwrap();
    assert_eq!(out, vec![esc]);
}

#[test]
fn keymap_display_shortcut_prefers_later_overrides_for_the_same_command() {
    let mut km = Keymap::empty();
    let cmd = CommandId::new(Arc::<str>::from("test.cmd"));

    let ctrl_p = KeyChord::new(
        KeyCode::KeyP,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    let ctrl_shift_p = KeyChord::new(
        KeyCode::KeyP,
        Modifiers {
            ctrl: true,
            shift: true,
            ..Default::default()
        },
    );

    km.push_binding(Binding {
        platform: PlatformFilter::All,
        sequence: vec![ctrl_p],
        when: None,
        command: Some(cmd.clone()),
    });
    km.push_binding(Binding {
        platform: PlatformFilter::All,
        sequence: vec![ctrl_shift_p],
        when: None,
        command: Some(cmd.clone()),
    });

    let base = InputContext {
        platform: Platform::Windows,
        ..Default::default()
    };

    let out = km
        .display_shortcut_for_command_sequence(&base, &cmd)
        .unwrap();
    assert_eq!(out, vec![ctrl_shift_p]);
}

#[test]
fn keymap_display_shortcut_ignores_explicit_unbinds() {
    let mut km = Keymap::empty();
    let cmd = CommandId::new(Arc::<str>::from("test.cmd"));

    let ctrl_p = KeyChord::new(
        KeyCode::KeyP,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    let ctrl_shift_p = KeyChord::new(
        KeyCode::KeyP,
        Modifiers {
            ctrl: true,
            shift: true,
            ..Default::default()
        },
    );

    km.push_binding(Binding {
        platform: PlatformFilter::All,
        sequence: vec![ctrl_p],
        when: None,
        command: Some(cmd.clone()),
    });
    km.push_binding(Binding {
        platform: PlatformFilter::All,
        sequence: vec![ctrl_p],
        when: None,
        command: None,
    });
    km.push_binding(Binding {
        platform: PlatformFilter::All,
        sequence: vec![ctrl_shift_p],
        when: None,
        command: Some(cmd.clone()),
    });

    let base = InputContext {
        platform: Platform::Windows,
        ..Default::default()
    };

    let out = km
        .display_shortcut_for_command_sequence(&base, &cmd)
        .unwrap();
    assert_eq!(out, vec![ctrl_shift_p]);
}
