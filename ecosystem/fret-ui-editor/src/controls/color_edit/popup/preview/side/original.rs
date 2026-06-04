use std::sync::Arc;

use fret_core::{Color, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate};
use fret_ui::element::{AnyElement, PressableA11y, PressableProps};
use fret_ui::{ElementContext, UiHost};

use super::super::super::super::{ColorEditAlphaPreview, model::format_hex};
use super::super::fill::preview_color_for_alpha_visibility;
use super::cell::{preview_cell_content, preview_cell_layout};

pub(super) fn original_reference_preview_cell<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    original: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    show_alpha: bool,
    enabled: bool,
    alpha_preview: ColorEditAlphaPreview,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let restore: OnActivate =
        Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
            let current = host
                .models_mut()
                .get_copied(&model)
                .unwrap_or(Color::TRANSPARENT);
            let next = restore_reference_color(original, current, show_alpha);
            let formatted = format_hex(next, show_alpha);

            let _ = host.models_mut().update(&model, |color| *color = next);
            let _ = host
                .models_mut()
                .update(&draft, |text| *text = formatted.as_ref().to_string());
            let _ = host.models_mut().update(&error, |value| *value = None);
            host.request_redraw(action_cx.window);
        });

    let color = preview_color_for_alpha_visibility(original, show_alpha);
    let mut cell = cx.pressable(
        PressableProps {
            layout: preview_cell_layout(),
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(SemanticsRole::Button),
                label: Some(Arc::from("Original color")),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, _st| {
            cx.pressable_add_on_activate(restore.clone());
            vec![preview_cell_content(
                cx,
                "Original",
                color,
                show_alpha,
                alpha_preview,
            )]
        },
    );

    if let Some(test_id) = test_id {
        cell = cell.test_id(test_id);
    }
    cell.a11y_value(format_hex(color, show_alpha))
}

pub(in crate::controls::color_edit) fn restore_reference_color(
    reference: Color,
    current: Color,
    show_alpha: bool,
) -> Color {
    if show_alpha {
        reference
    } else {
        let mut next = reference;
        next.a = current.a;
        next
    }
}
