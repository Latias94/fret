use crate::combobox::Combobox;
use crate::context_menu::{ContextMenu, ContextMenuShortcut};
use crate::dropdown_menu::{DropdownMenu, DropdownMenuShortcut};
use crate::menubar::{Menubar, MenubarMenuEntries, MenubarShortcut};

impl_ui_patch_chrome_layout!(Combobox);

impl_ui_patch_passthrough_patch_only!(ContextMenu);
impl_ui_patch_passthrough!(ContextMenuShortcut);

impl_ui_patch_passthrough_patch_only!(DropdownMenu);
impl_ui_patch_passthrough!(DropdownMenuShortcut);

impl_ui_patch_chrome_layout!(Menubar);
impl_ui_patch_passthrough!(MenubarMenuEntries);
impl_ui_patch_passthrough!(MenubarShortcut);
