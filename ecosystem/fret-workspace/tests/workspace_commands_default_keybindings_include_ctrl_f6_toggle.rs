use fret_core::{KeyCode, Modifiers};
use fret_runtime::{CommandId, CommandRegistry, KeyChord, PlatformFilter};
use fret_workspace::commands::{
    CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP, CMD_WORKSPACE_PANE_TOGGLE_TAB_STRIP_FOCUS,
    register_workspace_commands,
};

fn has_ctrl_f6_binding(meta: &fret_runtime::CommandMeta, platform: PlatformFilter) -> bool {
    let chord = KeyChord::new(
        KeyCode::F6,
        Modifiers {
            ctrl: true,
            ..Default::default()
        },
    );
    meta.default_keybindings
        .iter()
        .any(|b| b.platform == platform && b.sequence == vec![chord] && b.when.is_none())
}

#[test]
fn toggle_tab_strip_focus_has_default_ctrl_f6_binding() {
    let mut registry = CommandRegistry::default();
    register_workspace_commands(&mut registry);

    let toggle = registry
        .get(CommandId::from(CMD_WORKSPACE_PANE_TOGGLE_TAB_STRIP_FOCUS))
        .expect("toggle command should be registered");
    assert!(has_ctrl_f6_binding(toggle, PlatformFilter::Windows));
    assert!(has_ctrl_f6_binding(toggle, PlatformFilter::Linux));
    assert!(has_ctrl_f6_binding(toggle, PlatformFilter::Macos));

    let focus = registry
        .get(CommandId::from(CMD_WORKSPACE_PANE_FOCUS_TAB_STRIP))
        .expect("focus command should be registered");
    assert!(
        focus.default_keybindings.is_empty(),
        "focus-tab-strip remains unbound by default; toggle owns Ctrl+F6"
    );
}
