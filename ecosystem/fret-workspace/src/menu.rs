use std::sync::Arc;

use fret_runtime::{CommandId, Menu, MenuBar, MenuItem, MenuRole, SystemMenuType};

use crate::commands::{act, typed_command_id};

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
    /// Optional override for the top-level "File" menu title.
    pub file_menu_title: Option<Arc<str>>,
    /// Optional override for the top-level "Edit" menu title.
    pub edit_menu_title: Option<Arc<str>>,
    /// Optional override for the top-level "View" menu title.
    pub view_menu_title: Option<Arc<str>>,
    /// Optional override for the top-level "Window" menu title.
    pub window_menu_title: Option<Arc<str>>,

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

    /// Optional router navigation commands for editor-style apps.
    ///
    /// When set, `workspace_default_menu_bar` includes a "Navigate" menu with Back/Forward.
    pub router_back: Option<CommandId>,
    pub router_forward: Option<CommandId>,

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
            file_menu_title: None,
            edit_menu_title: None,
            view_menu_title: None,
            window_menu_title: None,

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

            router_back: None,
            router_forward: None,

            next_tab: typed_command_id::<act::WorkspaceTabNext>(),
            prev_tab: typed_command_id::<act::WorkspaceTabPrev>(),
            close_tab: typed_command_id::<act::WorkspaceTabClose>(),

            next_pane: typed_command_id::<act::WorkspacePaneNext>(),
            prev_pane: typed_command_id::<act::WorkspacePanePrev>(),

            split_right: typed_command_id::<act::WorkspacePaneSplitRight>(),
            split_left: typed_command_id::<act::WorkspacePaneSplitLeft>(),
            split_up: typed_command_id::<act::WorkspacePaneSplitUp>(),
            split_down: typed_command_id::<act::WorkspacePaneSplitDown>(),

            move_active_tab_next_pane: typed_command_id::<act::WorkspacePaneMoveActiveTabNext>(),
            move_active_tab_prev_pane: typed_command_id::<act::WorkspacePaneMoveActiveTabPrev>(),

            resize_pane_right: typed_command_id::<act::WorkspacePaneResizeRight>(),
            resize_pane_left: typed_command_id::<act::WorkspacePaneResizeLeft>(),
            resize_pane_up: typed_command_id::<act::WorkspacePaneResizeUp>(),
            resize_pane_down: typed_command_id::<act::WorkspacePaneResizeDown>(),
        }
    }
}

fn push_command(items: &mut Vec<MenuItem>, command: Option<CommandId>) {
    if let Some(command) = command {
        items.push(MenuItem::Command {
            command,
            when: None,
            toggle: None,
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
        mnemonic: None,
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
        file_menu_title,
        edit_menu_title,
        view_menu_title,
        window_menu_title,
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
        router_back,
        router_forward,
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
    if !file_items.is_empty() {
        push_separator(&mut file_items);
        file_items.push(MenuItem::Submenu {
            title: Arc::from("Recent"),
            when: None,
            items: vec![MenuItem::Label {
                title: Arc::from("No recent items"),
            }],
        });
    }
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
            title: file_menu_title.unwrap_or_else(|| Arc::from("File")),
            role: Some(MenuRole::File),
            mnemonic: Some('f'),
            items: file_items,
        });
    }
    if !edit_items.is_empty() {
        menus.push(Menu {
            title: edit_menu_title.unwrap_or_else(|| Arc::from("Edit")),
            role: Some(MenuRole::Edit),
            mnemonic: Some('e'),
            items: edit_items,
        });
    }
    if !view_items.is_empty() {
        menus.push(Menu {
            title: view_menu_title.unwrap_or_else(|| Arc::from("View")),
            role: Some(MenuRole::View),
            mnemonic: Some('v'),
            items: view_items,
        });
    }

    if router_back.is_some() || router_forward.is_some() {
        let mut nav_items = Vec::new();
        push_command(&mut nav_items, router_back);
        push_command(&mut nav_items, router_forward);

        if !nav_items.is_empty() {
            menus.push(Menu {
                title: Arc::from("Navigate"),
                role: None,
                mnemonic: Some('n'),
                items: nav_items,
            });
        }
    }

    menus.push(Menu {
        title: window_menu_title.unwrap_or_else(|| Arc::from("Window")),
        role: Some(MenuRole::Window),
        mnemonic: Some('w'),
        items: vec![
            MenuItem::Command {
                command: next_tab,
                when: None,
                toggle: None,
            },
            MenuItem::Command {
                command: prev_tab,
                when: None,
                toggle: None,
            },
            MenuItem::Separator,
            MenuItem::Command {
                command: typed_command_id::<act::WorkspaceTabMoveLeft>(),
                when: None,
                toggle: None,
            },
            MenuItem::Command {
                command: typed_command_id::<act::WorkspaceTabMoveRight>(),
                when: None,
                toggle: None,
            },
            MenuItem::Separator,
            MenuItem::Command {
                command: close_tab,
                when: None,
                toggle: None,
            },
            MenuItem::Command {
                command: typed_command_id::<act::WorkspaceTabCloseOthers>(),
                when: None,
                toggle: None,
            },
            MenuItem::Command {
                command: typed_command_id::<act::WorkspaceTabCloseLeft>(),
                when: None,
                toggle: None,
            },
            MenuItem::Command {
                command: typed_command_id::<act::WorkspaceTabCloseRight>(),
                when: None,
                toggle: None,
            },
            MenuItem::Separator,
            MenuItem::Command {
                command: next_pane,
                when: None,
                toggle: None,
            },
            MenuItem::Command {
                command: prev_pane,
                when: None,
                toggle: None,
            },
            MenuItem::Separator,
            MenuItem::Submenu {
                title: Arc::from("Split"),
                when: None,
                items: vec![
                    MenuItem::Command {
                        command: split_right,
                        when: None,
                        toggle: None,
                    },
                    MenuItem::Command {
                        command: split_left,
                        when: None,
                        toggle: None,
                    },
                    MenuItem::Command {
                        command: split_up,
                        when: None,
                        toggle: None,
                    },
                    MenuItem::Command {
                        command: split_down,
                        when: None,
                        toggle: None,
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
                        toggle: None,
                    },
                    MenuItem::Command {
                        command: move_active_tab_prev_pane,
                        when: None,
                        toggle: None,
                    },
                    MenuItem::Separator,
                    MenuItem::Command {
                        command: typed_command_id::<act::WorkspacePaneMoveActiveTabLeft>(),
                        when: None,
                        toggle: None,
                    },
                    MenuItem::Command {
                        command: typed_command_id::<act::WorkspacePaneMoveActiveTabRight>(),
                        when: None,
                        toggle: None,
                    },
                    MenuItem::Command {
                        command: typed_command_id::<act::WorkspacePaneMoveActiveTabUp>(),
                        when: None,
                        toggle: None,
                    },
                    MenuItem::Command {
                        command: typed_command_id::<act::WorkspacePaneMoveActiveTabDown>(),
                        when: None,
                        toggle: None,
                    },
                ],
            },
            MenuItem::Submenu {
                title: Arc::from("Focus Pane"),
                when: None,
                items: vec![
                    MenuItem::Command {
                        command: typed_command_id::<act::WorkspacePaneFocusLeft>(),
                        when: None,
                        toggle: None,
                    },
                    MenuItem::Command {
                        command: typed_command_id::<act::WorkspacePaneFocusRight>(),
                        when: None,
                        toggle: None,
                    },
                    MenuItem::Command {
                        command: typed_command_id::<act::WorkspacePaneFocusUp>(),
                        when: None,
                        toggle: None,
                    },
                    MenuItem::Command {
                        command: typed_command_id::<act::WorkspacePaneFocusDown>(),
                        when: None,
                        toggle: None,
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
                        toggle: None,
                    },
                    MenuItem::Command {
                        command: resize_pane_left,
                        when: None,
                        toggle: None,
                    },
                    MenuItem::Command {
                        command: resize_pane_up,
                        when: None,
                        toggle: None,
                    },
                    MenuItem::Command {
                        command: resize_pane_down,
                        when: None,
                        toggle: None,
                    },
                ],
            },
            MenuItem::Separator,
            MenuItem::Submenu {
                title: Arc::from("Windows"),
                when: None,
                items: vec![MenuItem::Label {
                    title: Arc::from("Window list not implemented"),
                }],
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

    #[test]
    fn workspace_default_menu_uses_custom_view_title_when_provided() {
        let mut cmds = WorkspaceMenuCommands::default();
        cmds.command_palette = Some(CommandId::new("app.command_palette"));
        cmds.view_menu_title = Some(Arc::from("视图"));

        let menu_bar = workspace_default_menu_bar(cmds);
        let view_menu = menu_bar
            .menus
            .iter()
            .find(|menu| menu.role == Some(MenuRole::View))
            .expect("view menu should be present");

        assert_eq!(view_menu.title.as_ref(), "视图");
    }

    #[test]
    fn workspace_default_menu_uses_custom_file_edit_window_titles_when_provided() {
        let mut cmds = WorkspaceMenuCommands::default();
        cmds.open = Some(CommandId::new("app.open"));
        cmds.undo = Some(CommandId::new("edit.undo"));
        cmds.file_menu_title = Some(Arc::from("文件"));
        cmds.edit_menu_title = Some(Arc::from("编辑"));
        cmds.window_menu_title = Some(Arc::from("窗口"));

        let menu_bar = workspace_default_menu_bar(cmds);

        let file_menu = menu_bar
            .menus
            .iter()
            .find(|menu| menu.role == Some(MenuRole::File))
            .expect("file menu should be present");
        assert_eq!(file_menu.title.as_ref(), "文件");

        let edit_menu = menu_bar
            .menus
            .iter()
            .find(|menu| menu.role == Some(MenuRole::Edit))
            .expect("edit menu should be present");
        assert_eq!(edit_menu.title.as_ref(), "编辑");

        let window_menu = menu_bar
            .menus
            .iter()
            .find(|menu| menu.role == Some(MenuRole::Window))
            .expect("window menu should be present");
        assert_eq!(window_menu.title.as_ref(), "窗口");
    }

    #[test]
    fn workspace_default_menu_includes_router_navigation_menu_when_configured() {
        let mut cmds = WorkspaceMenuCommands::default();
        cmds.router_back = Some(CommandId::new("router.back"));
        cmds.router_forward = Some(CommandId::new("router.forward"));

        let menu_bar = workspace_default_menu_bar(cmds);
        let navigate_menu = menu_bar
            .menus
            .iter()
            .find(|menu| menu.title.as_ref() == "Navigate")
            .expect("navigate menu should be present");

        assert!(
            navigate_menu.items.iter().any(|item| {
                matches!(item, MenuItem::Command { command, .. } if command == &CommandId::new("router.back"))
            }),
            "navigate menu should contain router.back"
        );
        assert!(
            navigate_menu.items.iter().any(|item| {
                matches!(item, MenuItem::Command { command, .. } if command == &CommandId::new("router.forward"))
            }),
            "navigate menu should contain router.forward"
        );
    }
}
