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

use fret_core::Px;
use fret_runtime::{CommandId, Model};
use fret_ui::action::UiActionHost;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::declarative::ElementContextThemeExt as _;
use fret_ui_kit::{
    OverlayController, OverlayRequest, ToastAction, ToastButtonStyle, ToastId, ToastLayerStyle,
    ToastPosition, ToastRequest, ToastStore, ToastTextStyle,
};

use crate::motion::ms_to_frames;
use crate::tokens::snackbar as snackbar_tokens;

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
        self.action = Some(ToastAction::new(label, command));
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
        req = req.duration(self.duration.to_duration());
        if let Some(action) = self.action {
            req = req.action(action);
        }
        req = req.dismissible(self.dismissible);
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

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let id = cx.root_id();

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

            let style = cx.with_theme(snackbar_toast_layer_style);

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
    let icon_size = snackbar_tokens::icon_size(theme);
    let container_shape = snackbar_tokens::container_shape_radius(theme);
    let shadow = snackbar_tokens::container_shadow(theme);
    let open_ticks = ms_to_frames(snackbar_tokens::open_duration_ms(theme));
    let close_ticks = ms_to_frames(snackbar_tokens::close_duration_ms(theme));
    let easing = snackbar_tokens::easing(theme);
    let single_line_height = snackbar_tokens::single_line_min_height(theme);
    let two_line_height = snackbar_tokens::two_line_min_height(theme);

    let palette = snackbar_tokens::palette();

    ToastLayerStyle {
        palette,
        shadow,
        open_ticks,
        close_ticks,
        easing,
        slide_distance: Px(16.0),
        show_close_button: true,
        border_color_key: None,
        border_width: Px(0.0),
        description_color_key: Some("md.comp.snackbar.supporting-text.color".to_string()),
        icon_size,
        single_line_min_height: single_line_height,
        two_line_min_height: two_line_height,
        container_padding: Some(snackbar_tokens::container_padding(theme)),
        container_radius: Some(container_shape),
        title: ToastTextStyle {
            style_key: Some("md.sys.typescale.body-medium".to_string()),
            color_key: Some("md.comp.snackbar.supporting-text.color".to_string()),
        },
        description: ToastTextStyle {
            style_key: Some("md.sys.typescale.body-medium".to_string()),
            color_key: Some("md.comp.snackbar.supporting-text.color".to_string()),
        },
        action: snackbar_tokens::action_button_style(theme),
        cancel: ToastButtonStyle::default(),
        close: snackbar_tokens::close_icon_button_style(theme),
        ..ToastLayerStyle::default()
    }
}
