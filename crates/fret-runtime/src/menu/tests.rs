use std::sync::Arc;

use crate::CommandId;

use super::*;

fn base_menu_bar() -> MenuBar {
    MenuBar {
        menus: vec![Menu {
            title: Arc::from("File"),
            role: None,
            mnemonic: None,
            items: vec![
                MenuItem::Command {
                    command: CommandId::new("app.open"),
                    when: None,
                    toggle: None,
                },
                MenuItem::Submenu {
                    title: Arc::from("Recent"),
                    when: None,
                    items: vec![MenuItem::Separator],
                },
            ],
        }],
    }
}

#[test]
fn normalize_removes_leading_trailing_and_duplicate_separators() {
    let mut bar = MenuBar {
        menus: vec![Menu {
            title: Arc::from("File"),
            role: None,
            mnemonic: None,
            items: vec![
                MenuItem::Separator,
                MenuItem::Separator,
                MenuItem::Command {
                    command: CommandId::new("app.open"),
                    when: None,
                    toggle: None,
                },
                MenuItem::Separator,
                MenuItem::Separator,
                MenuItem::SystemMenu {
                    title: Arc::from("Services"),
                    menu_type: SystemMenuType::Services,
                },
                MenuItem::Separator,
            ],
        }],
    };

    bar.normalize();

    assert_eq!(bar.menus.len(), 1);
    assert!(matches!(
        bar.menus[0].items.as_slice(),
        [
            MenuItem::Command { .. },
            MenuItem::Separator,
            MenuItem::SystemMenu { .. }
        ]
    ));
}

#[test]
fn normalize_drops_empty_submenus_recursively() {
    let mut bar = MenuBar {
        menus: vec![Menu {
            title: Arc::from("File"),
            role: None,
            mnemonic: None,
            items: vec![
                MenuItem::Submenu {
                    title: Arc::from("Empty"),
                    when: None,
                    items: vec![MenuItem::Separator, MenuItem::Separator],
                },
                MenuItem::Submenu {
                    title: Arc::from("NonEmpty"),
                    when: None,
                    items: vec![
                        MenuItem::Submenu {
                            title: Arc::from("NestedEmpty"),
                            when: None,
                            items: vec![MenuItem::Separator],
                        },
                        MenuItem::Command {
                            command: CommandId::new("app.open"),
                            when: None,
                            toggle: None,
                        },
                    ],
                },
            ],
        }],
    };

    bar.normalize();

    let items = &bar.menus[0].items;
    assert_eq!(items.len(), 1);
    let MenuItem::Submenu { title, items, .. } = &items[0] else {
        panic!("expected submenu");
    };
    assert_eq!(title.as_ref(), "NonEmpty");
    assert!(items.iter().all(|i| !matches!(
        i,
        MenuItem::Submenu { title, .. } if title.as_ref() == "NestedEmpty"
    )));
}

#[test]
fn patch_can_target_submenu_by_path() {
    let mut base = base_menu_bar();
    let patch = MenuBarPatch {
        ops: vec![MenuBarPatchOp::AppendItem {
            menu: MenuTarget::Path(vec!["File".into(), "Recent".into()]),
            item: MenuItem::Command {
                command: CommandId::new("app.open_recent"),
                when: None,
                toggle: None,
            },
        }],
    };

    patch.apply_to(&mut base).unwrap();

    let recent_items = match &base.menus[0].items[1] {
        MenuItem::Submenu { items, .. } => items,
        other => panic!("expected submenu, got {other:?}"),
    };

    assert!(recent_items.iter().any(|item| match item {
        MenuItem::Command { command, .. } => command.as_str() == "app.open_recent",
        _ => false,
    }));
}

#[test]
fn insert_item_before_can_use_index_anchor() {
    let mut base = base_menu_bar();
    let patch = MenuBarPatch {
        ops: vec![MenuBarPatchOp::InsertItemBefore {
            menu: MenuTarget::Title("File".into()),
            before: ItemAnchor::Index(0),
            item: MenuItem::Command {
                command: CommandId::new("app.new"),
                when: None,
                toggle: None,
            },
        }],
    };
    patch.apply_to(&mut base).unwrap();

    match &base.menus[0].items[0] {
        MenuItem::Command { command, .. } => assert_eq!(command.as_str(), "app.new"),
        other => panic!("expected command, got {other:?}"),
    }
}

#[test]
fn config_file_allows_menu_as_string_or_path_array() {
    let json = serde_json::json!({
        "menu_bar_version": 1,
        "ops": [
            { "type": "append_item", "menu": "File", "item": { "type": "separator" } },
            { "type": "append_item", "menu": ["File", "Recent"], "item": { "type": "separator" } }
        ]
    });

    let bytes = serde_json::to_vec(&json).unwrap();
    let cfg = MenuBarConfig::from_bytes(&bytes).unwrap();
    match cfg {
        MenuBarConfig::Patch(patch) => assert_eq!(patch.ops.len(), 2),
        other => panic!("expected patch, got {other:?}"),
    }
}

#[test]
fn remove_at_can_remove_submenu_by_title() {
    let mut base = base_menu_bar();
    let patch = MenuBarPatch {
        ops: vec![MenuBarPatchOp::RemoveAt {
            menu: MenuTarget::Title("File".into()),
            at: ItemSelector::Typed(ItemSelectorTyped::Submenu {
                title: "Recent".into(),
            }),
        }],
    };

    patch.apply_to(&mut base).unwrap();
    assert!(!base.menus[0].items.iter().any(|item| matches!(
        item,
        MenuItem::Submenu { title, .. } if title.as_ref() == "Recent"
    )));
}

#[test]
fn remove_at_can_remove_label_by_title() {
    let mut base = base_menu_bar();
    if let MenuItem::Submenu { items, .. } = &mut base.menus[0].items[1] {
        items.push(MenuItem::Label {
            title: Arc::from("No recent items"),
        });
    }

    let patch = MenuBarPatch {
        ops: vec![MenuBarPatchOp::RemoveAt {
            menu: MenuTarget::Path(vec!["File".into(), "Recent".into()]),
            at: ItemSelector::Typed(ItemSelectorTyped::Label {
                title: "No recent items".into(),
            }),
        }],
    };

    patch.apply_to(&mut base).unwrap();

    let recent_items = match &base.menus[0].items[1] {
        MenuItem::Submenu { items, .. } => items,
        other => panic!("expected submenu, got {other:?}"),
    };
    assert!(!recent_items.iter().any(|item| matches!(
        item,
        MenuItem::Label { title } if title.as_ref() == "No recent items"
    )));
}

#[test]
fn v2_replace_parses_roles_and_system_menus() {
    let json = serde_json::json!({
        "menu_bar_version": 2,
        "menus": [
            {
                "title": "Window",
                "role": "window",
                "items": [
                    { "type": "system_menu", "title": "Services", "system": "services" }
                ]
            }
        ]
    });

    let bytes = serde_json::to_vec(&json).unwrap();
    let bar = MenuBar::from_bytes(&bytes).unwrap();
    assert_eq!(bar.menus.len(), 1);
    assert_eq!(bar.menus[0].role, Some(MenuRole::Window));
    assert!(matches!(
        &bar.menus[0].items[0],
        MenuItem::SystemMenu {
            title,
            menu_type: SystemMenuType::Services
        } if title.as_ref() == "Services"
    ));
}

#[test]
fn v2_patch_ops_can_set_menu_role() {
    let json = serde_json::json!({
        "menu_bar_version": 2,
        "ops": [
            {
                "type": "append_menu",
                "title": "Help",
                "role": "help",
                "items": [
                    { "type": "command", "command": "app.command_palette" }
                ]
            }
        ]
    });

    let bytes = serde_json::to_vec(&json).unwrap();
    let cfg = MenuBarConfig::from_bytes(&bytes).unwrap();
    match cfg {
        MenuBarConfig::Patch(patch) => assert!(matches!(
            &patch.ops[0],
            MenuBarPatchOp::AppendMenu {
                role: Some(MenuRole::Help),
                ..
            }
        )),
        other => panic!("expected patch, got {other:?}"),
    }
}

#[test]
fn v2_replace_parses_label_items() {
    let json = serde_json::json!({
        "menu_bar_version": 2,
        "menus": [
            {
                "title": "File",
                "items": [
                    { "type": "label", "title": "No recent items" }
                ]
            }
        ]
    });

    let bytes = serde_json::to_vec(&json).unwrap();
    let bar = MenuBar::from_bytes(&bytes).unwrap();
    assert_eq!(bar.menus.len(), 1);
    assert!(matches!(
        &bar.menus[0].items[0],
        MenuItem::Label { title } if title.as_ref() == "No recent items"
    ));
}
