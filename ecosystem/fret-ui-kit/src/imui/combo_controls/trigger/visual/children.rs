use std::sync::Arc;

use fret_ui::element::AnyElement;
use fret_ui::{ElementContext, UiHost};

use crate::imui::control_chrome::{self, ImUiControlPalette};

pub(in crate::imui::combo_controls::trigger) fn combo_trigger_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    preview: Arc<str>,
    open: bool,
    palette: ImUiControlPalette,
) -> Vec<AnyElement> {
    let state_badge = combo_trigger_state_badge(cx, open, palette);

    vec![cx.flex(control_chrome::fill_stack_props(), move |cx| {
        let mut out = Vec::new();
        if !label.is_empty() {
            out.push(control_chrome::caption_text(cx, label.clone(), palette));
        }
        out.push(cx.flex(
            control_chrome::fill_row_props(fret_ui::element::MainAlign::SpaceBetween),
            move |cx| {
                vec![
                    control_chrome::fill_text(cx, preview.clone(), palette.foreground),
                    state_badge,
                ]
            },
        ));
        out
    })]
}

fn combo_trigger_state_badge<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    palette: ImUiControlPalette,
) -> AnyElement {
    if open {
        control_chrome::pill(
            cx,
            Arc::from("Open"),
            palette.accent_background,
            palette.accent_foreground,
        )
    } else {
        control_chrome::pill(
            cx,
            Arc::from("Menu"),
            palette.subtle_background,
            palette.muted_foreground,
        )
    }
}
