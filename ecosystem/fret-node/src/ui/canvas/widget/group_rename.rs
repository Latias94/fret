use fret_core::{KeyCode, Modifiers};
use fret_ui::UiHost;

use crate::ops::GraphOp;

use super::NodeGraphCanvas;

pub(super) fn handle_group_rename_escape<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
) -> bool {
    if canvas.interaction.group_rename.take().is_some() {
        cx.stop_propagation();
        cx.request_redraw();
        cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
        return true;
    }
    false
}

pub(super) fn handle_group_rename_text_input<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    text: &str,
) -> bool {
    let Some(rename) = canvas.interaction.group_rename.as_mut() else {
        return false;
    };

    for ch in text.chars() {
        if ch == '\r' || ch == '\n' {
            continue;
        }
        rename.text.push(ch);
    }

    cx.stop_propagation();
    cx.request_redraw();
    cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
    true
}

pub(super) fn handle_group_rename_key_down<H: UiHost>(
    canvas: &mut NodeGraphCanvas,
    cx: &mut fret_ui::retained_bridge::EventCx<'_, H>,
    key: KeyCode,
    _modifiers: Modifiers,
) -> bool {
    if canvas.interaction.group_rename.is_none() {
        return false;
    }

    match key {
        KeyCode::Escape => {
            let _ = canvas.interaction.group_rename.take();
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            true
        }
        KeyCode::Backspace => {
            let Some(rename) = canvas.interaction.group_rename.as_mut() else {
                return false;
            };
            if !rename.text.is_empty() {
                rename.text.pop();
            }
            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            true
        }
        KeyCode::Enter | KeyCode::NumpadEnter => {
            let Some(rename) = canvas.interaction.group_rename.take() else {
                return false;
            };

            let from = canvas
                .graph
                .read_ref(cx.app, |g| {
                    g.groups.get(&rename.group).map(|gg| gg.title.clone())
                })
                .ok()
                .flatten()
                .unwrap_or_else(|| rename.original.clone());
            let to: String = rename
                .text
                .chars()
                .filter(|c| *c != '\r' && *c != '\n')
                .collect();

            if from != to {
                let ops = vec![GraphOp::SetGroupTitle {
                    id: rename.group,
                    from,
                    to,
                }];
                if !canvas.commit_ops(cx.app, cx.window, Some("Rename Group"), ops) {
                    canvas.interaction.group_rename = Some(rename);
                }
            }

            cx.stop_propagation();
            cx.request_redraw();
            cx.invalidate_self(fret_ui::retained_bridge::Invalidation::Paint);
            true
        }
        _ => {
            cx.stop_propagation();
            true
        }
    }
}
