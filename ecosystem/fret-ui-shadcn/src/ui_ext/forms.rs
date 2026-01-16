use crate::collapsible::{Collapsible, CollapsibleContent};
use crate::field::Field;
use crate::input_group::InputGroup;
use crate::input_otp::InputOtp;
use crate::item::Item;
use crate::pagination::Pagination;

impl_ui_patch_chrome_layout!(InputGroup);
impl_ui_patch_chrome_layout!(InputOtp);

impl_ui_patch_layout_only_patch_only!(Collapsible);
impl_ui_patch_layout_only!(CollapsibleContent);

impl_ui_patch_layout_only!(Field);
impl_ui_patch_layout_only!(Item);
impl_ui_patch_layout_only!(Pagination);
