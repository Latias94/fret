//! AI Elements-aligned `SpeechInput` surface (UI-only).
//!
//! Upstream reference: `repo-ref/ai-elements/packages/elements/src/speech-input.tsx`.
//!
//! Notes:
//! - Upstream implements browser speech recognition / MediaRecorder fallbacks.
//! - In Fret, capture and transcription backends are **app-owned**; this surface only renders the
//!   button chrome and emits an intent (`on_listening_change`).

use std::sync::Arc;
use std::time::Duration;

use fret_core::{Color, Point, Px, Transform2D};
use fret_runtime::Model;
use fret_ui::action::ActionCx;
use fret_ui::element::{AnyElement, SemanticsDecoration, VisualTransformProps};
use fret_ui::{ElementContext, Invalidation, Theme, ThemeNamedColorKey, UiHost};
use fret_ui_kit::declarative::controllable_state;
use fret_ui_kit::declarative::motion::drive_loop_progress;
use fret_ui_kit::declarative::style as decl_style;
use fret_ui_kit::{ChromeRefinement, ColorRef, LayoutRefinement, Radius};
use fret_ui_shadcn::raw::button::ButtonStyle;
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
    focusable: bool,
    variant: ButtonVariant,
    size: ButtonSize,
    a11y_label: Option<Arc<str>>,
    style: ButtonStyle,
    chrome: ChromeRefinement,
    layout: LayoutRefinement,
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
            .field("focusable", &self.focusable)
            .field("variant", &self.variant)
            .field("size", &self.size)
            .field("has_a11y_label", &self.a11y_label.is_some())
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
            focusable: true,
            variant: ButtonVariant::Default,
            size: ButtonSize::Icon,
            a11y_label: None,
            style: ButtonStyle::default(),
            chrome: ChromeRefinement::default(),
            layout: LayoutRefinement::default(),
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

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    /// Button chrome variant for the idle/listening surface.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Button size for the record control.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Overrides the semantic label announced for the icon button.
    pub fn a11y_label(mut self, label: impl Into<Arc<str>>) -> Self {
        self.a11y_label = Some(label.into());
        self
    }

    /// Extra stateful button style overrides. User-provided slots win over SpeechInput defaults.
    pub fn style(mut self, style: ButtonStyle) -> Self {
        self.style = self.style.merged(style);
        self
    }

    /// Extra chrome refinements applied on top of the rounded speech-input baseline.
    pub fn refine_style(mut self, chrome: ChromeRefinement) -> Self {
        self.chrome = self.chrome.merge(chrome);
        self
    }

    /// Layout refinements forwarded to the inner button surface.
    pub fn refine_layout(mut self, layout: LayoutRefinement) -> Self {
        self.layout = self.layout.merge(layout);
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
            let variant = self.variant;
            let size = self.size;
            let focusable = self.focusable;
            let pulse_period = Duration::from_secs(2);
            let pulse_progress = drive_loop_progress(cx, listening_now, pulse_period);
            let theme = Theme::global(&*cx.app).snapshot();
            let pulse_center = Point::new(
                Px(icon_button_diameter(&theme, size).0 / 2.0),
                Px(icon_button_diameter(&theme, size).0 / 2.0),
            );

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
                .size(size)
                .variant(if listening_now {
                    ButtonVariant::Destructive
                } else {
                    variant
                })
                .style(button_style_for_state(&theme, listening_now).merged(self.style.clone()))
                .refine_style(
                    ChromeRefinement::default()
                        .rounded(Radius::Full)
                        .merge(self.chrome.clone()),
                )
                .refine_layout(self.layout.clone())
                .children([icon])
                .disabled(disabled)
                .focusable(focusable)
                .on_activate(on_activate);

            if let Some(a11y_label) = self.a11y_label.clone() {
                btn = btn.a11y_label(a11y_label);
            }

            if let Some(test_id) = self.test_id.clone() {
                btn = btn.test_id(test_id);
            }

            let button = btn.into_element(cx);

            if !listening_now {
                return button;
            }

            let mut layers = Vec::with_capacity(4);
            for index in 0..3 {
                let ring = speech_input_pulse_ring(
                    cx,
                    &theme,
                    pulse_progress.progress,
                    pulse_center,
                    index,
                    self.test_id.as_ref(),
                );
                layers.push(ring);
            }
            layers.push(button);

            let mut props = fret_ui::element::ContainerProps::default();
            props.layout = decl_style::layout_style(
                &theme,
                LayoutRefinement::default().relative().flex_shrink_0(),
            );

            cx.container(props, move |_cx| layers)
        })
    }
}

fn button_style_for_state(theme: &fret_ui::ThemeSnapshot, listening: bool) -> ButtonStyle {
    let border = theme.color_token("border");
    let idle_bg = theme.color_token("primary");
    let idle_fg = theme.color_token("primary-foreground");
    let listening_bg = theme.color_token("destructive");
    let listening_fg = ColorRef::Named(ThemeNamedColorKey::White);

    let (base_bg, hover_bg, base_fg) = if listening {
        (listening_bg, alpha(listening_bg, 0.88), listening_fg)
    } else {
        (idle_bg, alpha(idle_bg, 0.88), ColorRef::Color(idle_fg))
    };

    ButtonStyle::default()
        .background(
            fret_ui_kit::WidgetStateProperty::new(Some(ColorRef::Color(base_bg))).when(
                fret_ui_kit::WidgetStates::HOVERED,
                Some(ColorRef::Color(hover_bg)),
            ),
        )
        .foreground(fret_ui_kit::WidgetStateProperty::new(Some(base_fg)))
        .border_color(fret_ui_kit::WidgetStateProperty::new(Some(
            ColorRef::Color(border),
        )))
}

fn speech_input_pulse_ring<H: UiHost + 'static>(
    cx: &mut ElementContext<'_, H>,
    theme: &fret_ui::ThemeSnapshot,
    progress: f32,
    center: Point,
    index: usize,
    test_id_prefix: Option<&Arc<str>>,
) -> AnyElement {
    let phase = (progress + (index as f32 / 3.0)).fract();
    let scale = 1.0 + 0.4 * phase;
    let alpha_value = 0.32 * (1.0 - phase).clamp(0.0, 1.0);
    let ring_color = alpha(theme.color_token("destructive"), alpha_value);
    let transform = Transform2D::translation(center)
        * Transform2D::scale_uniform(scale)
        * Transform2D::translation(Point::new(Px(-center.x.0), Px(-center.y.0)));

    let ring = cx.container(
        decl_style::container_props(
            theme,
            ChromeRefinement::default()
                .rounded(Radius::Full)
                .border_width(Px(2.0))
                .border_color(ColorRef::Color(ring_color)),
            LayoutRefinement::default().w_full().h_full(),
        ),
        |_cx| Vec::new(),
    );

    let ring = cx.visual_transform_props(
        VisualTransformProps {
            layout: decl_style::layout_style(
                theme,
                LayoutRefinement::default().absolute().inset_px(Px(0.0)),
            ),
            transform,
        },
        move |_cx| vec![ring],
    );

    if let Some(prefix) = test_id_prefix {
        return ring.attach_semantics(
            SemanticsDecoration::default()
                .test_id(Arc::<str>::from(format!("{prefix}-pulse-{index}"))),
        );
    }

    ring
}

fn icon_button_diameter(theme: &fret_ui::ThemeSnapshot, size: ButtonSize) -> Px {
    let size_key = match size {
        ButtonSize::Xs | ButtonSize::IconXs => "xs",
        ButtonSize::Sm | ButtonSize::IconSm => "sm",
        ButtonSize::Default | ButtonSize::Icon => "md",
        ButtonSize::Lg | ButtonSize::IconLg => "lg",
    };

    theme
        .metric_by_key(&format!("component.size.{size_key}.icon_button.size"))
        .or_else(|| theme.metric_by_key(&format!("component.size.{size_key}.button.h")))
        .unwrap_or(Px(36.0))
}

fn alpha(mut color: Color, alpha: f32) -> Color {
    color.a = alpha.clamp(0.0, 1.0);
    color
}
