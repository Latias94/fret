use fret_core::Px;
use fret_runtime::Model;
use fret_ui::action::{
    OnCloseAutoFocus, OnDismissRequest, OnDismissiblePointerMove, OnOpenAutoFocus,
};
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;

use super::{ToastPosition, ToastStore, toast_layer_root_name};

#[derive(Clone)]
pub struct DismissiblePopoverRequest {
    pub id: GlobalElementId,
    pub root_name: String,
    pub trigger: GlobalElementId,
    /// Extra subtrees that should be treated as "inside" for outside-press + focus-outside
    /// dismissal policy (Radix DismissableLayer branches).
    pub dismissable_branches: Vec<GlobalElementId>,
    pub consume_outside_pointer_events: bool,
    /// When true and the popover is open, pointer events outside the overlay subtree should not
    /// reach underlay widgets (Radix `disableOutsidePointerEvents` outcome).
    pub disable_outside_pointer_events: bool,
    pub close_on_window_focus_lost: bool,
    pub close_on_window_resize: bool,
    pub open: Model<bool>,
    pub present: bool,
    pub initial_focus: Option<GlobalElementId>,
    pub on_open_auto_focus: Option<OnOpenAutoFocus>,
    pub on_close_auto_focus: Option<OnCloseAutoFocus>,
    pub on_dismiss_request: Option<OnDismissRequest>,
    pub on_pointer_move: Option<OnDismissiblePointerMove>,
    pub children: Vec<AnyElement>,
}

impl std::fmt::Debug for DismissiblePopoverRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DismissiblePopoverRequest")
            .field("id", &self.id)
            .field("root_name", &self.root_name)
            .field("trigger", &self.trigger)
            .field("dismissable_branches_len", &self.dismissable_branches.len())
            .field(
                "consume_outside_pointer_events",
                &self.consume_outside_pointer_events,
            )
            .field(
                "disable_outside_pointer_events",
                &self.disable_outside_pointer_events,
            )
            .field(
                "close_on_window_focus_lost",
                &self.close_on_window_focus_lost,
            )
            .field("close_on_window_resize", &self.close_on_window_resize)
            .field("open", &"<model>")
            .field("present", &self.present)
            .field("initial_focus", &self.initial_focus)
            .field("on_open_auto_focus", &self.on_open_auto_focus.is_some())
            .field("on_close_auto_focus", &self.on_close_auto_focus.is_some())
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .field("on_pointer_move", &self.on_pointer_move.is_some())
            .field("children_len", &self.children.len())
            .finish()
    }
}

#[derive(Clone)]
pub struct ModalRequest {
    pub id: GlobalElementId,
    pub root_name: String,
    pub trigger: Option<GlobalElementId>,
    pub close_on_window_focus_lost: bool,
    pub close_on_window_resize: bool,
    pub open: Model<bool>,
    pub present: bool,
    pub initial_focus: Option<GlobalElementId>,
    pub on_open_auto_focus: Option<OnOpenAutoFocus>,
    pub on_close_auto_focus: Option<OnCloseAutoFocus>,
    pub on_dismiss_request: Option<OnDismissRequest>,
    pub children: Vec<AnyElement>,
}

impl std::fmt::Debug for ModalRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModalRequest")
            .field("id", &self.id)
            .field("root_name", &self.root_name)
            .field("trigger", &self.trigger)
            .field(
                "close_on_window_focus_lost",
                &self.close_on_window_focus_lost,
            )
            .field("close_on_window_resize", &self.close_on_window_resize)
            .field("open", &"<model>")
            .field("present", &self.present)
            .field("initial_focus", &self.initial_focus)
            .field("on_open_auto_focus", &self.on_open_auto_focus.is_some())
            .field("on_close_auto_focus", &self.on_close_auto_focus.is_some())
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .field("children_len", &self.children.len())
            .finish()
    }
}

#[derive(Clone)]
pub struct HoverOverlayRequest {
    pub id: GlobalElementId,
    pub root_name: String,
    /// Whether the overlay should participate in hit-testing and input routing.
    ///
    /// When `false`, the overlay may remain mounted for close transitions but must be click-through
    /// and excluded from any observer passes.
    pub interactive: bool,
    pub trigger: GlobalElementId,
    pub open: Model<bool>,
    pub present: bool,
    pub children: Vec<AnyElement>,
}

impl std::fmt::Debug for HoverOverlayRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoverOverlayRequest")
            .field("id", &self.id)
            .field("root_name", &self.root_name)
            .field("interactive", &self.interactive)
            .field("trigger", &self.trigger)
            .field("open", &"<model>")
            .field("present", &self.present)
            .field("children_len", &self.children.len())
            .finish()
    }
}

#[derive(Clone)]
pub struct TooltipRequest {
    pub id: GlobalElementId,
    pub root_name: String,
    /// Whether the tooltip should participate in input observer routing.
    ///
    /// When `false`, the tooltip may remain mounted for close transitions but must not observe
    /// outside-press or pointer-move events.
    pub interactive: bool,
    pub trigger: Option<GlobalElementId>,
    pub open: Model<bool>,
    pub present: bool,
    pub on_dismiss_request: Option<OnDismissRequest>,
    pub on_pointer_move: Option<OnDismissiblePointerMove>,
    pub children: Vec<AnyElement>,
}

impl std::fmt::Debug for TooltipRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TooltipRequest")
            .field("id", &self.id)
            .field("root_name", &self.root_name)
            .field("interactive", &self.interactive)
            .field("trigger", &self.trigger)
            .field("open", &"<model>")
            .field("present", &self.present)
            .field("on_dismiss_request", &self.on_dismiss_request.is_some())
            .field("on_pointer_move", &self.on_pointer_move.is_some())
            .field("children_len", &self.children.len())
            .finish()
    }
}

#[derive(Clone)]
pub struct ToastLayerRequest {
    pub id: GlobalElementId,
    pub root_name: String,
    pub store: Model<ToastStore>,
    pub position: ToastPosition,
    pub margin: Option<Px>,
    pub gap: Option<Px>,
    pub toast_min_width: Option<Px>,
    pub toast_max_width: Option<Px>,
}

impl std::fmt::Debug for ToastLayerRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToastLayerRequest")
            .field("id", &self.id)
            .field("root_name", &self.root_name)
            .field("store", &"<model>")
            .field("position", &self.position)
            .field("margin", &self.margin)
            .field("gap", &self.gap)
            .field("toast_min_width", &self.toast_min_width)
            .field("toast_max_width", &self.toast_max_width)
            .finish()
    }
}

impl ToastLayerRequest {
    pub fn new(id: GlobalElementId, store: Model<ToastStore>) -> Self {
        Self {
            id,
            root_name: toast_layer_root_name(id),
            store,
            position: ToastPosition::default(),
            margin: None,
            gap: None,
            toast_min_width: None,
            toast_max_width: None,
        }
    }

    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    pub fn root_name(mut self, root_name: impl Into<String>) -> Self {
        self.root_name = root_name.into();
        self
    }

    pub fn margin(mut self, margin: Px) -> Self {
        self.margin = Some(margin);
        self
    }

    pub fn gap(mut self, gap: Px) -> Self {
        self.gap = Some(gap);
        self
    }

    pub fn toast_min_width(mut self, width: Px) -> Self {
        self.toast_min_width = Some(width);
        self
    }

    pub fn toast_max_width(mut self, width: Px) -> Self {
        self.toast_max_width = Some(width);
        self
    }
}
