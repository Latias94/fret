use crate::alert_dialog::AlertDialog;
use crate::dialog::Dialog;
use crate::drawer::Drawer;
use crate::popover::Popover;
use crate::sheet::Sheet;

impl_ui_patch_passthrough_patch_only!(Dialog);
impl_ui_patch_passthrough_patch_only!(AlertDialog);
impl_ui_patch_passthrough_patch_only!(Popover);
impl_ui_patch_passthrough_patch_only!(Sheet);
impl_ui_patch_passthrough_patch_only!(Drawer);
