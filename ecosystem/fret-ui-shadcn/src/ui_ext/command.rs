use crate::command::{
    Command, CommandDialog, CommandEmpty, CommandInput, CommandList, CommandPalette,
    CommandShortcut,
};

impl_ui_patch_chrome_layout!(Command);
impl_ui_patch_chrome_layout!(CommandPalette);
impl_ui_patch_layout_only!(CommandInput);

impl_ui_patch_passthrough_patch_only!(CommandDialog);
impl_ui_patch_passthrough!(CommandEmpty);
impl_ui_patch_passthrough!(CommandList);
impl_ui_patch_passthrough!(CommandShortcut);
