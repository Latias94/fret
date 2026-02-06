use std::sync::Arc;

use fret_runtime::{CommandId, Menu, MenuBar, MenuItem, MenuRole, SystemMenuType};

/// Command IDs used by `workspace_default_menu_bar`.
///
/// This keeps `fret-workspace` independent from `fret-app` core command constants.
#[derive(Debug, Clone)]
pub struct WorkspaceMenuCommands {
    /// Optional app menu title for macOS `MenuRole::App` (e.g. "Fret", "MyApp").
    ///
    /// If not provided, defaults to "App".
    pub app_menu_title: Option<Arc<str>>,
    /// Include a Services system menu in the app menu (macOS only; ignored elsewhere).
    pub include_services_menu: bool,
    pub about: Option<CommandId>,
    pub preferences: Option<CommandId>,
    pub hide: Option<CommandId>,
    pub hide_others: Option<CommandId>,
    pub show_all: Option<CommandId>,
    pub quit_app: Option<CommandId>,

    pub command_palette: Option<CommandId>,
    pub switch_locale: Option<CommandId>,

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

    pub split_right: CommandId,
    pub split_left: CommandId,
    pub split_up: CommandId,
    pub split_down: CommandId,

    pub move_active_tab_next_pane: CommandId,
    pub move_active_tab_prev_pane: CommandId,

    pub resize_pane_right: CommandId,
    pub resize_pane_left: CommandId,
    pub resize_pane_up: CommandId,
    pub resize_pane_down: CommandId,
}

impl Default for WorkspaceMenuCommands {
    fn default() -> Self {
        Self {
            app_menu_title: None,
            include_services_menu: false,
            about: None,
            preferences: None,
            hide: None,
            hide_others: None,
            show_all: None,
            quit_app: None,

            command_palette: None,
            switch_locale: None,

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

            split_right: CommandId::new(crate::commands::CMD_WORKSPACE_PANE_SPLIT_RIGHT),
            split_left: CommandId::new(crate::commands::CMD_WORKSPACE_PANE_SPLIT_LEFT),
            split_up: CommandId::new(crate::commands::CMD_WORKSPACE_PANE_SPLIT_UP),
            split_down: CommandId::new(crate::commands::CMD_WORKSPACE_PANE_SPLIT_DOWN),

            move_active_tab_next_pane: CommandId::new(
                crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_NEXT,
            ),
            move_active_tab_prev_pane: CommandId::new(
                crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_PREV,
            ),

            resize_pane_right: CommandId::new(crate::commands::CMD_WORKSPACE_PANE_RESIZE_RIGHT),
            resize_pane_left: CommandId::new(crate::commands::CMD_WORKSPACE_PANE_RESIZE_LEFT),
            resize_pane_up: CommandId::new(crate::commands::CMD_WORKSPACE_PANE_RESIZE_UP),
            resize_pane_down: CommandId::new(crate::commands::CMD_WORKSPACE_PANE_RESIZE_DOWN),
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

fn push_separator(items: &mut Vec<MenuItem>) {
    if items
        .last()
        .is_some_and(|i| matches!(i, MenuItem::Separator))
    {
        return;
    }
    items.push(MenuItem::Separator);
}

fn trim_trailing_separators(items: &mut Vec<MenuItem>) {
    while items
        .last()
        .is_some_and(|i| matches!(i, MenuItem::Separator))
    {
        items.pop();
    }
}

fn build_app_menu(cmds: &WorkspaceMenuCommands) -> Option<Menu> {
    if cmds.about.is_none()
        && cmds.preferences.is_none()
        && cmds.hide.is_none()
        && cmds.hide_others.is_none()
        && cmds.show_all.is_none()
        && cmds.quit_app.is_none()
        && !cmds.include_services_menu
    {
        return None;
    }

    let title = cmds
        .app_menu_title
        .clone()
        .unwrap_or_else(|| Arc::<str>::from("App"));

    let mut items = Vec::new();
    push_command(&mut items, cmds.about.clone());

    if cmds.preferences.is_some() {
        if !items.is_empty() {
            push_separator(&mut items);
        }
        push_command(&mut items, cmds.preferences.clone());
    }

    if cmds.include_services_menu {
        if !items.is_empty() {
            push_separator(&mut items);
        }
        items.push(MenuItem::SystemMenu {
            title: Arc::from("Services"),
            menu_type: SystemMenuType::Services,
        });
    }

    if cmds.hide.is_some() || cmds.hide_others.is_some() || cmds.show_all.is_some() {
        if !items.is_empty() {
            push_separator(&mut items);
        }
        push_command(&mut items, cmds.hide.clone());
        push_command(&mut items, cmds.hide_others.clone());
        push_command(&mut items, cmds.show_all.clone());
    }

    if cmds.quit_app.is_some() {
        if !items.is_empty() {
            push_separator(&mut items);
        }
        push_command(&mut items, cmds.quit_app.clone());
    }

    trim_trailing_separators(&mut items);
    if items.is_empty() {
        return None;
    }

    Some(Menu {
        title,
        role: Some(MenuRole::App),
        items,
    })
}

/// A minimal editor-style menu bar for workspace shells.
///
/// Notes:
/// - Menus are data-only (`fret-runtime`) and can be rendered by any UI surface.
/// - Apps can extend/replace this entirely; this is a "golden path" starting point.
pub fn workspace_default_menu_bar(cmds: WorkspaceMenuCommands) -> MenuBar {
    let app_menu = build_app_menu(&cmds);

    let WorkspaceMenuCommands {
        app_menu_title: _,
        include_services_menu: _,
        about: _,
        preferences: _,
        hide: _,
        hide_others: _,
        show_all: _,
        quit_app: _,
        command_palette,
        switch_locale,
        open,
        save,
        save_as,
        quit,
        undo,
        redo,
        cut,
        copy,
        paste,
        select_all,
        next_tab,
        prev_tab,
        close_tab,
        next_pane,
        prev_pane,
        split_right,
        split_left,
        split_up,
        split_down,
        move_active_tab_next_pane,
        move_active_tab_prev_pane,
        resize_pane_right,
        resize_pane_left,
        resize_pane_up,
        resize_pane_down,
    } = cmds;

    let mut file_items = Vec::new();
    push_command(&mut file_items, open);
    push_command(&mut file_items, save);
    push_command(&mut file_items, save_as);
    if quit.is_some() && !file_items.is_empty() {
        file_items.push(MenuItem::Separator);
    }
    push_command(&mut file_items, quit);

    let mut edit_items = Vec::new();
    push_command(&mut edit_items, undo);
    push_command(&mut edit_items, redo);
    if (cut.is_some() || copy.is_some() || paste.is_some()) && !edit_items.is_empty() {
        edit_items.push(MenuItem::Separator);
    }
    push_command(&mut edit_items, cut);
    push_command(&mut edit_items, copy);
    push_command(&mut edit_items, paste);
    if select_all.is_some() && !edit_items.is_empty() {
        edit_items.push(MenuItem::Separator);
    }
    push_command(&mut edit_items, select_all);

    let mut view_items = Vec::new();
    push_command(&mut view_items, command_palette);
    push_command(&mut view_items, switch_locale);

    let mut menus = Vec::new();
    if let Some(app_menu) = app_menu {
        menus.push(app_menu);
    }
    if !file_items.is_empty() {
        menus.push(Menu {
            title: Arc::from("File"),
            role: Some(MenuRole::File),
            items: file_items,
        });
    }
    if !edit_items.is_empty() {
        menus.push(Menu {
            title: Arc::from("Edit"),
            role: Some(MenuRole::Edit),
            items: edit_items,
        });
    }
    if !view_items.is_empty() {
        menus.push(Menu {
            title: Arc::from("View"),
            role: Some(MenuRole::View),
            items: view_items,
        });
    }

    menus.push(Menu {
        title: Arc::from("Window"),
        role: Some(MenuRole::Window),
        items: vec![
            MenuItem::Command {
                command: next_tab,
                when: None,
            },
            MenuItem::Command {
                command: prev_tab,
                when: None,
            },
            MenuItem::Separator,
            MenuItem::Command {
                command: CommandId::new(crate::commands::CMD_WORKSPACE_TAB_MOVE_LEFT),
                when: None,
            },
            MenuItem::Command {
                command: CommandId::new(crate::commands::CMD_WORKSPACE_TAB_MOVE_RIGHT),
                when: None,
            },
            MenuItem::Separator,
            MenuItem::Command {
                command: close_tab,
                when: None,
            },
            MenuItem::Command {
                command: CommandId::new(crate::commands::CMD_WORKSPACE_TAB_CLOSE_OTHERS),
                when: None,
            },
            MenuItem::Command {
                command: CommandId::new(crate::commands::CMD_WORKSPACE_TAB_CLOSE_LEFT),
                when: None,
            },
            MenuItem::Command {
                command: CommandId::new(crate::commands::CMD_WORKSPACE_TAB_CLOSE_RIGHT),
                when: None,
            },
            MenuItem::Separator,
            MenuItem::Command {
                command: next_pane,
                when: None,
            },
            MenuItem::Command {
                command: prev_pane,
                when: None,
            },
            MenuItem::Separator,
            MenuItem::Submenu {
                title: Arc::from("Split"),
                when: None,
                items: vec![
                    MenuItem::Command {
                        command: split_right,
                        when: None,
                    },
                    MenuItem::Command {
                        command: split_left,
                        when: None,
                    },
                    MenuItem::Command {
                        command: split_up,
                        when: None,
                    },
                    MenuItem::Command {
                        command: split_down,
                        when: None,
                    },
                ],
            },
            MenuItem::Submenu {
                title: Arc::from("Move Tab"),
                when: None,
                items: vec![
                    MenuItem::Command {
                        command: move_active_tab_next_pane,
                        when: None,
                    },
                    MenuItem::Command {
                        command: move_active_tab_prev_pane,
                        when: None,
                    },
                    MenuItem::Separator,
                    MenuItem::Command {
                        command: CommandId::new(
                            crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_LEFT,
                        ),
                        when: None,
                    },
                    MenuItem::Command {
                        command: CommandId::new(
                            crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_RIGHT,
                        ),
                        when: None,
                    },
                    MenuItem::Command {
                        command: CommandId::new(
                            crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_UP,
                        ),
                        when: None,
                    },
                    MenuItem::Command {
                        command: CommandId::new(
                            crate::commands::CMD_WORKSPACE_PANE_MOVE_ACTIVE_TAB_DOWN,
                        ),
                        when: None,
                    },
                ],
            },
            MenuItem::Submenu {
                title: Arc::from("Focus Pane"),
                when: None,
                items: vec![
                    MenuItem::Command {
                        command: CommandId::new(crate::commands::CMD_WORKSPACE_PANE_FOCUS_LEFT),
                        when: None,
                    },
                    MenuItem::Command {
                        command: CommandId::new(crate::commands::CMD_WORKSPACE_PANE_FOCUS_RIGHT),
                        when: None,
                    },
                    MenuItem::Command {
                        command: CommandId::new(crate::commands::CMD_WORKSPACE_PANE_FOCUS_UP),
                        when: None,
                    },
                    MenuItem::Command {
                        command: CommandId::new(crate::commands::CMD_WORKSPACE_PANE_FOCUS_DOWN),
                        when: None,
                    },
                ],
            },
            MenuItem::Submenu {
                title: Arc::from("Resize"),
                when: None,
                items: vec![
                    MenuItem::Command {
                        command: resize_pane_right,
                        when: None,
                    },
                    MenuItem::Command {
                        command: resize_pane_left,
                        when: None,
                    },
                    MenuItem::Command {
                        command: resize_pane_up,
                        when: None,
                    },
                    MenuItem::Command {
                        command: resize_pane_down,
                        when: None,
                    },
                ],
            },
        ],
    });

    MenuBar { menus }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_default_menu_includes_locale_switch_command_in_view_menu() {
        let mut cmds = WorkspaceMenuCommands::default();
        cmds.command_palette = Some(CommandId::new("app.command_palette"));
        cmds.switch_locale = Some(CommandId::new("app.locale.switch_next"));

        let menu_bar = workspace_default_menu_bar(cmds);
        let view_menu = menu_bar
            .menus
            .iter()
            .find(|menu| menu.role == Some(MenuRole::View))
            .expect("view menu should be present");

        assert!(
            view_menu.items.iter().any(|item| {
                matches!(
                    item,
                    MenuItem::Command { command, .. }
                        if command == &CommandId::new("app.locale.switch_next")
                )
            }),
            "view menu should contain locale switch command"
        );
    }
}
