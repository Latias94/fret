use crate::alert::{Alert, AlertDescription, AlertTitle};
use crate::badge::Badge;
use crate::breadcrumb::Breadcrumb;
use crate::empty::Empty;
use crate::kbd::{Kbd, KbdGroup};
use crate::separator::Separator;

impl_ui_patch_chrome_layout!(Alert);
impl_ui_patch_passthrough!(AlertTitle);
impl_ui_patch_passthrough!(AlertDescription);
impl_ui_patch_chrome_layout!(Badge);
impl_ui_patch_chrome_layout!(Kbd);
impl_ui_patch_layout_only!(KbdGroup);
impl_ui_patch_chrome_layout!(Breadcrumb);
impl_ui_patch_chrome_layout!(Empty);
impl_ui_patch_layout_only!(Separator);

// Upstream-shaped breadcrumb primitives (see `breadcrumb::primitives`).
impl_ui_patch_chrome_layout_patch_only!(crate::breadcrumb::primitives::Breadcrumb);
impl_ui_patch_chrome_layout_patch_only!(crate::breadcrumb::primitives::BreadcrumbList);
impl_ui_patch_chrome_layout_patch_only!(crate::breadcrumb::primitives::BreadcrumbItem);
impl_ui_patch_chrome_layout!(crate::breadcrumb::primitives::BreadcrumbLink);
impl_ui_patch_chrome_layout!(crate::breadcrumb::primitives::BreadcrumbPage);
impl_ui_patch_chrome_layout!(crate::breadcrumb::primitives::BreadcrumbSeparator);
impl_ui_patch_chrome_layout!(crate::breadcrumb::primitives::BreadcrumbEllipsis);
