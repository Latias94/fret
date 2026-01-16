use crate::accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};
use crate::avatar::{Avatar, AvatarFallback, AvatarImage};
use crate::progress::Progress;
use crate::skeleton::Skeleton;
use crate::table::{Table, TableCell};
use crate::tabs::Tabs;
use crate::toggle::Toggle;
use crate::toggle_group::ToggleGroup;
use ::fret_ui_kit::{UiIntoElement, UiPatch, UiPatchTarget, UiSupportsChrome, UiSupportsLayout};
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

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

macro_rules! impl_ui_patch_layout_only {
    ($ty:ty) => {
        impl UiPatchTarget for $ty {
            fn apply_ui_patch(self, patch: UiPatch) -> Self {
                self.refine_layout(patch.layout)
            }
        }

        impl UiSupportsLayout for $ty {}

        impl UiIntoElement for $ty {
            fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
                <Self>::into_element(self, cx)
            }
        }
    };
}

impl_ui_patch_chrome_layout!(Avatar);
impl_ui_patch_chrome_layout!(AvatarFallback);
impl_ui_patch_layout_only!(AvatarImage);

impl_ui_patch_chrome_layout!(Progress);
impl_ui_patch_chrome_layout!(Skeleton);

impl_ui_patch_chrome_layout!(Tabs);
impl_ui_patch_chrome_layout!(Toggle);
impl_ui_patch_chrome_layout!(ToggleGroup);

impl_ui_patch_chrome_layout!(Table);
impl_ui_patch_chrome_layout!(TableCell);

impl_ui_patch_chrome_layout!(AccordionTrigger);
impl_ui_patch_chrome_layout!(AccordionContent);
impl_ui_patch_chrome_layout!(AccordionItem);
impl_ui_patch_layout_only!(Accordion);
