use crate::accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};
use crate::avatar::{Avatar, AvatarFallback, AvatarImage};
use crate::progress::Progress;
use crate::skeleton::Skeleton;
use crate::table::{Table, TableCell};
use crate::tabs::Tabs;
use crate::toggle::Toggle;
use crate::toggle_group::ToggleGroup;

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

impl_ui_patch_chrome_layout_patch_only!(AccordionTrigger);
impl_ui_patch_chrome_layout_patch_only!(AccordionContent);
impl_ui_patch_chrome_layout_patch_only!(AccordionItem);
impl_ui_patch_layout_only!(Accordion);
