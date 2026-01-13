use fret_core::{KeyCode, Modifiers};
use fret_runtime::{
    CommandId, CommandMeta, CommandRegistry, CommandScope, DefaultKeybinding, KeyChord,
    PlatformFilter, WhenExpr,
};

pub const COMMAND_PALETTE: &str = "app.command_palette";
pub const COMMAND_PALETTE_LEGACY: &str = "command_palette.toggle";
pub const FOCUS_NEXT: &str = "focus.next";
pub const FOCUS_PREVIOUS: &str = "focus.previous";

pub fn register_core_commands(registry: &mut CommandRegistry) {
    register_command_palette(registry);
    register_legacy_command_palette_alias(registry);
    register_focus_commands(registry);
}

pub fn register_command_palette(registry: &mut CommandRegistry) {
    let when = WhenExpr::parse("!focus.is_text_input").expect("valid when expression");

    let mut ctrl_mods = Modifiers::default();
    ctrl_mods.ctrl = true;
    let mut meta_mods = Modifiers::default();
    meta_mods.meta = true;

    let meta = CommandMeta::new("Command Palette")
        .with_category("App")
        .with_keywords(["command palette", "commands", "palette", "search"])
        .with_default_keybindings([
            DefaultKeybinding {
                platform: PlatformFilter::Windows,
                chord: KeyChord::new(KeyCode::KeyK, ctrl_mods),
                when: Some(when.clone()),
            },
            DefaultKeybinding {
                platform: PlatformFilter::Linux,
                chord: KeyChord::new(KeyCode::KeyK, ctrl_mods),
                when: Some(when.clone()),
            },
            DefaultKeybinding {
                platform: PlatformFilter::Web,
                chord: KeyChord::new(KeyCode::KeyK, ctrl_mods),
                when: Some(when.clone()),
            },
            DefaultKeybinding {
                platform: PlatformFilter::Macos,
                chord: KeyChord::new(KeyCode::KeyK, meta_mods),
                when: Some(when),
            },
        ])
        .hidden();

    registry.register(CommandId::new(COMMAND_PALETTE), meta);
}

pub fn register_legacy_command_palette_alias(registry: &mut CommandRegistry) {
    registry.register(
        CommandId::new(COMMAND_PALETTE_LEGACY),
        CommandMeta::new("Command Palette").hidden(),
    );
}

pub fn register_focus_commands(registry: &mut CommandRegistry) {
    registry.register(
        CommandId::new(FOCUS_NEXT),
        CommandMeta::new("Focus Next")
            .with_category("Focus")
            .with_keywords(["focus", "tab", "next"])
            .with_scope(CommandScope::Widget),
    );
    registry.register(
        CommandId::new(FOCUS_PREVIOUS),
        CommandMeta::new("Focus Previous")
            .with_category("Focus")
            .with_keywords(["focus", "tab", "previous"])
            .with_scope(CommandScope::Widget),
    );
}
