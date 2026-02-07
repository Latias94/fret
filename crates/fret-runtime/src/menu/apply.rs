use super::{
    ItemAnchor, ItemSelector, ItemSelectorTyped, Menu, MenuBar, MenuBarError, MenuBarPatch,
    MenuBarPatchOp, MenuItem, MenuTarget,
};

impl MenuBarPatch {
    pub fn apply_to(&self, base: &mut MenuBar) -> Result<(), MenuBarError> {
        for (idx, op) in self.ops.iter().cloned().enumerate() {
            apply_patch_op(base, idx, op)?;
        }
        Ok(())
    }
}

fn apply_patch_op(
    menu_bar: &mut MenuBar,
    index: usize,
    op: MenuBarPatchOp,
) -> Result<(), MenuBarError> {
    fn menu_index(menu_bar: &MenuBar, title: &str) -> Option<usize> {
        menu_bar
            .menus
            .iter()
            .position(|m| m.title.as_ref() == title)
    }

    fn resolve_menu_items_mut<'a>(
        menu_bar: &'a mut MenuBar,
        target: &MenuTarget,
    ) -> Option<&'a mut Vec<MenuItem>> {
        let path: Vec<&str> = match target {
            MenuTarget::Title(title) => vec![title.as_str()],
            MenuTarget::Path(parts) => parts.iter().map(|s| s.as_str()).collect(),
        };
        let (first, rest) = path.split_first()?;
        let menu = menu_bar
            .menus
            .iter_mut()
            .find(|m| m.title.as_ref() == *first)?;
        let mut items: *mut Vec<MenuItem> = &mut menu.items;

        for title in rest {
            // Safety: we only reborrow `items` inside the loop and never keep references across
            // iterations.
            let next = unsafe { &mut *items }
                .iter_mut()
                .find_map(|item| match item {
                    MenuItem::Submenu {
                        title: t, items, ..
                    } if t.as_ref() == *title => Some(items as *mut Vec<MenuItem>),
                    _ => None,
                })?;
            items = next;
        }

        // Safety: `items` points to the last submenu `items` vec (or top-level menu items).
        Some(unsafe { &mut *items })
    }

    fn resolve_anchor_index(items: &[MenuItem], anchor: &ItemAnchor) -> Option<usize> {
        match anchor {
            ItemAnchor::Index(i) => Some(*i),
            ItemAnchor::Command(cmd) => items.iter().position(|item| match item {
                MenuItem::Command { command, .. } => command.as_str() == cmd.as_str(),
                _ => false,
            }),
        }
    }

    fn resolve_selector_index(items: &[MenuItem], selector: &ItemSelector) -> Option<usize> {
        match selector {
            ItemSelector::Anchor(anchor) => resolve_anchor_index(items, anchor),
            ItemSelector::Typed(ItemSelectorTyped::Submenu { title }) => items.iter().position(
                |item| matches!(item, MenuItem::Submenu { title: t, .. } if t.as_ref() == title),
            ),
            ItemSelector::Typed(ItemSelectorTyped::Label { title }) => items.iter().position(
                |item| matches!(item, MenuItem::Label { title: t } if t.as_ref() == title),
            ),
        }
    }

    let fail = |error: String| Err(MenuBarError::PatchFailed { index, error });

    match op {
        MenuBarPatchOp::AppendMenu {
            title,
            role,
            mnemonic,
            items,
        } => {
            menu_bar.menus.push(Menu {
                title: title.into(),
                role,
                mnemonic,
                items,
            });
            Ok(())
        }
        MenuBarPatchOp::InsertMenuBefore {
            title,
            role,
            mnemonic,
            before,
            items,
        } => {
            let Some(before_idx) = menu_index(menu_bar, &before) else {
                return fail(format!("target menu not found: {before}"));
            };
            menu_bar.menus.insert(
                before_idx,
                Menu {
                    title: title.into(),
                    role,
                    mnemonic,
                    items,
                },
            );
            Ok(())
        }
        MenuBarPatchOp::InsertMenuAfter {
            title,
            role,
            mnemonic,
            after,
            items,
        } => {
            let Some(after_idx) = menu_index(menu_bar, &after) else {
                return fail(format!("target menu not found: {after}"));
            };
            let insert_at = (after_idx + 1).min(menu_bar.menus.len());
            menu_bar.menus.insert(
                insert_at,
                Menu {
                    title: title.into(),
                    role,
                    mnemonic,
                    items,
                },
            );
            Ok(())
        }
        MenuBarPatchOp::RemoveMenu { title } => {
            let Some(idx) = menu_index(menu_bar, &title) else {
                return fail(format!("menu not found: {title}"));
            };
            menu_bar.menus.remove(idx);
            Ok(())
        }
        MenuBarPatchOp::RenameMenu { from, to } => {
            let Some(idx) = menu_index(menu_bar, &from) else {
                return fail(format!("menu not found: {from}"));
            };
            menu_bar.menus[idx].title = to.into();
            Ok(())
        }
        MenuBarPatchOp::MoveMenuBefore { title, before } => {
            let Some(from_idx) = menu_index(menu_bar, &title) else {
                return fail(format!("menu not found: {title}"));
            };
            let Some(mut to_idx) = menu_index(menu_bar, &before) else {
                return fail(format!("target menu not found: {before}"));
            };
            let menu = menu_bar.menus.remove(from_idx);
            if from_idx < to_idx {
                to_idx = to_idx.saturating_sub(1);
            }
            menu_bar.menus.insert(to_idx, menu);
            Ok(())
        }
        MenuBarPatchOp::MoveMenuAfter { title, after } => {
            let Some(from_idx) = menu_index(menu_bar, &title) else {
                return fail(format!("menu not found: {title}"));
            };
            let Some(mut to_idx) = menu_index(menu_bar, &after) else {
                return fail(format!("target menu not found: {after}"));
            };
            let menu = menu_bar.menus.remove(from_idx);
            if from_idx <= to_idx {
                to_idx = to_idx.saturating_sub(1);
            }
            let insert_at = (to_idx + 1).min(menu_bar.menus.len());
            menu_bar.menus.insert(insert_at, menu);
            Ok(())
        }
        MenuBarPatchOp::RemoveAt { menu, at } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            let Some(item_idx) = resolve_selector_index(items, &at) else {
                return fail("item selector not found".to_string());
            };
            if item_idx >= items.len() {
                return fail(format!("index out of bounds: {item_idx}"));
            }
            items.remove(item_idx);
            Ok(())
        }
        MenuBarPatchOp::MoveAtBefore { menu, at, before } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            let Some(from_idx) = resolve_selector_index(items, &at) else {
                return fail("item selector not found".to_string());
            };
            let Some(mut to_idx) = resolve_selector_index(items, &before) else {
                return fail("before selector not found".to_string());
            };
            if from_idx >= items.len() {
                return fail(format!("index out of bounds: {from_idx}"));
            }
            if to_idx >= items.len() {
                return fail(format!("index out of bounds: {to_idx}"));
            }

            let item = items.remove(from_idx);
            if from_idx < to_idx {
                to_idx = to_idx.saturating_sub(1);
            }
            items.insert(to_idx.min(items.len()), item);
            Ok(())
        }
        MenuBarPatchOp::MoveAtAfter { menu, at, after } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            let Some(from_idx) = resolve_selector_index(items, &at) else {
                return fail("item selector not found".to_string());
            };
            let Some(mut to_idx) = resolve_selector_index(items, &after) else {
                return fail("after selector not found".to_string());
            };
            if from_idx >= items.len() {
                return fail(format!("index out of bounds: {from_idx}"));
            }
            if to_idx >= items.len() {
                return fail(format!("index out of bounds: {to_idx}"));
            }

            let item = items.remove(from_idx);
            if from_idx <= to_idx {
                to_idx = to_idx.saturating_sub(1);
            }
            let insert_at = (to_idx + 1).min(items.len());
            items.insert(insert_at, item);
            Ok(())
        }
        MenuBarPatchOp::RemoveItem { menu, command } => apply_patch_op(
            menu_bar,
            index,
            MenuBarPatchOp::RemoveAt {
                menu,
                at: ItemSelector::Anchor(ItemAnchor::Command(command)),
            },
        ),
        MenuBarPatchOp::InsertItemBefore { menu, before, item } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            let before = ItemSelector::Anchor(before);
            let Some(item_idx) = resolve_selector_index(items, &before) else {
                return fail("before selector not found".to_string());
            };
            let insert_at = item_idx.min(items.len());
            items.insert(insert_at, item);
            Ok(())
        }
        MenuBarPatchOp::InsertItemAfter { menu, after, item } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            let after = ItemSelector::Anchor(after);
            let Some(item_idx) = resolve_selector_index(items, &after) else {
                return fail("after selector not found".to_string());
            };
            let insert_at = (item_idx + 1).min(items.len());
            items.insert(insert_at, item);
            Ok(())
        }
        MenuBarPatchOp::PrependItem { menu, item } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            items.insert(0, item);
            Ok(())
        }
        MenuBarPatchOp::AppendItem { menu, item } => {
            let Some(items) = resolve_menu_items_mut(menu_bar, &menu) else {
                return fail("menu path not found".to_string());
            };
            items.push(item);
            Ok(())
        }
        MenuBarPatchOp::MoveItemBefore {
            menu,
            command,
            before,
        } => apply_patch_op(
            menu_bar,
            index,
            MenuBarPatchOp::MoveAtBefore {
                menu,
                at: ItemSelector::Anchor(ItemAnchor::Command(command)),
                before: ItemSelector::Anchor(before),
            },
        ),
        MenuBarPatchOp::MoveItemAfter {
            menu,
            command,
            after,
        } => apply_patch_op(
            menu_bar,
            index,
            MenuBarPatchOp::MoveAtAfter {
                menu,
                at: ItemSelector::Anchor(ItemAnchor::Command(command)),
                after: ItemSelector::Anchor(after),
            },
        ),
    }
}
