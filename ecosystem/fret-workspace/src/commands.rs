use fret_core::{KeyCode, Modifiers};
use fret_runtime::{
    CommandId, CommandMeta, CommandRegistry, DefaultKeybinding, KeyChord, PlatformFilter,
};

pub const CMD_WORKSPACE_TAB_NEXT: &str = "workspace.tab.next";
pub const CMD_WORKSPACE_TAB_PREV: &str = "workspace.tab.prev";
pub const CMD_WORKSPACE_TAB_CLOSE: &str = "workspace.tab.close";

/// Prefix for "close a specific tab" commands.
///
/// This is intentionally a prefix-based command family so apps can implement their own tab models
/// without exposing internal IDs via generic enum payloads.
pub const CMD_WORKSPACE_TAB_CLOSE_PREFIX: &str = "workspace.tab.close.";

fn kb(platform: PlatformFilter, key: KeyCode, mods: Modifiers) -> DefaultKeybinding {
    DefaultKeybinding {
        platform,
        chord: KeyChord::new(key, mods),
        when: None,
    }
}

pub fn register_workspace_commands(registry: &mut CommandRegistry) {
    let win_ctrl = |key: KeyCode, shift: bool| {
        kb(
            PlatformFilter::Windows,
            key,
            Modifiers {
                ctrl: true,
                shift,
                ..Default::default()
            },
        )
    };
    let linux_ctrl = |key: KeyCode, shift: bool| {
        kb(
            PlatformFilter::Linux,
            key,
            Modifiers {
                ctrl: true,
                shift,
                ..Default::default()
            },
        )
    };
    let mac_ctrl = |key: KeyCode, shift: bool| {
        kb(
            PlatformFilter::Macos,
            key,
            Modifiers {
                ctrl: true,
                shift,
                ..Default::default()
            },
        )
    };
    let mac_cmd = |key: KeyCode| {
        kb(
            PlatformFilter::Macos,
            key,
            Modifiers {
                meta: true,
                ..Default::default()
            },
        )
    };

    registry.register(
        CommandId::new(CMD_WORKSPACE_TAB_NEXT),
        CommandMeta::new("Next Tab")
            .with_category("Workspace")
            .with_keywords(["tab", "next", "workspace"])
            .with_default_keybindings([
                win_ctrl(KeyCode::Tab, false),
                linux_ctrl(KeyCode::Tab, false),
                mac_ctrl(KeyCode::Tab, false),
            ]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_TAB_PREV),
        CommandMeta::new("Previous Tab")
            .with_category("Workspace")
            .with_keywords(["tab", "previous", "workspace"])
            .with_default_keybindings([
                win_ctrl(KeyCode::Tab, true),
                linux_ctrl(KeyCode::Tab, true),
                mac_ctrl(KeyCode::Tab, true),
            ]),
    );

    registry.register(
        CommandId::new(CMD_WORKSPACE_TAB_CLOSE),
        CommandMeta::new("Close Tab")
            .with_category("Workspace")
            .with_keywords(["tab", "close", "workspace"])
            .with_default_keybindings([
                win_ctrl(KeyCode::KeyW, false),
                linux_ctrl(KeyCode::KeyW, false),
                mac_cmd(KeyCode::KeyW),
            ]),
    );
}
