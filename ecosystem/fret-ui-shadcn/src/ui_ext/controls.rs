use crate::button::Button;
use crate::card::Card;
use crate::checkbox::Checkbox;
use crate::input::Input;
use crate::select::Select;
use crate::slider::Slider;
use crate::switch::Switch;
use crate::textarea::Textarea;

impl_ui_patch_chrome_layout!(Button);
impl_ui_patch_chrome_layout!(Checkbox);
impl_ui_patch_chrome_layout!(Input);
impl_ui_patch_chrome_layout!(Switch);
impl_ui_patch_chrome_layout!(Textarea);

impl_ui_patch_chrome_layout!(Card);

impl_ui_patch_chrome_layout!(Select);
impl_ui_patch_chrome_layout!(Slider);
