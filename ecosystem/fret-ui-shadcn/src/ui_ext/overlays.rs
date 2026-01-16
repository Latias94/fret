use crate::alert_dialog::AlertDialogContent;
use crate::dialog::DialogContent;
use crate::drawer::DrawerContent;
use crate::hover_card::HoverCardContent;
use crate::popover::PopoverContent;
use crate::sheet::SheetContent;
use crate::tooltip::TooltipContent;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};
use ::fret_ui_kit::{UiIntoElement, UiPatch, UiPatchTarget, UiSupportsChrome, UiSupportsLayout};

macro_rules! impl_ui_patch_chrome_layout {
    ($ty:ty) => {
        impl UiPatchTarget for $ty {
            fn apply_ui_patch(self, patch: UiPatch) -> Self {
                self.refine_style(patch.chrome).refine_layout(patch.layout)
            }
        }

        impl UiSupportsChrome for $ty {}
        impl UiSupportsLayout for $ty {}

        impl UiIntoElement for $ty {
            fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
                <Self>::into_element(self, cx)
            }
        }
    };
}

impl_ui_patch_chrome_layout!(PopoverContent);
impl_ui_patch_chrome_layout!(TooltipContent);

impl_ui_patch_chrome_layout!(AlertDialogContent);
impl_ui_patch_chrome_layout!(DialogContent);
impl_ui_patch_chrome_layout!(SheetContent);
impl_ui_patch_chrome_layout!(HoverCardContent);
impl_ui_patch_chrome_layout!(DrawerContent);
