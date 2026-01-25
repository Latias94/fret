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

impl DismissiblePopoverRequest {
    pub fn new(
        id: GlobalElementId,
        trigger: GlobalElementId,
        open: Model<bool>,
        children: impl IntoIterator<Item = AnyElement>,
    ) -> Self {
        Self {
            id,
            root_name: super::popover_root_name(id),
            trigger,
            dismissable_branches: Vec::new(),
            consume_outside_pointer_events: false,
            disable_outside_pointer_events: false,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open,
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: children.into_iter().collect(),
        }
    }

    pub fn root_name(mut self, root_name: impl Into<String>) -> Self {
        self.root_name = root_name.into();
        self
    }

    pub fn dismissable_branches(
        mut self,
        branches: impl IntoIterator<Item = GlobalElementId>,
    ) -> Self {
        self.dismissable_branches = branches.into_iter().collect();
        self
    }

    pub fn consume_outside_pointer_events(mut self, enabled: bool) -> Self {
        self.consume_outside_pointer_events = enabled;
        self
    }

    pub fn disable_outside_pointer_events(mut self, enabled: bool) -> Self {
        self.disable_outside_pointer_events = enabled;
        self
    }

    pub fn close_on_window_focus_lost(mut self, enabled: bool) -> Self {
        self.close_on_window_focus_lost = enabled;
        self
    }

    pub fn close_on_window_resize(mut self, enabled: bool) -> Self {
        self.close_on_window_resize = enabled;
        self
    }

    pub fn present(mut self, present: bool) -> Self {
        self.present = present;
        self
    }

    pub fn initial_focus(mut self, initial_focus: Option<GlobalElementId>) -> Self {
        self.initial_focus = initial_focus;
        self
    }

    pub fn on_open_auto_focus(mut self, on_open_auto_focus: Option<OnOpenAutoFocus>) -> Self {
        self.on_open_auto_focus = on_open_auto_focus;
        self
    }

    pub fn on_close_auto_focus(mut self, on_close_auto_focus: Option<OnCloseAutoFocus>) -> Self {
        self.on_close_auto_focus = on_close_auto_focus;
        self
    }

    pub fn on_dismiss_request(mut self, on_dismiss_request: Option<OnDismissRequest>) -> Self {
        self.on_dismiss_request = on_dismiss_request;
        self
    }

    pub fn on_pointer_move(mut self, on_pointer_move: Option<OnDismissiblePointerMove>) -> Self {
        self.on_pointer_move = on_pointer_move;
        self
    }
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

impl ModalRequest {
    pub fn new(
        id: GlobalElementId,
        open: Model<bool>,
        children: impl IntoIterator<Item = AnyElement>,
    ) -> Self {
        Self {
            id,
            root_name: super::modal_root_name(id),
            trigger: None,
            close_on_window_focus_lost: false,
            close_on_window_resize: false,
            open,
            present: true,
            initial_focus: None,
            on_open_auto_focus: None,
            on_close_auto_focus: None,
            on_dismiss_request: None,
            children: children.into_iter().collect(),
        }
    }

    pub fn root_name(mut self, root_name: impl Into<String>) -> Self {
        self.root_name = root_name.into();
        self
    }

    pub fn trigger(mut self, trigger: Option<GlobalElementId>) -> Self {
        self.trigger = trigger;
        self
    }

    pub fn close_on_window_focus_lost(mut self, enabled: bool) -> Self {
        self.close_on_window_focus_lost = enabled;
        self
    }

    pub fn close_on_window_resize(mut self, enabled: bool) -> Self {
        self.close_on_window_resize = enabled;
        self
    }

    pub fn present(mut self, present: bool) -> Self {
        self.present = present;
        self
    }

    pub fn initial_focus(mut self, initial_focus: Option<GlobalElementId>) -> Self {
        self.initial_focus = initial_focus;
        self
    }

    pub fn on_open_auto_focus(mut self, on_open_auto_focus: Option<OnOpenAutoFocus>) -> Self {
        self.on_open_auto_focus = on_open_auto_focus;
        self
    }

    pub fn on_close_auto_focus(mut self, on_close_auto_focus: Option<OnCloseAutoFocus>) -> Self {
        self.on_close_auto_focus = on_close_auto_focus;
        self
    }

    pub fn on_dismiss_request(mut self, on_dismiss_request: Option<OnDismissRequest>) -> Self {
        self.on_dismiss_request = on_dismiss_request;
        self
    }
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
    pub children: Vec<AnyElement>,
}

impl HoverOverlayRequest {
    pub fn new(
        id: GlobalElementId,
        trigger: GlobalElementId,
        children: impl IntoIterator<Item = AnyElement>,
    ) -> Self {
        Self {
            id,
            root_name: super::hover_overlay_root_name(id),
            interactive: true,
            trigger,
            children: children.into_iter().collect(),
        }
    }

    pub fn root_name(mut self, root_name: impl Into<String>) -> Self {
        self.root_name = root_name.into();
        self
    }

    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }
}

impl std::fmt::Debug for HoverOverlayRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HoverOverlayRequest")
            .field("id", &self.id)
            .field("root_name", &self.root_name)
            .field("interactive", &self.interactive)
            .field("trigger", &self.trigger)
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
    pub on_dismiss_request: Option<OnDismissRequest>,
    pub on_pointer_move: Option<OnDismissiblePointerMove>,
    pub children: Vec<AnyElement>,
}

impl TooltipRequest {
    pub fn new(id: GlobalElementId, children: impl IntoIterator<Item = AnyElement>) -> Self {
        Self {
            id,
            root_name: super::tooltip_root_name(id),
            interactive: true,
            trigger: None,
            on_dismiss_request: None,
            on_pointer_move: None,
            children: children.into_iter().collect(),
        }
    }

    pub fn root_name(mut self, root_name: impl Into<String>) -> Self {
        self.root_name = root_name.into();
        self
    }

    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    pub fn trigger(mut self, trigger: Option<GlobalElementId>) -> Self {
        self.trigger = trigger;
        self
    }

    pub fn on_dismiss_request(mut self, on_dismiss_request: Option<OnDismissRequest>) -> Self {
        self.on_dismiss_request = on_dismiss_request;
        self
    }

    pub fn on_pointer_move(mut self, on_pointer_move: Option<OnDismissiblePointerMove>) -> Self {
        self.on_pointer_move = on_pointer_move;
        self
    }
}

impl std::fmt::Debug for TooltipRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TooltipRequest")
            .field("id", &self.id)
            .field("root_name", &self.root_name)
            .field("interactive", &self.interactive)
            .field("trigger", &self.trigger)
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
