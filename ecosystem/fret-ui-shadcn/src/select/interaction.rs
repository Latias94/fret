use fret_core::{Point, Px};
use fret_runtime::TimerToken;
use fret_ui::elements::GlobalElementId;
use fret_ui_kit::primitives::select as radix_select;

#[derive(Default)]
pub(super) struct SelectOpenChangeCallbackState {
    initialized: bool,
    last_open: bool,
    pending_complete: Option<bool>,
}

#[derive(Debug, Default)]
pub(super) struct SelectScrollArrowAutoScrollState {
    pub(super) token: Option<TimerToken>,
    pub(super) last_pos: Option<Point>,
    pub(super) hovered: bool,
}

#[derive(Debug)]
pub(super) struct SelectTriggerKeyState {
    pub(super) trigger: radix_select::SelectTriggerKeyState,
    pub(super) pointer: radix_select::SelectTriggerPointerState,
    pub(super) content: radix_select::SelectContentKeyState,
    pub(super) was_open: bool,
    pub(super) opened_by_pointer: bool,
    pub(super) opened_by_touch: bool,
    pub(super) scroll_handle: fret_ui::scroll::ScrollHandle,
    pub(super) value_node: Option<GlobalElementId>,
    pub(super) viewport: Option<GlobalElementId>,
    pub(super) listbox: Option<GlobalElementId>,
    pub(super) content_panel: Option<GlobalElementId>,
    pub(super) selected_item: Option<GlobalElementId>,
    pub(super) selected_item_text: Option<GlobalElementId>,
    pub(super) alignment_item_pos: Option<usize>,
    pub(super) alignment_item_has_leading_non_item: bool,
    pub(super) width_probe: Option<GlobalElementId>,
    // Item-aligned select placement can be sensitive to sub-frame layout settling (e.g.
    // text measurement, scroll affordances). To avoid visible "jitter" on hover/focus
    // changes, lock the first stable item-aligned layout for the duration of a single
    // open session (cleared on close/unmount).
    pub(super) last_item_aligned_layout: Option<radix_select::SelectItemAlignedLayout>,
    pub(super) pending_item_aligned_scroll_to_y: Option<Px>,
    pub(super) last_item_aligned_scroll_to_y: Option<Px>,
    pub(super) item_aligned_user_scrolled: bool,
    pub(super) did_item_aligned_scroll_initial: bool,
    pub(super) did_item_aligned_scroll_reposition: bool,
    pub(super) did_item_aligned_focus_scroll: bool,
    pub(super) item_aligned_scroll_up_visible: bool,
    pub(super) pending_active_align_top_scroll: bool,
    pub(super) pending_active_scroll_into_view: bool,
    pub(super) last_item_pointer_move_pos_window: Option<Point>,
    pub(super) keyboard_hover_suppressed: bool,
}

impl SelectTriggerKeyState {
    pub(super) fn new() -> Self {
        Self {
            trigger: radix_select::SelectTriggerKeyState::default(),
            pointer: radix_select::SelectTriggerPointerState::default(),
            content: radix_select::SelectContentKeyState::default(),
            was_open: false,
            opened_by_pointer: false,
            opened_by_touch: false,
            scroll_handle: fret_ui::scroll::ScrollHandle::default(),
            value_node: None,
            viewport: None,
            listbox: None,
            content_panel: None,
            selected_item: None,
            selected_item_text: None,
            alignment_item_pos: None,
            alignment_item_has_leading_non_item: false,
            width_probe: None,
            last_item_aligned_layout: None,
            pending_item_aligned_scroll_to_y: None,
            last_item_aligned_scroll_to_y: None,
            item_aligned_user_scrolled: false,
            did_item_aligned_scroll_initial: false,
            did_item_aligned_scroll_reposition: false,
            did_item_aligned_focus_scroll: false,
            item_aligned_scroll_up_visible: false,
            pending_active_align_top_scroll: false,
            pending_active_scroll_into_view: false,
            last_item_pointer_move_pos_window: None,
            keyboard_hover_suppressed: false,
        }
    }
}

pub(super) fn select_open_change_events(
    state: &mut SelectOpenChangeCallbackState,
    open: bool,
    present: bool,
    animating: bool,
) -> (Option<bool>, Option<bool>) {
    let mut changed = None;
    let mut completed = None;

    if !state.initialized {
        state.initialized = true;
        state.last_open = open;
    } else if state.last_open != open {
        state.last_open = open;
        state.pending_complete = Some(open);
        changed = Some(open);
    }

    if state.pending_complete == Some(open) && present == open && !animating {
        state.pending_complete = None;
        completed = Some(open);
    }

    (changed, completed)
}
