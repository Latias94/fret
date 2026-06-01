//! Material 3 snackbar (MVP).
//!
//! This is implemented as a Material-styled `ToastLayer` skin:
//! - persistence, stacking, and timers are provided by `fret-ui-kit`'s toast store
//! - visuals (colors/typography/heights) are driven by `md.comp.snackbar.*` tokens
//!
//! Notes:
//! - For this MVP, snackbar entries are posted to a dedicated `ToastStore` model so they do not
//!   interfere with the shadcn `Toaster` used by the UI gallery shell.

use std::sync::Arc;
use std::time::Duration;

use fret_core::{Corners, Edges, Px};
use fret_runtime::{CommandId, Model};
use fret_ui::action::UiActionHost;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::{
    ColorRef, OverlayController, OverlayRequest, OverrideSlot, ToastAction, ToastButtonStyle,
    ToastId, ToastLayerStyle, ToastOffset, ToastPosition, ToastRequest, ToastStore, ToastTextStyle,
    WidgetStateProperty, WidgetStates, resolve_override_slot_with,
};

use crate::foundation::style_overrides::merge_style_override_slots;
use crate::motion::ms_to_frames;
use crate::tokens::snackbar as snackbar_tokens;

#[derive(Debug, Clone, Default)]
pub struct SnackbarStyle {
    pub container_background: OverrideSlot<ColorRef>,
    pub supporting_text_color: OverrideSlot<ColorRef>,
    pub action_label_color: OverrideSlot<ColorRef>,
    pub action_state_layer_color: OverrideSlot<ColorRef>,
    pub close_icon_color: OverrideSlot<ColorRef>,
    pub close_state_layer_color: OverrideSlot<ColorRef>,
    pub container_corner_radius: OverrideSlot<Px>,
    pub container_padding: OverrideSlot<Edges>,
    pub single_line_min_height: OverrideSlot<Px>,
    pub two_line_min_height: OverrideSlot<Px>,
}

impl SnackbarStyle {
    pub fn container_background(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.container_background = Some(color);
        self
    }

    pub fn supporting_text_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.supporting_text_color = Some(color);
        self
    }

    pub fn action_label_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.action_label_color = Some(color);
        self
    }

    pub fn action_state_layer_color(
        mut self,
        color: WidgetStateProperty<Option<ColorRef>>,
    ) -> Self {
        self.action_state_layer_color = Some(color);
        self
    }

    pub fn close_icon_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.close_icon_color = Some(color);
        self
    }

    pub fn close_state_layer_color(mut self, color: WidgetStateProperty<Option<ColorRef>>) -> Self {
        self.close_state_layer_color = Some(color);
        self
    }

    pub fn container_corner_radius(mut self, radius: WidgetStateProperty<Option<Px>>) -> Self {
        self.container_corner_radius = Some(radius);
        self
    }

    pub fn container_padding(mut self, padding: WidgetStateProperty<Option<Edges>>) -> Self {
        self.container_padding = Some(padding);
        self
    }

    pub fn single_line_min_height(mut self, height: WidgetStateProperty<Option<Px>>) -> Self {
        self.single_line_min_height = Some(height);
        self
    }

    pub fn two_line_min_height(mut self, height: WidgetStateProperty<Option<Px>>) -> Self {
        self.two_line_min_height = Some(height);
        self
    }

    pub fn merged(self, other: Self) -> Self {
        merge_style_override_slots!(
            self,
            other,
            [
                container_background,
                supporting_text_color,
                action_label_color,
                action_state_layer_color,
                close_icon_color,
                close_state_layer_color,
                container_corner_radius,
                container_padding,
                single_line_min_height,
                two_line_min_height,
            ]
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SnackbarDuration {
    Short,
    Long,
    Custom(Duration),
    Indefinite,
}

impl SnackbarDuration {
    pub fn to_duration(self) -> Option<Duration> {
        match self {
            // Material defaults (desktop-friendly).
            Self::Short => Some(Duration::from_secs(4)),
            Self::Long => Some(Duration::from_secs(10)),
            Self::Custom(d) => Some(d),
            Self::Indefinite => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Snackbar {
    pub message: Arc<str>,
    pub supporting_text: Option<Arc<str>>,
    pub action: Option<ToastAction>,
    pub duration: SnackbarDuration,
    pub dismissible: bool,
    pub test_id: Option<Arc<str>>,
}

impl Snackbar {
    pub fn new(message: impl Into<Arc<str>>) -> Self {
        Self {
            message: message.into(),
            supporting_text: None,
            action: None,
            duration: SnackbarDuration::Short,
            dismissible: true,
            test_id: None,
        }
    }

    pub fn supporting_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    pub fn action(mut self, label: impl Into<Arc<str>>, command: impl Into<CommandId>) -> Self {
        self.action = Some(ToastAction::new(label, command));
        self
    }

    pub fn action_id(self, label: impl Into<Arc<str>>, action: impl Into<CommandId>) -> Self {
        self.action(label, action)
    }

    pub fn action_command(self, label: impl Into<Arc<str>>, command: impl Into<CommandId>) -> Self {
        self.action(label, command)
    }

    pub fn duration(mut self, duration: SnackbarDuration) -> Self {
        self.duration = duration;
        self
    }

    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_toast_request(self) -> ToastRequest {
        let mut req = ToastRequest::new(self.message).variant(fret_ui_kit::ToastVariant::Default);
        if let Some(desc) = self.supporting_text {
            req = req.description(desc);
        }
        req = req.duration(self.duration.to_duration());
        if let Some(action) = self.action {
            req = req.action(action);
        }
        req = req.dismissible(self.dismissible);
        if let Some(test_id) = self.test_id {
            req = req.test_id(test_id);
        }
        req
    }
}

#[derive(Debug, Clone)]
pub struct SnackbarController {
    store: Model<ToastStore>,
}

impl SnackbarController {
    pub fn new(store: Model<ToastStore>) -> Self {
        Self { store }
    }

    pub fn store(&self) -> Model<ToastStore> {
        self.store.clone()
    }

    pub fn show(
        &self,
        host: &mut dyn UiActionHost,
        window: fret_core::AppWindowId,
        snackbar: Snackbar,
    ) -> ToastId {
        OverlayController::toast_action(
            host,
            self.store.clone(),
            window,
            snackbar.into_toast_request(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct SnackbarHost {
    store: Model<ToastStore>,
    position: ToastPosition,
    max_snackbars: usize,
    margin: Option<Px>,
    gap: Option<Px>,
    min_width: Option<Px>,
    max_width: Option<Px>,
    style: SnackbarStyle,
}

#[derive(Debug, Default)]
struct SnackbarHostConfigState {
    max_snackbars: Option<usize>,
}

impl SnackbarHost {
    pub fn new(store: Model<ToastStore>) -> Self {
        Self {
            store,
            position: ToastPosition::BottomCenter,
            max_snackbars: 1,
            margin: None,
            gap: None,
            min_width: None,
            max_width: None,
            style: SnackbarStyle::default(),
        }
    }

    pub fn controller(&self) -> SnackbarController {
        SnackbarController::new(self.store.clone())
    }

    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    pub fn max_snackbars(mut self, max: usize) -> Self {
        self.max_snackbars = max.max(1);
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

    pub fn min_width(mut self, width: Px) -> Self {
        self.min_width = Some(width);
        self
    }

    pub fn max_width(mut self, width: Px) -> Self {
        self.max_width = Some(width);
        self
    }

    pub fn style(mut self, style: SnackbarStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let id = cx.root_id();

            let config_changed = cx.slot_state(SnackbarHostConfigState::default, |st| {
                let max = Some(self.max_snackbars);
                if st.max_snackbars == max {
                    return false;
                }
                st.max_snackbars = max;
                true
            });
            if config_changed {
                let _ = cx.app.models_mut().update(&self.store, |st| {
                    st.set_window_max_toasts(cx.window, Some(self.max_snackbars))
                });
            }

            let (style, default_margin, default_max_width) = cx.with_theme(|theme| {
                (
                    snackbar_toast_layer_style(theme, &self.style),
                    snackbar_tokens::host_margin(theme),
                    snackbar_tokens::container_max_width(theme),
                )
            });

            let mut request = OverlayRequest::toast_layer(id, self.store.clone())
                .toast_position(self.position)
                .toast_style(style)
                .toast_margin(self.margin.unwrap_or(default_margin))
                .toast_mobile_offset(ToastOffset::all(default_margin))
                .toast_container_aria_label("Alert");
            if let Some(gap) = self.gap {
                request = request.toast_gap(gap);
            }
            if let Some(width) = self.min_width {
                request = request.toast_min_width(width);
            }
            let max_width = self
                .max_width
                .or_else(|| self.min_width.is_none().then_some(default_max_width));
            if let Some(width) = max_width {
                request = request.toast_max_width(width);
            }
            OverlayController::request(cx, request);

            cx.stack(|_cx| Vec::new())
        })
    }
}

fn snackbar_color_override(
    theme: &Theme,
    slot: &OverrideSlot<ColorRef>,
    states: WidgetStates,
) -> Option<fret_core::Color> {
    resolve_override_slot_with(
        slot.as_ref(),
        states,
        |color| Some(color.resolve(theme)),
        || None,
    )
}

fn snackbar_corner_radii_override(
    slot: &OverrideSlot<Px>,
    states: WidgetStates,
    fallback: impl FnOnce() -> Corners,
) -> Corners {
    resolve_override_slot_with(
        slot.as_ref(),
        states,
        |value| Corners::all(*value),
        fallback,
    )
}

fn snackbar_edges_override(
    slot: &OverrideSlot<Edges>,
    states: WidgetStates,
    fallback: impl FnOnce() -> Edges,
) -> Edges {
    resolve_override_slot_with(slot.as_ref(), states, |value| *value, fallback)
}

fn snackbar_optional_metric_override(
    slot: &OverrideSlot<Px>,
    states: WidgetStates,
    fallback: impl FnOnce() -> Option<Px>,
) -> Option<Px> {
    resolve_override_slot_with(slot.as_ref(), states, |value| Some(*value), fallback)
}

fn snackbar_toast_layer_style(theme: &Theme, style: &SnackbarStyle) -> ToastLayerStyle {
    let icon_size = snackbar_tokens::icon_size(theme);
    let states = WidgetStates::empty();
    let container_corner_radii =
        snackbar_corner_radii_override(&style.container_corner_radius, states, || {
            snackbar_tokens::container_shape(theme)
        });
    let shadow = snackbar_tokens::container_shadow(theme);
    let open_ticks = ms_to_frames(snackbar_tokens::open_duration_ms(theme));
    let close_ticks = ms_to_frames(snackbar_tokens::close_duration_ms(theme));
    let easing = snackbar_tokens::easing(theme);
    let single_line_height =
        snackbar_optional_metric_override(&style.single_line_min_height, states, || {
            snackbar_tokens::single_line_min_height(theme)
        });
    let two_line_height =
        snackbar_optional_metric_override(&style.two_line_min_height, states, || {
            snackbar_tokens::two_line_min_height(theme)
        });
    let container_padding = snackbar_edges_override(&style.container_padding, states, || {
        snackbar_tokens::container_padding(theme)
    });

    let palette = snackbar_tokens::palette();
    let background_color = snackbar_color_override(theme, &style.container_background, states);
    let supporting_text_color =
        snackbar_color_override(theme, &style.supporting_text_color, states);
    let mut action = snackbar_tokens::action_button_style(theme);
    action.label_color = snackbar_color_override(theme, &style.action_label_color, states);
    action.state_layer_color =
        snackbar_color_override(theme, &style.action_state_layer_color, states);
    let mut close = snackbar_tokens::close_icon_button_style(theme);
    close.icon_color = snackbar_color_override(theme, &style.close_icon_color, states);
    close.state_layer_color =
        snackbar_color_override(theme, &style.close_state_layer_color, states);

    ToastLayerStyle {
        palette,
        background_color,
        foreground_color: supporting_text_color,
        shadow,
        open_ticks,
        close_ticks,
        easing,
        slide_distance: Px(0.0),
        scale_from: Some(snackbar_tokens::closed_scale(theme)),
        show_close_button: true,
        close_button_aria_label: Some(Arc::from("Dismiss")),
        border_color_key: None,
        border_width: Px(0.0),
        description_color_key: Some("md.comp.snackbar.supporting-text.color".to_string()),
        icon_size,
        single_line_min_height: single_line_height,
        two_line_min_height: two_line_height,
        container_padding: Some(container_padding),
        container_corner_radii: Some(container_corner_radii),
        title: ToastTextStyle {
            style_key: Some("md.comp.snackbar.supporting-text".to_string()),
            color_key: Some("md.comp.snackbar.supporting-text.color".to_string()),
            color: supporting_text_color,
        },
        description: ToastTextStyle {
            style_key: Some("md.comp.snackbar.supporting-text".to_string()),
            color_key: Some("md.comp.snackbar.supporting-text.color".to_string()),
            color: supporting_text_color,
        },
        description_color: supporting_text_color,
        action,
        cancel: ToastButtonStyle::default(),
        close,
        ..ToastLayerStyle::default()
    }
}
