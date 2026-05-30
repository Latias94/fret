use std::sync::Arc;

use fret_core::{Color, KeyCode, Px};
use fret_runtime::Model;
use fret_ui::action::ActionCx;
use fret_ui::element::{
    AnyElement, LayoutStyle, Length, PointerRegionProps, SizeStyle, TextInputProps,
};
use fret_ui::{ElementContext, Theme, UiHost};
use fret_ui_kit::{ChromeRefinement, Size};

use crate::primitives::chrome::resolve_editor_text_field_style;

use super::model::{format_hex, parse_hex};

pub(super) struct ColorEditInputArgs {
    pub(super) model: Model<Color>,
    pub(super) draft: Model<String>,
    pub(super) error: Model<Option<Arc<str>>>,
    pub(super) current_hex: Arc<str>,
    pub(super) show_alpha: bool,
    pub(super) enabled: bool,
    pub(super) focusable: bool,
    pub(super) test_id: Option<Arc<str>>,
    pub(super) row_height: Px,
}

pub(super) fn color_hex_input<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: ColorEditInputArgs,
) -> AnyElement {
    let (chrome, text_style) = {
        let theme = Theme::global(&*cx.app);
        let (chrome, text_style) =
            resolve_editor_text_field_style(theme, Size::default(), &ChromeRefinement::default());
        (chrome, text_style)
    };

    // Keep the draft synced while not focused so external updates (undo, scripts) show up.
    let mut props = TextInputProps::new(args.draft.clone());
    props.layout = LayoutStyle {
        size: SizeStyle {
            width: Length::Fill,
            height: Length::Auto,
            min_height: Some(Length::Px(args.row_height)),
            ..Default::default()
        },
        ..Default::default()
    };
    props.enabled = args.enabled;
    props.focusable = args.focusable;
    props.test_id = args.test_id.clone();
    props.chrome = chrome;
    props.text_style = text_style;

    let input = cx.text_input(props);
    let input_id = input.id;
    let is_focused = cx.is_focused_element(input_id);

    if !is_focused {
        let _ = cx
            .app
            .models_mut()
            .update(&args.draft, |s| *s = args.current_hex.as_ref().to_string());
        let _ = cx.app.models_mut().update(&args.error, |e| *e = None);
    }

    let model_for_key = args.model.clone();
    let draft_for_key = args.draft.clone();
    let error_for_key = args.error.clone();
    let show_alpha = args.show_alpha;
    cx.key_add_on_key_down_capture_for(
        input_id,
        Arc::new(move |host, action_cx: ActionCx, down| match down.key {
            KeyCode::Enter | KeyCode::NumpadEnter => {
                let text = host
                    .models_mut()
                    .read(&draft_for_key, |s| s.clone())
                    .unwrap_or_default();
                let current = host
                    .models_mut()
                    .get_copied(&model_for_key)
                    .unwrap_or(Color::TRANSPARENT);
                if let Some(next) = parse_hex(&text, show_alpha, current) {
                    let _ = host.models_mut().update(&model_for_key, |c| *c = next);
                    let _ = host.models_mut().update(&error_for_key, |e| *e = None);
                } else {
                    let _ = host
                        .models_mut()
                        .update(&error_for_key, |e| *e = Some(Arc::from("Invalid color")));
                }
                host.request_redraw(action_cx.window);
                true
            }
            KeyCode::Escape => {
                let current = host
                    .models_mut()
                    .get_copied(&model_for_key)
                    .unwrap_or_else(|| Color::from_srgb_hex_rgb(0x00_00_00));
                let formatted = format_hex(current, show_alpha);
                let _ = host
                    .models_mut()
                    .update(&draft_for_key, |s| *s = formatted.as_ref().to_string());
                let _ = host.models_mut().update(&error_for_key, |e| *e = None);
                host.request_redraw(action_cx.window);
                true
            }
            _ => false,
        }),
    );

    cx.pointer_region(
        PointerRegionProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    min_height: Some(Length::Px(args.row_height)),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: args.enabled && args.focusable,
            capture_phase_pointer_moves: false,
        },
        move |cx| {
            cx.pointer_region_on_pointer_down(Arc::new(move |host, action_cx, _down| {
                host.request_focus(input_id);
                host.request_redraw(action_cx.window);
                false
            }));
            vec![input]
        },
    )
}
