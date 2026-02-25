//! Editor-grade numeric control: drag-to-scrub with an optional typing mode.
//!
//! v1 goals (workstream):
//! - scrub (drag-to-change) with Shift slow / Alt fast outcomes,
//! - double-click to switch into a typing mode,
//! - Escape cancels scrub to the pre-edit value (handled by `DragValueCore`).

use std::panic::Location;
use std::sync::{Arc, Mutex};

use crate::controls::numeric_input::{
    NumericFormatFn, NumericInput, NumericInputErrorDisplay, NumericInputOptions,
    NumericInputOutcome, NumericParseFn, NumericValidateFn,
};
use crate::primitives::drag_value_core::DragValueScalar;
use crate::primitives::style::EditorStyle;
use crate::primitives::visuals::{EditorFrameState, EditorWidgetVisuals};
use crate::primitives::{DragValueCore, DragValueCoreOptions};
use fret_core::text::{TextOverflow, TextWrap};
use fret_core::{Corners, Edges, Px, TextAlign, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, PointerDownCx, PressablePointerDownResult, UiActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, FlexItemStyle, InsetStyle, LayoutStyle, Length, Overflow,
    PositionStyle, SizeStyle, TextProps,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::typography;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DragValueMode {
    Scrub,
    Typing,
}

#[derive(Debug)]
struct DragValueState {
    mode: DragValueMode,
    scrub_id: Option<fret_ui::GlobalElementId>,
    input_id: Option<fret_ui::GlobalElementId>,
}

impl Default for DragValueState {
    fn default() -> Self {
        Self {
            mode: DragValueMode::Scrub,
            scrub_id: None,
            input_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DragValueOptions {
    pub layout: LayoutStyle,
    /// Explicit identity source for internal state (scrub/typing focus restore).
    ///
    /// This is the editor-control equivalent of egui's `id_source(...)` / ImGui's `PushID`.
    /// Use this when a helper function builds multiple drag values from the same callsite and
    /// you need stable, per-instance state separation.
    pub id_source: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
}

impl Default for DragValueOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    grow: 1.0,
                    basis: Length::Px(Px(0.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            id_source: None,
            test_id: None,
        }
    }
}

#[derive(Clone)]
pub struct DragValue<T> {
    model: Model<T>,
    format: NumericFormatFn<T>,
    parse: NumericParseFn<T>,
    validate: Option<NumericValidateFn<T>>,
    options: DragValueOptions,
}

impl<T> DragValue<T>
where
    T: DragValueScalar + Default,
{
    pub fn new(model: Model<T>, format: NumericFormatFn<T>, parse: NumericParseFn<T>) -> Self {
        Self {
            model,
            format,
            parse,
            validate: None,
            options: DragValueOptions::default(),
        }
    }

    pub fn validate(mut self, validate: Option<NumericValidateFn<T>>) -> Self {
        self.validate = validate;
        self
    }

    pub fn options(mut self, options: DragValueOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let model_id = self.model.id();
        let loc = Location::caller();
        let callsite = (loc.file(), loc.line(), loc.column());
        let id_source = self.options.id_source.clone();

        if let Some(id_source) = id_source.as_deref() {
            cx.keyed(("fret-ui-editor.drag_value", id_source, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        } else {
            cx.keyed(("fret-ui-editor.drag_value", callsite, model_id), |cx| {
                self.into_element_keyed(cx)
            })
        }
    }

    fn into_element_keyed<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let state: Arc<Mutex<DragValueState>> = cx.with_state(
            || Arc::new(Mutex::new(DragValueState::default())),
            |s| s.clone(),
        );

        let value = cx
            .get_model_copied(&self.model, Invalidation::Paint)
            .unwrap_or_default();
        let value_text = (self.format)(value);

        let mode = state.lock().unwrap_or_else(|e| e.into_inner()).mode;

        let typing = mode == DragValueMode::Typing;

        let (density, scrub_chrome) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            (style.density, style.frame_chrome_small())
        };

        let model_for_change = self.model.clone();
        let on_change_live: Arc<dyn Fn(&mut dyn UiActionHost, ActionCx, T) + 'static> =
            Arc::new(move |host, action_cx, next| {
                let _ = host.models_mut().update(&model_for_change, |v| *v = next);
                host.request_redraw(action_cx.window);
            });

        let mut scrub_opts = DragValueCoreOptions::default();
        scrub_opts.layout = if typing {
            hidden_layout(self.options.layout)
        } else {
            self.options.layout
        };
        scrub_opts.enabled = mode == DragValueMode::Scrub;
        scrub_opts.scrub_on_double_click = false;

        let state_for_scrub_record = state.clone();
        let scrub = DragValueCore::new(value, on_change_live)
            .a11y_label(value_text.clone())
            .options(scrub_opts)
            .into_element(cx, move |cx, resp| {
                // Record the scrub element id for focus restore from typing mode.
                let scrub_id = cx.root_id();
                let mut st = state_for_scrub_record
                    .lock()
                    .unwrap_or_else(|e| e.into_inner());
                st.scrub_id = Some(scrub_id);

                let state_for_double_click = state_for_scrub_record.clone();
                cx.pressable_add_on_pointer_down(Arc::new(
                    move |host, action_cx, down: PointerDownCx| {
                        if down.click_count < 2 {
                            return PressablePointerDownResult::Continue;
                        }

                        let mut st = state_for_double_click
                            .lock()
                            .unwrap_or_else(|e| e.into_inner());
                        st.mode = DragValueMode::Typing;
                        if let Some(input_id) = st.input_id {
                            host.request_focus(input_id);
                        }
                        host.request_redraw(action_cx.window);
                        PressablePointerDownResult::SkipDefaultAndStopPropagation
                    },
                ));

                let theme = Theme::global(&*cx.app);
                let visuals = EditorWidgetVisuals::new(theme).frame_visuals(
                    scrub_chrome,
                    EditorFrameState {
                        enabled: true,
                        hovered: resp.hovered,
                        pressed: resp.dragging || resp.pressed,
                        focused: resp.focused || cx.is_focused_element(scrub_id),
                        open: false,
                    },
                );

                vec![cx.container(
                    ContainerProps {
                        layout: LayoutStyle {
                            size: SizeStyle {
                                width: Length::Fill,
                                height: Length::Fill,
                                min_height: Some(Length::Px(density.row_height)),
                                ..Default::default()
                            },
                            ..Default::default()
                        },
                        padding: scrub_chrome.padding.into(),
                        background: Some(visuals.bg),
                        border: Edges::all(scrub_chrome.border_width),
                        border_color: Some(visuals.border),
                        corner_radii: Corners::all(scrub_chrome.radius),
                        ..Default::default()
                    },
                    move |cx| {
                        vec![cx.text_props(TextProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Fill,
                                    height: Length::Auto,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            text: value_text.clone(),
                            style: Some(typography::as_control_text(TextStyle {
                                size: scrub_chrome.text_px,
                                line_height: Some(density.row_height),
                                ..Default::default()
                            })),
                            color: Some(visuals.fg),
                            wrap: TextWrap::None,
                            overflow: TextOverflow::Ellipsis,
                            align: TextAlign::Start,
                            ink_overflow: Default::default(),
                        })]
                    },
                )]
            });

        let mut input_layout = self.options.layout;
        if !typing {
            input_layout = hidden_layout(input_layout);
        }

        let state_for_input = state.clone();
        let input = NumericInput::new(self.model.clone(), self.format.clone(), self.parse.clone())
            .validate(self.validate.clone())
            .options(NumericInputOptions {
                layout: input_layout,
                enabled: typing,
                focusable: typing,
                test_id: self.options.test_id.clone(),
                // Avoid growing the row height when a commit-time validation error occurs.
                // A small trailing status icon keeps the inspector layout stable.
                error_display: NumericInputErrorDisplay::TrailingIcon,
                ..Default::default()
            })
            .on_outcome(Some(Arc::new(move |host, action_cx, outcome| {
                let mut st = state_for_input.lock().unwrap_or_else(|e| e.into_inner());
                match outcome {
                    NumericInputOutcome::Committed | NumericInputOutcome::Canceled => {
                        st.mode = DragValueMode::Scrub;
                        if let Some(scrub_id) = st.scrub_id {
                            host.request_focus(scrub_id);
                        }
                        host.request_redraw(action_cx.window);
                    }
                }
            })))
            .into_element(cx);

        {
            let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
            st.input_id = Some(input.id);
        }

        // Render both: scrub stays mounted so focus can restore, input stays mounted so focus
        // requests have a stable target.
        cx.container(Default::default(), move |_cx| vec![scrub, input])
    }
}

fn hidden_layout(mut layout: LayoutStyle) -> LayoutStyle {
    layout.size = SizeStyle {
        width: Length::Px(Px(0.0)),
        height: Length::Px(Px(0.0)),
        min_width: Some(Length::Px(Px(0.0))),
        min_height: Some(Length::Px(Px(0.0))),
        ..Default::default()
    };
    layout.position = PositionStyle::Absolute;
    layout.inset = InsetStyle {
        top: Some(Px(0.0)).into(),
        left: Some(Px(0.0)).into(),
        ..Default::default()
    };
    layout.overflow = Overflow::Clip;
    layout
}
