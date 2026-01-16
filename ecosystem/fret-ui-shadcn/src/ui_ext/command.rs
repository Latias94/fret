use crate::command::{Command, CommandInput, CommandPalette};
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

impl_ui_patch_chrome_layout!(Command);
impl_ui_patch_chrome_layout!(CommandPalette);
impl_ui_patch_layout_only!(CommandInput);
