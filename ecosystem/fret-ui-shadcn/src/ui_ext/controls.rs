use crate::button::Button;
use crate::card::Card;
use crate::checkbox::Checkbox;
use crate::input::Input;
use crate::select::Select;
use crate::slider::Slider;
use crate::switch::Switch;
use crate::textarea::Textarea;
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

impl_ui_patch_chrome_layout!(Button);
impl_ui_patch_chrome_layout!(Checkbox);
impl_ui_patch_chrome_layout!(Input);
impl_ui_patch_chrome_layout!(Switch);
impl_ui_patch_chrome_layout!(Textarea);

impl_ui_patch_chrome_layout!(Card);

impl UiPatchTarget for Select {
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_layout(patch.layout)
    }
}

impl UiSupportsLayout for Select {}

impl UiIntoElement for Select {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        Select::into_element(self, cx)
    }
}

impl UiPatchTarget for Slider {
    fn apply_ui_patch(self, patch: UiPatch) -> Self {
        self.refine_layout(patch.layout)
    }
}

impl UiSupportsLayout for Slider {}

impl UiIntoElement for Slider {
    fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        Slider::into_element(self, cx)
    }
}
