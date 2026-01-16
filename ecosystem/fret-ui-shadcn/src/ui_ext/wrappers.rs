use crate::hover_card::{HoverCardAnchor, HoverCardTrigger};
use crate::popover::{PopoverAnchor, PopoverTrigger};
use crate::tooltip::{TooltipAnchor, TooltipTrigger};

impl_ui_patch_passthrough!(PopoverTrigger);
impl_ui_patch_passthrough!(PopoverAnchor);

impl_ui_patch_passthrough!(TooltipTrigger);
impl_ui_patch_passthrough!(TooltipAnchor);

impl_ui_patch_passthrough!(HoverCardTrigger);
impl_ui_patch_passthrough!(HoverCardAnchor);
