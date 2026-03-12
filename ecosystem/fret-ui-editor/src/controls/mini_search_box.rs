//! Minimal search box control for editor filtering surfaces.
//!
//! v1 scope:
//! - single-line text input bound to a `Model<String>`
//! - optional clear button
//! - stable `test_id` anchors

use std::sync::Arc;

use fret_runtime::Model;
use fret_ui::action::{ActivateReason, OnActivate};
use fret_ui::element::{AnyElement, LayoutStyle, Length, SizeStyle, TextInputProps};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::{ChromeRefinement, Size};

use crate::primitives::chrome::{joined_text_input_style, resolve_editor_text_field_style};
use crate::primitives::input_group::{
    derived_test_id, editor_clear_button_segment, editor_joined_input_frame,
};
use crate::primitives::style::EditorStyle;
use crate::primitives::text_entry::{
    EditorTextCancelBehavior, EditorTextSelectionBehavior, editor_text_entry_focus_state,
    sync_editor_text_entry_focus_selection,
};

#[derive(Debug, Clone)]
pub struct MiniSearchBoxOptions {
    pub layout: LayoutStyle,
    pub size: Size,
    pub placeholder: Option<Arc<str>>,
    pub enabled: bool,
    pub focusable: bool,
    pub test_id: Option<Arc<str>>,
    pub input_test_id: Option<Arc<str>>,
    pub clear_test_id: Option<Arc<str>>,
    pub selection_behavior: EditorTextSelectionBehavior,
    pub cancel_behavior: EditorTextCancelBehavior,
}

impl Default for MiniSearchBoxOptions {
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
            placeholder: Some(Arc::from("Search…")),
            enabled: true,
            focusable: true,
            test_id: None,
            input_test_id: None,
            clear_test_id: None,
            selection_behavior: EditorTextSelectionBehavior::SelectAllOnFocus,
            cancel_behavior: EditorTextCancelBehavior::Clear,
        }
    }
}

#[derive(Clone)]
pub struct MiniSearchBox {
    model: Model<String>,
    options: MiniSearchBoxOptions,
}

impl MiniSearchBox {
    pub fn new(model: Model<String>) -> Self {
        Self {
            model,
            options: MiniSearchBoxOptions::default(),
        }
    }

    pub fn options(mut self, options: MiniSearchBoxOptions) -> Self {
        self.options = options;
        self
    }

    #[track_caller]
    pub fn into_element<H: UiHost>(self, cx: &mut ElementContext<'_, H>) -> AnyElement {
        let MiniSearchBox { model, options } = self;

        let has_value = cx
            .read_model_ref(&model, Invalidation::Layout, |s| !s.is_empty())
            .unwrap_or(false);
        let enabled_for_paint = options.enabled;
        let selection_behavior = options.selection_behavior;
        let clear_test_id = options.clear_test_id.clone();
        let input_test_id = options
            .input_test_id
            .clone()
            .or_else(|| derived_test_id(options.test_id.as_ref(), "input"));
        let focus_state = editor_text_entry_focus_state(cx);

        let (density, frame_chrome, chrome, text_style) = {
            let theme = Theme::global(&*cx.app);
            let style = EditorStyle::resolve(theme);
            let density = style.density;
            let frame_chrome = style.frame_chrome_small();
            let (chrome, text_style) =
                resolve_editor_text_field_style(theme, options.size, &ChromeRefinement::default());
            (density, frame_chrome, chrome, text_style)
        };

        let mut input_props = TextInputProps::new(model.clone());
        input_props.layout = LayoutStyle {
            size: SizeStyle {
                width: Length::Fill,
                height: Length::Fill,
                min_height: Some(Length::Px(density.row_height)),
                ..Default::default()
            },
            ..Default::default()
        };
        input_props.enabled = options.enabled;
        input_props.focusable = options.focusable;
        input_props.placeholder = options.placeholder.clone();
        input_props.test_id = input_test_id;
        if matches!(options.cancel_behavior, EditorTextCancelBehavior::Clear) {
            input_props.cancel_command = Some("text.clear".into());
        }

        // Joined field: the frame is drawn by the input group. Keep the inner text input
        // transparent and borderless to avoid double chrome.
        input_props.chrome = joined_text_input_style(chrome);
        input_props.text_style = text_style;

        let model_for_trailing = model.clone();

        editor_joined_input_frame(
            cx,
            options.layout,
            density,
            frame_chrome,
            enabled_for_paint,
            false,
            options.test_id.clone(),
            move |cx| {
                let input = cx.text_input(input_props);
                let input_id = input.id;
                let is_focused = cx.is_focused_element(input_id);
                sync_editor_text_entry_focus_selection(
                    cx,
                    &focus_state,
                    input_id,
                    is_focused,
                    has_value,
                    selection_behavior,
                );
                input
            },
            move |cx| {
                if !has_value {
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
                    Arc::from("Clear search"),
                    clear_test_id.clone(),
                    on_activate,
                )]
            },
        )
    }
}
