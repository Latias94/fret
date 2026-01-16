use crate::alert::{Alert, AlertDescription, AlertTitle};
use crate::badge::Badge;
use crate::breadcrumb::Breadcrumb;
use crate::kbd::Kbd;

impl_ui_patch_passthrough!(Alert);
impl_ui_patch_passthrough!(AlertTitle);
impl_ui_patch_passthrough!(AlertDescription);
impl_ui_patch_passthrough!(Badge);
impl_ui_patch_passthrough!(Kbd);
impl_ui_patch_passthrough!(Breadcrumb);
