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

use fret_core::{Edges, Px};
use fret_runtime::{CommandId, Model};
use fret_ui::action::UiActionHost;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::{
    OverlayController, OverlayRequest, ToastAction, ToastButtonStyle, ToastIconButtonStyle,
    ToastId, ToastLayerStyle, ToastPosition, ToastRequest, ToastStore, ToastTextStyle,
    ToastVariantColors, ToastVariantPalette,
};

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
}

impl Snackbar {
    pub fn new(message: impl Into<Arc<str>>) -> Self {
        Self {
            message: message.into(),
            supporting_text: None,
            action: None,
            duration: SnackbarDuration::Short,
            dismissible: true,
        }
    }

    pub fn supporting_text(mut self, text: impl Into<Arc<str>>) -> Self {
        self.supporting_text = Some(text.into());
        self
    }

    pub fn action(mut self, label: impl Into<Arc<str>>, command: CommandId) -> Self {
        self.action = Some(ToastAction {
            label: label.into(),
            command,
        });
        self
    }

    pub fn duration(mut self, duration: SnackbarDuration) -> Self {
        self.duration = duration;
        self
    }

    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    pub fn into_toast_request(self) -> ToastRequest {
        let mut req = ToastRequest::new(self.message).variant(fret_ui_kit::ToastVariant::Default);
        if let Some(desc) = self.supporting_text {
            req = req.description(desc);
        }
        req.duration = self.duration.to_duration();
        req.action = self.action;
        req.dismissible = self.dismissible;
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

    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let id = cx.root_id();
            let theme = Theme::global(&*cx.app).clone();

            let config_changed = cx.with_state(SnackbarHostConfigState::default, |st| {
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

            let style = snackbar_toast_layer_style(&theme);

            let mut request = OverlayRequest::toast_layer(id, self.store.clone())
                .toast_position(self.position)
                .toast_style(style);
            if let Some(margin) = self.margin {
                request = request.toast_margin(margin);
            }
            if let Some(gap) = self.gap {
                request = request.toast_gap(gap);
            }
            if let Some(width) = self.min_width {
                request = request.toast_min_width(width);
            }
            if let Some(width) = self.max_width {
                request = request.toast_max_width(width);
            }
            OverlayController::request(cx, request);

            cx.stack(|_cx| Vec::new())
        })
    }
}

fn snackbar_toast_layer_style(theme: &Theme) -> ToastLayerStyle {
    let icon_size = theme
        .metric_by_key("md.comp.snackbar.icon.size")
        .unwrap_or(Px(24.0));
    let container_shape = theme
        .metric_by_key("md.comp.snackbar.container.shape")
        .unwrap_or(Px(4.0));
    let single_line_height =
        theme.metric_by_key("md.comp.snackbar.with-single-line.container.height");
    let two_line_height = theme.metric_by_key("md.comp.snackbar.with-two-lines.container.height");

    let palette = ToastVariantPalette {
        default: ToastVariantColors::new(
            "md.comp.snackbar.container.color",
            "md.comp.snackbar.supporting-text.color",
        ),
        destructive: ToastVariantColors::new(
            "md.comp.snackbar.container.color",
            "md.comp.snackbar.supporting-text.color",
        ),
        success: ToastVariantColors::new(
            "md.comp.snackbar.container.color",
            "md.comp.snackbar.supporting-text.color",
        ),
        info: ToastVariantColors::new(
            "md.comp.snackbar.container.color",
            "md.comp.snackbar.supporting-text.color",
        ),
        warning: ToastVariantColors::new(
            "md.comp.snackbar.container.color",
            "md.comp.snackbar.supporting-text.color",
        ),
        error: ToastVariantColors::new(
            "md.comp.snackbar.container.color",
            "md.comp.snackbar.supporting-text.color",
        ),
        loading: ToastVariantColors::new(
            "md.comp.snackbar.container.color",
            "md.comp.snackbar.supporting-text.color",
        ),
    };

    ToastLayerStyle {
        palette,
        border_color_key: None,
        border_width: Px(0.0),
        description_color_key: Some("md.comp.snackbar.supporting-text.color".to_string()),
        icon_size,
        single_line_min_height: single_line_height,
        two_line_min_height: two_line_height,
        // Token source does not define padding; keep a conservative default that fits the fixed
        // container heights.
        container_padding: Some(Edges {
            left: Px(16.0),
            right: Px(16.0),
            top: Px(8.0),
            bottom: Px(8.0),
        }),
        container_radius: Some(container_shape),
        title: ToastTextStyle {
            style_key: Some("md.sys.typescale.body-medium".to_string()),
            color_key: Some("md.comp.snackbar.supporting-text.color".to_string()),
        },
        description: ToastTextStyle {
            style_key: Some("md.sys.typescale.body-medium".to_string()),
            color_key: Some("md.comp.snackbar.supporting-text.color".to_string()),
        },
        action: ToastButtonStyle {
            label_style_key: Some("md.sys.typescale.label-large".to_string()),
            label_color_key: Some("md.comp.snackbar.action.label-text.color".to_string()),
            state_layer_color_key: Some(
                "md.comp.snackbar.action.hover.state-layer.color".to_string(),
            ),
            hover_state_layer_opacity_key: Some(
                "md.comp.snackbar.action.hover.state-layer.opacity".to_string(),
            ),
            focus_state_layer_opacity_key: Some(
                "md.comp.snackbar.action.focus.state-layer.opacity".to_string(),
            ),
            pressed_state_layer_opacity_key: Some(
                "md.comp.snackbar.action.pressed.state-layer.opacity".to_string(),
            ),
            hover_state_layer_opacity: 0.08,
            focus_state_layer_opacity: 0.1,
            pressed_state_layer_opacity: 0.1,
            padding: Edges {
                left: Px(12.0),
                right: Px(12.0),
                top: Px(4.0),
                bottom: Px(4.0),
            },
            radius: Px(4.0),
        },
        cancel: ToastButtonStyle::default(),
        close: ToastIconButtonStyle {
            icon_color_key: Some("md.comp.snackbar.icon.color".to_string()),
            state_layer_color_key: Some(
                "md.comp.snackbar.icon.hover.state-layer.color".to_string(),
            ),
            hover_state_layer_opacity_key: Some(
                "md.comp.snackbar.icon.hover.state-layer.opacity".to_string(),
            ),
            focus_state_layer_opacity_key: Some(
                "md.comp.snackbar.icon.focus.state-layer.opacity".to_string(),
            ),
            pressed_state_layer_opacity_key: Some(
                "md.comp.snackbar.icon.pressed.state-layer.opacity".to_string(),
            ),
            hover_state_layer_opacity: 0.08,
            focus_state_layer_opacity: 0.1,
            pressed_state_layer_opacity: 0.1,
            padding: Edges {
                left: Px(8.0),
                right: Px(8.0),
                top: Px(8.0),
                bottom: Px(8.0),
            },
            radius: Px(4.0),
        },
    }
}
