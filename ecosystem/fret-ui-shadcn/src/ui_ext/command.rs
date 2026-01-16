use crate::command::{Command, CommandInput, CommandPalette};

impl_ui_patch_chrome_layout!(Command);
impl_ui_patch_chrome_layout!(CommandPalette);
impl_ui_patch_layout_only!(CommandInput);
