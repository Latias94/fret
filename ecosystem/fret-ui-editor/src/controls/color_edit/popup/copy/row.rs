use std::sync::Arc;

use fret_core::{Corners, Edges, Px, SemanticsRole};
use fret_runtime::{Effect, Model};
use fret_ui::action::{ActionCx, ActivateReason, OnActivate};
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableA11y, PressableProps, SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::popup_list::{
    EditorPopupListRowState, editor_popup_list_row_palette, editor_popup_list_row_radius,
};
use crate::primitives::readout::editor_popup_list_row_text_props;

use super::ColorEditCopyEntry;

pub(super) fn color_copy_menu_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    entry: ColorEditCopyEntry,
    open: Model<bool>,
    row_height: Px,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let copy_text = entry.text.clone();
    let label = entry.text.clone();
    let a11y_label = Arc::<str>::from(format!("Copy color as {}", entry.text.as_ref()));
    let on_activate: OnActivate =
        Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
            let token = host.next_clipboard_token();
            host.push_effect(Effect::ClipboardWriteText {
                window: action_cx.window,
                token,
                text: copy_text.to_string(),
            });
            let _ = host.models_mut().update(&open, |value| *value = false);
            host.request_redraw(action_cx.window);
        });

    let mut row = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(row_height),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: true,
            focusable: true,
            a11y: PressableA11y {
                role: Some(SemanticsRole::MenuItem),
                label: Some(a11y_label),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            cx.pressable_add_on_activate(on_activate.clone());
            let (bg, fg) = {
                let theme = Theme::global(&*cx.app);
                let palette = editor_popup_list_row_palette(
                    theme,
                    st.hovered || st.hovered_raw || st.focused,
                    EditorPopupListRowState::default(),
                );
                (palette.bg, palette.fg)
            };

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    padding: Edges::symmetric(Px(8.0), Px(0.0)).into(),
                    background: bg,
                    border: Edges::all(Px(0.0)),
                    corner_radii: Corners::all(editor_popup_list_row_radius()),
                    ..Default::default()
                },
                {
                    let label = label.clone();
                    move |cx| {
                        vec![cx.text_props(editor_popup_list_row_text_props(
                            label.clone(),
                            fg,
                            row_height,
                        ))]
                    }
                },
            )]
        },
    );

    if let Some(test_id) = test_id {
        row = row.test_id(test_id);
    }
    row
}
