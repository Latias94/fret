//! Command ids and registration for the node graph editor.
//!
//! This module provides:
//! - stable `CommandId` string constants for node-graph editing actions,
//! - an optional registration helper to expose these commands to menus and command palette UIs.

use fret_core::{KeyCode, Modifiers};
use fret_runtime::{
    CommandId, CommandMeta, CommandRegistry, CommandScope, DefaultKeybinding, KeyChord,
    PlatformFilter, WhenExpr,
};

pub const CMD_NODE_GRAPH_UNDO: &str = "node_graph.undo";
pub const CMD_NODE_GRAPH_REDO: &str = "node_graph.redo";
pub const CMD_NODE_GRAPH_OPEN_INSERT_NODE: &str = "node_graph.open_insert_node";
pub const CMD_NODE_GRAPH_COPY: &str = "node_graph.copy";
pub const CMD_NODE_GRAPH_CUT: &str = "node_graph.cut";
pub const CMD_NODE_GRAPH_PASTE: &str = "node_graph.paste";
pub const CMD_NODE_GRAPH_DUPLICATE: &str = "node_graph.duplicate";
pub const CMD_NODE_GRAPH_SELECT_ALL: &str = "node_graph.select_all";
pub const CMD_NODE_GRAPH_DELETE_SELECTION: &str = "node_graph.delete_selection";

fn kb(platform: PlatformFilter, key: KeyCode, mods: Modifiers) -> DefaultKeybinding {
    DefaultKeybinding {
        platform,
        chord: KeyChord::new(key, mods),
        when: None,
    }
}

fn when_node_graph_editing() -> WhenExpr {
    WhenExpr::parse("!focus.is_text_input").expect("valid when expr")
}

pub fn register_node_graph_commands(registry: &mut CommandRegistry) {
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
    let win_ctrl = |key: KeyCode| {
        kb(
            PlatformFilter::Windows,
            key,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        )
    };
    let linux_ctrl = |key: KeyCode| {
        kb(
            PlatformFilter::Linux,
            key,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        )
    };
    let web_ctrl = |key: KeyCode| {
        kb(
            PlatformFilter::Web,
            key,
            Modifiers {
                ctrl: true,
                ..Default::default()
            },
        )
    };

    let mac_cmd_shift = |key: KeyCode| {
        kb(
            PlatformFilter::Macos,
            key,
            Modifiers {
                meta: true,
                shift: true,
                ..Default::default()
            },
        )
    };
    let win_ctrl_shift = |key: KeyCode| {
        kb(
            PlatformFilter::Windows,
            key,
            Modifiers {
                ctrl: true,
                shift: true,
                ..Default::default()
            },
        )
    };
    let linux_ctrl_shift = |key: KeyCode| {
        kb(
            PlatformFilter::Linux,
            key,
            Modifiers {
                ctrl: true,
                shift: true,
                ..Default::default()
            },
        )
    };
    let web_ctrl_shift = |key: KeyCode| {
        kb(
            PlatformFilter::Web,
            key,
            Modifiers {
                ctrl: true,
                shift: true,
                ..Default::default()
            },
        )
    };

    let widget = CommandScope::Widget;

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_OPEN_INSERT_NODE),
        CommandMeta::new("Insert Node…")
            .with_category("Node Graph")
            .with_keywords(["insert", "node", "create", "add"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_UNDO),
        CommandMeta::new("Undo")
            .with_category("Node Graph")
            .with_keywords(["undo", "history"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .with_default_keybindings([
                mac_cmd(KeyCode::KeyZ),
                win_ctrl(KeyCode::KeyZ),
                linux_ctrl(KeyCode::KeyZ),
                web_ctrl(KeyCode::KeyZ),
            ])
            .repeatable(),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_REDO),
        CommandMeta::new("Redo")
            .with_category("Node Graph")
            .with_keywords(["redo", "history"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .with_default_keybindings([
                mac_cmd_shift(KeyCode::KeyZ),
                win_ctrl(KeyCode::KeyY),
                linux_ctrl(KeyCode::KeyY),
                web_ctrl(KeyCode::KeyY),
                win_ctrl_shift(KeyCode::KeyZ),
                linux_ctrl_shift(KeyCode::KeyZ),
                web_ctrl_shift(KeyCode::KeyZ),
            ])
            .repeatable(),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_COPY),
        CommandMeta::new("Copy")
            .with_category("Node Graph")
            .with_keywords(["copy", "clipboard"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .with_default_keybindings([
                mac_cmd(KeyCode::KeyC),
                win_ctrl(KeyCode::KeyC),
                linux_ctrl(KeyCode::KeyC),
                web_ctrl(KeyCode::KeyC),
            ]),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_CUT),
        CommandMeta::new("Cut")
            .with_category("Node Graph")
            .with_keywords(["cut", "clipboard"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .with_default_keybindings([
                mac_cmd(KeyCode::KeyX),
                win_ctrl(KeyCode::KeyX),
                linux_ctrl(KeyCode::KeyX),
                web_ctrl(KeyCode::KeyX),
            ]),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_PASTE),
        CommandMeta::new("Paste")
            .with_category("Node Graph")
            .with_keywords(["paste", "clipboard"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .with_default_keybindings([
                mac_cmd(KeyCode::KeyV),
                win_ctrl(KeyCode::KeyV),
                linux_ctrl(KeyCode::KeyV),
                web_ctrl(KeyCode::KeyV),
            ]),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_DUPLICATE),
        CommandMeta::new("Duplicate")
            .with_category("Node Graph")
            .with_keywords(["duplicate", "clone"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .with_default_keybindings([
                mac_cmd(KeyCode::KeyD),
                win_ctrl(KeyCode::KeyD),
                linux_ctrl(KeyCode::KeyD),
                web_ctrl(KeyCode::KeyD),
            ]),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_SELECT_ALL),
        CommandMeta::new("Select All")
            .with_category("Node Graph")
            .with_keywords(["select", "all"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .with_default_keybindings([
                mac_cmd(KeyCode::KeyA),
                win_ctrl(KeyCode::KeyA),
                linux_ctrl(KeyCode::KeyA),
                web_ctrl(KeyCode::KeyA),
            ]),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_DELETE_SELECTION),
        CommandMeta::new("Delete Selection")
            .with_category("Node Graph")
            .with_keywords(["delete", "remove", "selection"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );
}
