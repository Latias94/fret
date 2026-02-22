//! Reusable editor text field control.
//!
//! v1 scope:
//! - single-line input (`TextInput`)
//! - optional multiline mode (`TextArea`) with a minimum height
//! - optional clear affordance

use std::sync::Arc;

use fret_core::{Px, TextStyle};
use fret_runtime::Model;
use fret_ui::action::{ActivateReason, OnActivate};
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle, TextAreaProps, TextInputProps};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::{ChromeRefinement, Size};

use crate::primitives::chrome::{
    joined_text_area_style, joined_text_input_style, resolve_editor_text_area_field_style,
    resolve_editor_text_field_style,
};
use crate::primitives::input_group::{editor_clear_button_segment, editor_joined_input_frame};
use crate::primitives::style::EditorStyle;

#[derive(Debug, Clone)]
pub struct TextFieldOptions {
    pub layout: LayoutStyle,
    pub size: Size,
    pub placeholder: Option<Arc<str>>,
    pub enabled: bool,
    pub focusable: bool,
    pub clear_button: bool,
    pub a11y_label: Option<Arc<str>>,
    pub test_id: Option<Arc<str>>,
    pub clear_test_id: Option<Arc<str>>,

    /// When true, uses `TextArea` (multiline) instead of `TextInput`.
    pub multiline: bool,
    /// Minimum height for multiline text areas.
    pub min_height: Option<Px>,
}

impl Default for TextFieldOptions {
    fn default() -> Self {
        Self {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            size: Size::Small,
            placeholder: None,
            enabled: true,
            focusable: true,
            clear_button: false,
            a11y_label: None,
            test_id: None,
            clear_test_id: None,
            multiline: false,
            min_height: None,
        }
    }
}

#[derive(Clone)]
pub struct TextField {
    model: Model<String>,
    options: TextFieldOptions,
}

impl TextField {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            options: TextFieldOptions::default(),
        }
    }

    pub fn options(mut self, options: TextFieldOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let TextField { model, options } = self;

        let layout = options.layout;
        let size = options.size;
        let placeholder = options.placeholder.clone();
        let enabled_for_paint = options.enabled;
        let focusable = options.focusable;
        let clear_button = options.clear_button;
        let a11y_label = options.a11y_label.clone();
        let test_id = options.test_id.clone();
        let clear_test_id = options.clear_test_id.clone();
        let multiline = options.multiline;
        let min_height = options.min_height;

        let has_value = cx
            .read_model_ref(&model, Invalidation::Layout, |s| !s.is_empty())
            .unwrap_or(false);

        let clear_enabled = clear_button && has_value && enabled_for_paint;

        let (density, frame_chrome) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            (style.density, style.frame_chrome(size))
        };

        let model_for_input = model.clone();
        let model_for_trailing = model.clone();

        editor_joined_input_frame(
            cx,
            layout,
            density,
            frame_chrome,
            enabled_for_paint,
            false,
            None,
            move |cx| {
                if multiline {
                    let (chrome, text_style) = {
                        let theme = Theme::global(&*cx.app);
                        resolve_editor_text_area_field_style(
                            theme,
                            size,
                            &ChromeRefinement::default(),
                        )
                    };

                    let mut props = TextAreaProps::new(model_for_input.clone());
                    props.layout = LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    props.enabled = enabled_for_paint;
                    props.focusable = focusable;
                    props.a11y_label = a11y_label.clone();
                    props.test_id = test_id.clone();

                    props.chrome = joined_text_area_style(chrome);
                    props.text_style = text_style;
                    props.min_height = min_height.unwrap_or_else(|| {
                        let baseline = Px(80.0);
                        let dense = Px(density.row_height.0 * 3.0);
                        Px(baseline.0.max(dense.0))
                    });

                    cx.text_area(props)
                } else {
                    let (chrome, text_style) = {
                        let theme = Theme::global(&*cx.app);
                        resolve_editor_text_field_style(theme, size, &ChromeRefinement::default())
                    };

                    let mut props = TextInputProps::new(model_for_input.clone());
                    props.layout = LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            min_height: Some(density.row_height),
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    props.enabled = enabled_for_paint;
                    props.focusable = focusable;
                    props.placeholder = placeholder.clone();
                    props.a11y_label = a11y_label.clone();
                    props.test_id = test_id.clone();

                    props.chrome = joined_text_input_style(chrome);
                    props.text_style = TextStyle {
                        line_height: Some(density.row_height),
                        line_height_policy: fret_core::TextLineHeightPolicy::FixedFromStyle,
                        ..text_style
                    };

                    cx.text_input(props)
                }
            },
            move |cx| {
                if !clear_enabled {
                    return Vec::new();
                }

                let model_for_clear = model_for_trailing.clone();
                let on_activate: OnActivate =
                    Arc::new(move |host, action_cx, _reason: ActivateReason| {
                        let _ = host.models_mut().update(&model_for_clear, |s| s.clear());
                        host.request_redraw(action_cx.window);
                    });

                vec![editor_clear_button_segment(
                    cx,
                    density,
                    enabled_for_paint,
                    Arc::from("Clear text"),
                    clear_test_id.clone(),
                    on_activate,
                )]
            },
        )
    }
}
