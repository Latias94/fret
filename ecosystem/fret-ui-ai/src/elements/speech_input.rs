//! AI Elements-aligned `SpeechInput` surface (UI-only).
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/speech-input.tsx`.
//!
//! Notes:
//! - Upstream implements browser speech recognition / MediaRecorder fallbacks.
//! - In Fret, capture and transcription backends are **app-owned**; this surface only renders the
//!   button chrome and emits an intent (`on_listening_change`).

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::action::ActionCx;
use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, Invalidation, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_shadcn::{Button, ButtonSize, ButtonVariant, Spinner};

pub type OnSpeechInputListeningChange =
    Arc<dyn Fn(&mut dyn fret_ui::action::UiActionHost, ActionCx, bool) + 'static>;

#[derive(Clone)]
pub struct SpeechInput {
    listening: Option<Model<bool>>,
    default_listening: bool,
    processing: Option<Model<bool>>,
    default_processing: bool,
    disabled: bool,
    on_listening_change: Option<OnSpeechInputListeningChange>,
    test_id: Option<Arc<str>>,
}

impl std::fmt::Debug for SpeechInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpeechInput")
            .field("listening", &self.listening.as_ref().map(|m| m.id()))
            .field("default_listening", &self.default_listening)
            .field("processing", &self.processing.as_ref().map(|m| m.id()))
            .field("default_processing", &self.default_processing)
            .field("disabled", &self.disabled)
            .field(
                "has_on_listening_change",
                &self.on_listening_change.is_some(),
            )
            .field("test_id", &self.test_id.as_deref())
            .finish()
    }
}

impl SpeechInput {
    pub fn new() -> Self {
        Self {
            listening: None,
            default_listening: false,
            processing: None,
            default_processing: false,
            disabled: false,
            on_listening_change: None,
            test_id: None,
        }
    }

    /// Controlled listening model (whether we're recording/listening).
    pub fn listening_model(mut self, model: Model<bool>) -> Self {
        self.listening = Some(model);
        self
    }

    /// Uncontrolled initial listening value.
    pub fn default_listening(mut self, listening: bool) -> Self {
        self.default_listening = listening;
        self
    }

    /// Controlled processing model (e.g. waiting for ASR).
    pub fn processing_model(mut self, model: Model<bool>) -> Self {
        self.processing = Some(model);
        self
    }

    /// Uncontrolled initial processing value.
    pub fn default_processing(mut self, processing: bool) -> Self {
        self.default_processing = processing;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Intent seam: app-owned backends should start/stop capture and drive transcription.
    pub fn on_listening_change(mut self, cb: OnSpeechInputListeningChange) -> Self {
        self.on_listening_change = Some(cb);
        self
    }

    pub fn test_id(mut self, id: impl Into<Arc<str>>) -> Self {
        self.test_id = Some(id.into());
        self
    }

    pub fn into_element<H: UiHost + 'static>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        cx.scope(|cx| {
            let listening =
                controllable_state::use_controllable_model(cx, self.listening.clone(), || {
                    self.default_listening
                })
                .model();
            let processing =
                controllable_state::use_controllable_model(cx, self.processing.clone(), || {
                    self.default_processing
                })
                .model();

            let listening_now = cx
                .get_model_copied(&listening, Invalidation::Paint)
                .unwrap_or(false);
            let processing_now = cx
                .get_model_copied(&processing, Invalidation::Paint)
                .unwrap_or(false);

            let disabled = self.disabled || processing_now;
            let on_listening_change = self.on_listening_change.clone();

            let icon = if processing_now {
                Spinner::new().into_element(cx)
            } else if listening_now {
                fret_ui_kit::declarative::icon::icon(
                    cx,
                    fret_icons::IconId::new_static("lucide.square"),
                )
            } else {
                fret_ui_kit::declarative::icon::icon(
                    cx,
                    fret_icons::IconId::new_static("lucide.mic"),
                )
            };

            let on_activate: fret_ui::action::OnActivate = Arc::new(move |host, action_cx, _| {
                if host.models_mut().get_copied(&processing).unwrap_or(false) {
                    return;
                }

                let mut next = false;
                let _ = host.models_mut().update(&listening, |v| {
                    *v = !*v;
                    next = *v;
                });

                if let Some(cb) = on_listening_change.clone() {
                    cb(host, action_cx, next);
                }

                host.notify(action_cx);
                host.request_redraw(action_cx.window);
            });

            let label: Arc<str> = Arc::from(if listening_now {
                "Stop recording"
            } else {
                "Start recording"
            });

            let mut btn = Button::new(label)
                .size(ButtonSize::Icon)
                .variant(if listening_now {
                    ButtonVariant::Destructive
                } else {
                    ButtonVariant::Default
                })
                .children([icon])
                .disabled(disabled)
                .on_activate(on_activate);

            if let Some(test_id) = self.test_id {
                btn = btn.test_id(test_id);
            }

            btn.into_element(cx)
        })
    }
}
