use std::sync::Arc;

use fret_core::Px;
use fret_runtime::Model;
use fret_ui::action::{
    OnCloseAutoFocus, OnDismissRequest, OnDismissiblePointerMove, OnOpenAutoFocus,
};
use fret_ui::element::AnyElement;
use fret_ui::elements::GlobalElementId;

use super::{
    DEFAULT_VISIBLE_TOASTS, ToastPosition, ToastStore, ToastVariant, toast_layer_root_name,
};

#[derive(Debug, Clone)]
pub enum ToastIconOverride {
    /// Do not render an icon.
    Hidden,
    /// Render a text glyph (e.g. "!" / "i" / "×").
    Glyph(Arc<str>),
    /// Render an SVG icon from the shared icon registry (`fret-icons`).
    #[cfg(feature = "icons")]
    IconId(fret_icons::IconId),
}

impl ToastIconOverride {
    pub fn hidden() -> Self {
        Self::Hidden
    }

    pub fn glyph(glyph: impl Into<Arc<str>>) -> Self {
        Self::Glyph(glyph.into())
    }

    #[cfg(feature = "icons")]
    pub fn icon(icon: fret_icons::IconId) -> Self {
        Self::IconId(icon)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ToastIconOverrides {
    /// Overrides the toast close button icon (`icons.close` in Sonner).
    pub close_button: Option<ToastIconOverride>,
    /// Overrides the loading icon (`icons.loading` in Sonner).
    pub loading: Option<ToastIconOverride>,
    /// Overrides the success icon (`icons.success` in Sonner).
    pub success: Option<ToastIconOverride>,
    /// Overrides the info icon (`icons.info` in Sonner).
    pub info: Option<ToastIconOverride>,
    /// Overrides the warning icon (`icons.warning` in Sonner).
    pub warning: Option<ToastIconOverride>,
    /// Overrides the error icon (`icons.error` in Sonner).
    ///
    /// In Fret this applies to both `ToastVariant::Error` and `ToastVariant::Destructive`.
    pub error: Option<ToastIconOverride>,
}

impl ToastIconOverrides {
    pub fn for_variant(&self, variant: ToastVariant) -> Option<&ToastIconOverride> {
        match variant {
            ToastVariant::Success => self.success.as_ref(),
            ToastVariant::Info => self.info.as_ref(),
            ToastVariant::Warning => self.warning.as_ref(),
            ToastVariant::Error | ToastVariant::Destructive => self.error.as_ref(),
            ToastVariant::Loading | ToastVariant::Default => None,
        }
    }
}

/// Sonner-style viewport offsets (`offset` / `mobileOffset`).
///
/// Sonner allows specifying either a single value or per-side offsets. In Fret we model the
/// per-side shape directly and apply default fallbacks at render time.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ToastOffset {
    pub top: Option<Px>,
    pub right: Option<Px>,
    pub bottom: Option<Px>,
    pub left: Option<Px>,
}

impl ToastOffset {
    pub fn all(px: Px) -> Self {
        Self {
            top: Some(px),
            right: Some(px),
            bottom: Some(px),
            left: Some(px),
        }
    }

    pub fn top(mut self, px: Px) -> Self {
        self.top = Some(px);
        self
    }

    pub fn right(mut self, px: Px) -> Self {
        self.right = Some(px);
        self
    }

    pub fn bottom(mut self, px: Px) -> Self {
        self.bottom = Some(px);
        self
    }

    pub fn left(mut self, px: Px) -> Self {
        self.left = Some(px);
        self
    }
}

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
    /// Sonner-style icon overrides (`icons.*`).
    pub icons: ToastIconOverrides,
    /// Whether to render a close (X) icon button on toasts.
    ///
    /// Note: this is distinct from per-toast "dismissible" behavior (e.g. swipe-to-dismiss).
    pub show_close_button: bool,
    /// A11y label for the close button (Sonner: `closeButtonAriaLabel`, default: "Close toast").
    pub close_button_aria_label: Option<Arc<str>>,
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
            icons: ToastIconOverrides::default(),
            show_close_button: true,
            close_button_aria_label: Some(Arc::from("Close toast")),
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
    pub on_pointer_move: Option<OnDismissiblePointerMove>,
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
            .field("on_pointer_move", &self.on_pointer_move.is_some())
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
    pub style: ToastLayerStyle,
    pub toaster_id: Option<Arc<str>>,
    pub visible_toasts: usize,
    pub expand_by_default: bool,
    pub rich_colors: bool,
    pub invert: bool,
    pub container_aria_label: Option<Arc<str>>,
    pub custom_aria_label: Option<Arc<str>>,
    pub offset: Option<ToastOffset>,
    pub mobile_offset: Option<ToastOffset>,
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
            .field("toaster_id", &self.toaster_id)
            .field("visible_toasts", &self.visible_toasts)
            .field("expand_by_default", &self.expand_by_default)
            .field("rich_colors", &self.rich_colors)
            .field("invert", &self.invert)
            .field("container_aria_label", &self.container_aria_label)
            .field("custom_aria_label", &self.custom_aria_label)
            .field("offset", &self.offset)
            .field("mobile_offset", &self.mobile_offset)
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
            toaster_id: None,
            visible_toasts: DEFAULT_VISIBLE_TOASTS,
            expand_by_default: false,
            rich_colors: false,
            invert: false,
            container_aria_label: None,
            custom_aria_label: None,
            offset: None,
            mobile_offset: None,
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

    pub fn toaster_id_opt(mut self, id: Option<Arc<str>>) -> Self {
        self.toaster_id = id;
        self
    }

    pub fn visible_toasts(mut self, visible_toasts: usize) -> Self {
        self.visible_toasts = visible_toasts.max(1);
        self
    }

    pub fn expand_by_default(mut self, expand: bool) -> Self {
        self.expand_by_default = expand;
        self
    }

    pub fn rich_colors(mut self, rich_colors: bool) -> Self {
        self.rich_colors = rich_colors;
        self
    }

    pub fn invert(mut self, invert: bool) -> Self {
        self.invert = invert;
        self
    }

    /// Sets the a11y label for the toast viewport container (mirrors Sonner `containerAriaLabel`).
    pub fn container_aria_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.container_aria_label = Some(label.into());
        self
    }

    pub fn container_aria_label_opt(mut self, label: Option<Arc<str>>) -> Self {
        match label {
            Some(label) => self.container_aria_label(label),
            None => {
                self.container_aria_label = None;
                self
            }
        }
    }

    /// Sets a custom a11y label for the toast viewport container (mirrors Sonner `customAriaLabel`).
    pub fn custom_aria_label_opt(mut self, label: Option<Arc<str>>) -> Self {
        self.custom_aria_label = label;
        self
    }

    pub fn offset(mut self, offset: ToastOffset) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn mobile_offset(mut self, offset: ToastOffset) -> Self {
        self.mobile_offset = Some(offset);
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
