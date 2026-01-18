use crate::alert::{Alert, AlertDescription, AlertTitle};
use crate::badge::Badge;
use crate::breadcrumb::Breadcrumb;
use crate::empty::Empty;
use crate::kbd::{Kbd, KbdGroup};

impl_ui_patch_chrome_layout!(Alert);
impl_ui_patch_passthrough!(AlertTitle);
impl_ui_patch_passthrough!(AlertDescription);
impl_ui_patch_chrome_layout!(Badge);
impl_ui_patch_chrome_layout!(Kbd);
impl_ui_patch_layout_only!(KbdGroup);
impl_ui_patch_chrome_layout!(Breadcrumb);
impl_ui_patch_chrome_layout!(Empty);
