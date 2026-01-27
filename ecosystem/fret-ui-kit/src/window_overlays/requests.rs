use fret_core::Px;
use fret_runtime::Model;
use fret_ui::action::{
    OnCloseAutoFocus, OnDismissRequest, OnDismissiblePointerMove, OnOpenAutoFocus,
};
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;

use super::{ToastPosition, ToastStore, ToastVariant, toast_layer_root_name};

#[derive(Debug, Clone)]
pub struct ToastVariantColors {
    pub bg: String,
    pub fg: String,
}

impl ToastVariantColors {
    pub fn new(bg: impl Into<String>, fg: impl Into<String>) -> Self {
        Self {
            bg: bg.into(),
            fg: fg.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToastVariantPalette {
    pub default: ToastVariantColors,
    pub destructive: ToastVariantColors,
    pub success: ToastVariantColors,
    pub info: ToastVariantColors,
    pub warning: ToastVariantColors,
    pub error: ToastVariantColors,
    pub loading: ToastVariantColors,
}

impl Default for ToastVariantPalette {
    fn default() -> Self {
        // Default keys match the shadcn/sonner theme surface. These are used as the baseline for
        // `fret-ui-kit`'s toast layer renderer.
        Self {
            default: ToastVariantColors::new("popover", "popover-foreground"),
            destructive: ToastVariantColors::new("destructive", "destructive-foreground"),
            success: ToastVariantColors::new("success", "success-foreground"),
            info: ToastVariantColors::new("info", "info-foreground"),
            warning: ToastVariantColors::new("warning", "warning-foreground"),
            error: ToastVariantColors::new("destructive", "destructive-foreground"),
            loading: ToastVariantColors::new("popover", "popover-foreground"),
        }
    }
}

impl ToastVariantPalette {
    pub fn for_variant(&self, variant: ToastVariant) -> &ToastVariantColors {
        match variant {
            ToastVariant::Default => &self.default,
            ToastVariant::Destructive => &self.destructive,
            ToastVariant::Success => &self.success,
            ToastVariant::Info => &self.info,
            ToastVariant::Warning => &self.warning,
            ToastVariant::Error => &self.error,
            ToastVariant::Loading => &self.loading,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToastTextStyle {
    pub style_key: Option<String>,
    pub color_key: Option<String>,
}

impl Default for ToastTextStyle {
    fn default() -> Self {
        Self {
            style_key: None,
            color_key: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToastButtonStyle {
    pub label_style_key: Option<String>,
    pub label_color_key: Option<String>,
    pub state_layer_color_key: Option<String>,
    pub hover_state_layer_opacity_key: Option<String>,
    pub focus_state_layer_opacity_key: Option<String>,
    pub pressed_state_layer_opacity_key: Option<String>,
    pub hover_state_layer_opacity: f32,
    pub focus_state_layer_opacity: f32,
    pub pressed_state_layer_opacity: f32,
    pub padding: fret_core::Edges,
    pub radius: fret_core::Px,
}

impl Default for ToastButtonStyle {
    fn default() -> Self {
        Self {
            label_style_key: None,
            label_color_key: None,
            state_layer_color_key: Some("muted".to_string()),
            hover_state_layer_opacity_key: None,
            focus_state_layer_opacity_key: None,
            pressed_state_layer_opacity_key: None,
            hover_state_layer_opacity: 0.6,
            focus_state_layer_opacity: 0.6,
            pressed_state_layer_opacity: 0.8,
            padding: fret_core::Edges {
                left: Px(8.0),
                right: Px(8.0),
                top: Px(4.0),
                bottom: Px(4.0),
            },
            radius: Px(6.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToastIconButtonStyle {
    pub icon_color_key: Option<String>,
    pub state_layer_color_key: Option<String>,
    pub hover_state_layer_opacity_key: Option<String>,
    pub focus_state_layer_opacity_key: Option<String>,
    pub pressed_state_layer_opacity_key: Option<String>,
    pub hover_state_layer_opacity: f32,
    pub focus_state_layer_opacity: f32,
    pub pressed_state_layer_opacity: f32,
    pub padding: fret_core::Edges,
    pub radius: fret_core::Px,
}

impl Default for ToastIconButtonStyle {
    fn default() -> Self {
        Self {
            icon_color_key: None,
            state_layer_color_key: Some("muted".to_string()),
            hover_state_layer_opacity_key: None,
            focus_state_layer_opacity_key: None,
            pressed_state_layer_opacity_key: None,
            hover_state_layer_opacity: 0.6,
            focus_state_layer_opacity: 0.6,
            pressed_state_layer_opacity: 0.8,
            padding: fret_core::Edges {
                left: Px(8.0),
                right: Px(8.0),
                top: Px(4.0),
                bottom: Px(4.0),
            },
            radius: Px(6.0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToastLayerStyle {
    pub palette: ToastVariantPalette,
    /// Optional shadow for the toast container.
    pub shadow: Option<fret_ui::element::ShadowStyle>,
    /// Motion timing for enter/exit presence.
    ///
    /// Defaults keep the existing shadcn-aligned behavior.
    pub open_ticks: u64,
    pub close_ticks: u64,
    pub easing: Option<fret_ui::theme::CubicBezier>,
    pub slide_distance: Px,
    /// Optional border color. When omitted, no border is drawn.
    pub border_color_key: Option<String>,
    pub border_width: Px,
    pub description_color_key: Option<String>,
    pub icon_size: Px,
    pub single_line_min_height: Option<Px>,
    pub two_line_min_height: Option<Px>,
    pub container_padding: Option<fret_core::Edges>,
    pub container_radius: Option<fret_core::Px>,
    pub title: ToastTextStyle,
    pub description: ToastTextStyle,
    pub action: ToastButtonStyle,
    pub cancel: ToastButtonStyle,
    pub close: ToastIconButtonStyle,
}

impl Default for ToastLayerStyle {
    fn default() -> Self {
        Self {
            palette: ToastVariantPalette::default(),
            shadow: None,
            open_ticks: 12,
            close_ticks: 12,
            easing: None,
            slide_distance: Px(16.0),
            border_color_key: Some("border".to_string()),
            border_width: Px(1.0),
            description_color_key: Some("muted-foreground".to_string()),
            icon_size: Px(16.0),
            single_line_min_height: None,
            two_line_min_height: None,
            container_padding: None,
            container_radius: None,
            title: ToastTextStyle::default(),
            description: ToastTextStyle::default(),
            action: ToastButtonStyle::default(),
            cancel: ToastButtonStyle::default(),
            close: ToastIconButtonStyle::default(),
        }
    }
}

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
    pub open: Model<bool>,
    pub present: bool,
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
    pub style: ToastLayerStyle,
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
            .field("style", &self.style)
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
            style: ToastLayerStyle::default(),
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

    pub fn style(mut self, style: ToastLayerStyle) -> Self {
        self.style = style;
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
