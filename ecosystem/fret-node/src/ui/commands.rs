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
pub const CMD_NODE_GRAPH_OPEN_SPLIT_EDGE_INSERT_NODE: &str =
    "node_graph.open_split_edge_insert_node";
pub const CMD_NODE_GRAPH_INSERT_REROUTE: &str = "node_graph.insert_reroute";
pub const CMD_NODE_GRAPH_OPEN_CONVERSION_PICKER: &str = "node_graph.open_conversion_picker";
pub const CMD_NODE_GRAPH_COPY: &str = "node_graph.copy";
pub const CMD_NODE_GRAPH_CUT: &str = "node_graph.cut";
pub const CMD_NODE_GRAPH_PASTE: &str = "node_graph.paste";
pub const CMD_NODE_GRAPH_DUPLICATE: &str = "node_graph.duplicate";
pub const CMD_NODE_GRAPH_SELECT_ALL: &str = "node_graph.select_all";
pub const CMD_NODE_GRAPH_DELETE_SELECTION: &str = "node_graph.delete_selection";
pub const CMD_NODE_GRAPH_NUDGE_LEFT: &str = "node_graph.nudge_left";
pub const CMD_NODE_GRAPH_NUDGE_RIGHT: &str = "node_graph.nudge_right";
pub const CMD_NODE_GRAPH_NUDGE_UP: &str = "node_graph.nudge_up";
pub const CMD_NODE_GRAPH_NUDGE_DOWN: &str = "node_graph.nudge_down";
pub const CMD_NODE_GRAPH_NUDGE_LEFT_FAST: &str = "node_graph.nudge_left_fast";
pub const CMD_NODE_GRAPH_NUDGE_RIGHT_FAST: &str = "node_graph.nudge_right_fast";
pub const CMD_NODE_GRAPH_NUDGE_UP_FAST: &str = "node_graph.nudge_up_fast";
pub const CMD_NODE_GRAPH_NUDGE_DOWN_FAST: &str = "node_graph.nudge_down_fast";
pub const CMD_NODE_GRAPH_ALIGN_LEFT: &str = "node_graph.align_left";
pub const CMD_NODE_GRAPH_ALIGN_RIGHT: &str = "node_graph.align_right";
pub const CMD_NODE_GRAPH_ALIGN_TOP: &str = "node_graph.align_top";
pub const CMD_NODE_GRAPH_ALIGN_BOTTOM: &str = "node_graph.align_bottom";
pub const CMD_NODE_GRAPH_ALIGN_CENTER_X: &str = "node_graph.align_center_x";
pub const CMD_NODE_GRAPH_ALIGN_CENTER_Y: &str = "node_graph.align_center_y";
pub const CMD_NODE_GRAPH_DISTRIBUTE_X: &str = "node_graph.distribute_x";
pub const CMD_NODE_GRAPH_DISTRIBUTE_Y: &str = "node_graph.distribute_y";
pub const CMD_NODE_GRAPH_FOCUS_NEXT: &str = "node_graph.focus_next";
pub const CMD_NODE_GRAPH_FOCUS_PREV: &str = "node_graph.focus_prev";
pub const CMD_NODE_GRAPH_FOCUS_NEXT_EDGE: &str = "node_graph.focus_next_edge";
pub const CMD_NODE_GRAPH_FOCUS_PREV_EDGE: &str = "node_graph.focus_prev_edge";
pub const CMD_NODE_GRAPH_FRAME_SELECTION: &str = "node_graph.frame_selection";
pub const CMD_NODE_GRAPH_FRAME_ALL: &str = "node_graph.frame_all";
pub const CMD_NODE_GRAPH_RESET_VIEW: &str = "node_graph.reset_view";
pub const CMD_NODE_GRAPH_ZOOM_IN: &str = "node_graph.zoom_in";
pub const CMD_NODE_GRAPH_ZOOM_OUT: &str = "node_graph.zoom_out";
pub const CMD_NODE_GRAPH_TOGGLE_CONNECTION_MODE: &str = "node_graph.toggle_connection_mode";
pub const CMD_NODE_GRAPH_CREATE_GROUP: &str = "node_graph.create_group";
pub const CMD_NODE_GRAPH_GROUP_BRING_TO_FRONT: &str = "node_graph.group.bring_to_front";
pub const CMD_NODE_GRAPH_GROUP_SEND_TO_BACK: &str = "node_graph.group.send_to_back";
pub const CMD_NODE_GRAPH_GROUP_RENAME: &str = "node_graph.group.rename";

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
        CommandId::from(CMD_NODE_GRAPH_CREATE_GROUP),
        CommandMeta::new("Create Group")
            .with_category("Node Graph")
            .with_keywords(["group", "frame", "container"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_GROUP_BRING_TO_FRONT),
        CommandMeta::new("Bring Group to Front")
            .with_category("Node Graph")
            .with_keywords(["group", "bring", "front", "z", "order"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_GROUP_SEND_TO_BACK),
        CommandMeta::new("Send Group to Back")
            .with_category("Node Graph")
            .with_keywords(["group", "send", "back", "z", "order"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_GROUP_RENAME),
        CommandMeta::new("Rename Group…")
            .with_category("Node Graph")
            .with_keywords(["group", "rename", "title"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_OPEN_SPLIT_EDGE_INSERT_NODE),
        CommandMeta::new("Insert Node on Edge…")
            .with_category("Node Graph")
            .with_keywords(["insert", "node", "edge", "split"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_INSERT_REROUTE),
        CommandMeta::new("Insert Reroute")
            .with_category("Node Graph")
            .with_keywords(["reroute", "edge", "wire"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_OPEN_CONVERSION_PICKER),
        CommandMeta::new("Convert Connection…")
            .with_category("Node Graph")
            .with_keywords(["convert", "connection", "type"])
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

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_NUDGE_LEFT),
        CommandMeta::new("Nudge Left")
            .with_category("Node Graph")
            .with_keywords(["nudge", "move", "left"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .repeatable(),
    );
    registry.register(
        CommandId::from(CMD_NODE_GRAPH_NUDGE_RIGHT),
        CommandMeta::new("Nudge Right")
            .with_category("Node Graph")
            .with_keywords(["nudge", "move", "right"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .repeatable(),
    );
    registry.register(
        CommandId::from(CMD_NODE_GRAPH_NUDGE_UP),
        CommandMeta::new("Nudge Up")
            .with_category("Node Graph")
            .with_keywords(["nudge", "move", "up"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .repeatable(),
    );
    registry.register(
        CommandId::from(CMD_NODE_GRAPH_NUDGE_DOWN),
        CommandMeta::new("Nudge Down")
            .with_category("Node Graph")
            .with_keywords(["nudge", "move", "down"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .repeatable(),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_NUDGE_LEFT_FAST),
        CommandMeta::new("Nudge Left (Fast)")
            .with_category("Node Graph")
            .with_keywords(["nudge", "move", "left", "fast"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .repeatable(),
    );
    registry.register(
        CommandId::from(CMD_NODE_GRAPH_NUDGE_RIGHT_FAST),
        CommandMeta::new("Nudge Right (Fast)")
            .with_category("Node Graph")
            .with_keywords(["nudge", "move", "right", "fast"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .repeatable(),
    );
    registry.register(
        CommandId::from(CMD_NODE_GRAPH_NUDGE_UP_FAST),
        CommandMeta::new("Nudge Up (Fast)")
            .with_category("Node Graph")
            .with_keywords(["nudge", "move", "up", "fast"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .repeatable(),
    );
    registry.register(
        CommandId::from(CMD_NODE_GRAPH_NUDGE_DOWN_FAST),
        CommandMeta::new("Nudge Down (Fast)")
            .with_category("Node Graph")
            .with_keywords(["nudge", "move", "down", "fast"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .repeatable(),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_ALIGN_LEFT),
        CommandMeta::new("Align Left")
            .with_category("Node Graph")
            .with_keywords(["align", "left"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );
    registry.register(
        CommandId::from(CMD_NODE_GRAPH_ALIGN_RIGHT),
        CommandMeta::new("Align Right")
            .with_category("Node Graph")
            .with_keywords(["align", "right"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );
    registry.register(
        CommandId::from(CMD_NODE_GRAPH_ALIGN_TOP),
        CommandMeta::new("Align Top")
            .with_category("Node Graph")
            .with_keywords(["align", "top"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );
    registry.register(
        CommandId::from(CMD_NODE_GRAPH_ALIGN_BOTTOM),
        CommandMeta::new("Align Bottom")
            .with_category("Node Graph")
            .with_keywords(["align", "bottom"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );
    registry.register(
        CommandId::from(CMD_NODE_GRAPH_ALIGN_CENTER_X),
        CommandMeta::new("Align Center X")
            .with_category("Node Graph")
            .with_keywords(["align", "center", "x"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );
    registry.register(
        CommandId::from(CMD_NODE_GRAPH_ALIGN_CENTER_Y),
        CommandMeta::new("Align Center Y")
            .with_category("Node Graph")
            .with_keywords(["align", "center", "y"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );
    registry.register(
        CommandId::from(CMD_NODE_GRAPH_DISTRIBUTE_X),
        CommandMeta::new("Distribute X")
            .with_category("Node Graph")
            .with_keywords(["distribute", "x", "horizontal"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );
    registry.register(
        CommandId::from(CMD_NODE_GRAPH_DISTRIBUTE_Y),
        CommandMeta::new("Distribute Y")
            .with_category("Node Graph")
            .with_keywords(["distribute", "y", "vertical"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT),
        CommandMeta::new("Focus Next Node")
            .with_category("Node Graph")
            .with_keywords(["focus", "next", "node", "tab"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .with_default_keybindings([kb(PlatformFilter::All, KeyCode::Tab, Modifiers::default())])
            .repeatable(),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_FOCUS_PREV),
        CommandMeta::new("Focus Previous Node")
            .with_category("Node Graph")
            .with_keywords(["focus", "previous", "node", "tab"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .with_default_keybindings([kb(
                PlatformFilter::All,
                KeyCode::Tab,
                Modifiers {
                    shift: true,
                    ..Default::default()
                },
            )])
            .repeatable(),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_FOCUS_NEXT_EDGE),
        CommandMeta::new("Focus Next Edge")
            .with_category("Node Graph")
            .with_keywords(["focus", "next", "edge"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .repeatable(),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_FOCUS_PREV_EDGE),
        CommandMeta::new("Focus Previous Edge")
            .with_category("Node Graph")
            .with_keywords(["focus", "previous", "edge"])
            .with_scope(widget)
            .with_when(when_node_graph_editing())
            .repeatable(),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_FRAME_SELECTION),
        CommandMeta::new("Frame Selection")
            .with_category("Node Graph")
            .with_keywords(["frame", "focus", "fit", "view"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_FRAME_ALL),
        CommandMeta::new("Frame All")
            .with_category("Node Graph")
            .with_keywords(["frame", "focus", "fit", "view", "all"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_RESET_VIEW),
        CommandMeta::new("Reset View")
            .with_category("Node Graph")
            .with_keywords(["reset", "view", "pan", "zoom"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_ZOOM_IN),
        CommandMeta::new("Zoom In")
            .with_category("Node Graph")
            .with_keywords(["zoom", "in"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_ZOOM_OUT),
        CommandMeta::new("Zoom Out")
            .with_category("Node Graph")
            .with_keywords(["zoom", "out"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );

    registry.register(
        CommandId::from(CMD_NODE_GRAPH_TOGGLE_CONNECTION_MODE),
        CommandMeta::new("Toggle Connection Mode")
            .with_category("Node Graph")
            .with_keywords(["connection", "mode", "strict", "loose"])
            .with_scope(widget)
            .with_when(when_node_graph_editing()),
    );
}
