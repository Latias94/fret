use crate::hover_card::HoverCard;
use crate::resizable::{ResizablePanel, ResizablePanelGroup};
use crate::scroll_area::{ScrollArea, ScrollAreaRoot};
use crate::spinner::Spinner;
use crate::tooltip::Tooltip;

impl_ui_patch_layout_only_patch_only!(ResizablePanel);
impl_ui_patch_layout_only!(ResizablePanelGroup);

impl_ui_patch_layout_only!(ScrollArea);
impl_ui_patch_layout_only!(ScrollAreaRoot);

impl_ui_patch_layout_only!(Spinner);

// These types accept layout refinements, but their `into_element` signatures require child/content
// closures, so they opt into patch-only for now.
impl_ui_patch_layout_only_patch_only!(Tooltip);
impl_ui_patch_layout_only_patch_only!(HoverCard);
