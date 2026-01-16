use crate::alert_dialog::AlertDialogContent;
use crate::alert_dialog::AlertDialogTrigger;
use crate::dialog::DialogClose;
use crate::dialog::DialogContent;
use crate::drawer::DrawerClose;
use crate::drawer::DrawerContent;
use crate::hover_card::HoverCardContent;
use crate::popover::PopoverContent;
use crate::sheet::SheetContent;
use crate::tooltip::TooltipContent;

impl_ui_patch_chrome_layout!(PopoverContent);
impl_ui_patch_chrome_layout!(TooltipContent);

impl_ui_patch_chrome_layout!(AlertDialogContent);
impl_ui_patch_passthrough!(AlertDialogTrigger);
impl_ui_patch_chrome_layout!(DialogContent);
impl_ui_patch_chrome_layout!(DialogClose);
impl_ui_patch_chrome_layout!(SheetContent);
impl_ui_patch_chrome_layout!(HoverCardContent);
impl_ui_patch_chrome_layout!(DrawerContent);
impl_ui_patch_chrome_layout!(DrawerClose);
