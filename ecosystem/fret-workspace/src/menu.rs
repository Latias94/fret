use std::sync::Arc;

use fret_runtime::{CommandId, Menu, MenuBar, MenuItem};

/// Command IDs used by `workspace_default_menu_bar`.
///
/// This keeps `fret-workspace` independent from `fret-app` core command constants.
#[derive(Debug, Clone)]
pub struct WorkspaceMenuCommands {
    pub command_palette: Option<CommandId>,

    pub open: Option<CommandId>,
    pub save: Option<CommandId>,
    pub save_as: Option<CommandId>,
    pub quit: Option<CommandId>,

    pub undo: Option<CommandId>,
    pub redo: Option<CommandId>,
    pub cut: Option<CommandId>,
    pub copy: Option<CommandId>,
    pub paste: Option<CommandId>,
    pub select_all: Option<CommandId>,

    pub next_tab: CommandId,
    pub prev_tab: CommandId,
    pub close_tab: CommandId,

    pub next_pane: CommandId,
    pub prev_pane: CommandId,
}

impl Default for WorkspaceMenuCommands {
    fn default() -> Self {
        Self {
            command_palette: None,

            open: None,
            save: None,
            save_as: None,
            quit: None,

            undo: None,
            redo: None,
            cut: None,
            copy: None,
            paste: None,
            select_all: None,

            next_tab: CommandId::new(crate::commands::CMD_WORKSPACE_TAB_NEXT),
            prev_tab: CommandId::new(crate::commands::CMD_WORKSPACE_TAB_PREV),
            close_tab: CommandId::new(crate::commands::CMD_WORKSPACE_TAB_CLOSE),

            next_pane: CommandId::new(crate::commands::CMD_WORKSPACE_PANE_NEXT),
            prev_pane: CommandId::new(crate::commands::CMD_WORKSPACE_PANE_PREV),
        }
    }
}

fn push_command(items: &mut Vec<MenuItem>, command: Option<CommandId>) {
    if let Some(command) = command {
        items.push(MenuItem::Command {
            command,
            when: None,
        });
    }
}

/// A minimal editor-style menu bar for workspace shells.
///
/// Notes:
/// - Menus are data-only (`fret-runtime`) and can be rendered by any UI surface.
/// - Apps can extend/replace this entirely; this is a "golden path" starting point.
pub fn workspace_default_menu_bar(cmds: WorkspaceMenuCommands) -> MenuBar {
    let mut file_items = Vec::new();
    push_command(&mut file_items, cmds.open);
    push_command(&mut file_items, cmds.save);
    push_command(&mut file_items, cmds.save_as);
    if cmds.quit.is_some() && !file_items.is_empty() {
        file_items.push(MenuItem::Separator);
    }
    push_command(&mut file_items, cmds.quit);

    let mut edit_items = Vec::new();
    push_command(&mut edit_items, cmds.undo);
    push_command(&mut edit_items, cmds.redo);
    if (cmds.cut.is_some() || cmds.copy.is_some() || cmds.paste.is_some()) && !edit_items.is_empty()
    {
        edit_items.push(MenuItem::Separator);
    }
    push_command(&mut edit_items, cmds.cut);
    push_command(&mut edit_items, cmds.copy);
    push_command(&mut edit_items, cmds.paste);
    if cmds.select_all.is_some() && !edit_items.is_empty() {
        edit_items.push(MenuItem::Separator);
    }
    push_command(&mut edit_items, cmds.select_all);

    let mut view_items = Vec::new();
    if let Some(cp) = cmds.command_palette {
        view_items.push(MenuItem::Command {
            command: cp,
            when: None,
        });
    }

    let mut menus = Vec::new();
    if !file_items.is_empty() {
        menus.push(Menu {
            title: Arc::from("File"),
            items: file_items,
        });
    }
    if !edit_items.is_empty() {
        menus.push(Menu {
            title: Arc::from("Edit"),
            items: edit_items,
        });
    }
    if !view_items.is_empty() {
        menus.push(Menu {
            title: Arc::from("View"),
            items: view_items,
        });
    }

    menus.push(Menu {
        title: Arc::from("Window"),
        items: vec![
            MenuItem::Command {
                command: cmds.next_tab,
                when: None,
            },
            MenuItem::Command {
                command: cmds.prev_tab,
                when: None,
            },
            MenuItem::Separator,
            MenuItem::Command {
                command: cmds.close_tab,
                when: None,
            },
            MenuItem::Separator,
            MenuItem::Command {
                command: cmds.next_pane,
                when: None,
            },
            MenuItem::Command {
                command: cmds.prev_pane,
                when: None,
            },
        ],
    });

    MenuBar { menus }
}
