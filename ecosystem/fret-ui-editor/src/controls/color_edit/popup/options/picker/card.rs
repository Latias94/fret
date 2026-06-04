use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::colors::{editor_accent, editor_border};
use crate::primitives::popup_list::{
    EditorPopupListRowState, editor_popup_list_row_palette, editor_popup_list_row_radius,
};
use crate::primitives::readout::editor_popup_list_option_caption_text_props;

use super::super::super::super::model::hsv_from_color;
use super::super::super::super::{ColorEditPopupPicker, ColorEditPopupRuntimeOptions};
use super::super::thumbnail::{PICKER_OPTION_THUMBNAIL_HEIGHT, picker_option_thumbnail};

pub(super) fn picker_option_button<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    picker: ColorEditPopupPicker,
    current: Color,
    selected: bool,
    runtime_model: Model<ColorEditPopupRuntimeOptions>,
    enabled: bool,
    row_height: Px,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let on_activate: OnActivate =
        Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
            let _ = host
                .models_mut()
                .update(&runtime_model, |runtime| runtime.picker = picker);
            host.request_redraw(action_cx.window);
        });

    let hsv = hsv_from_color(current);
    let label = Arc::<str>::from(label);
    let a11y_label = label.clone();
    let card_height = Px(PICKER_OPTION_THUMBNAIL_HEIGHT.0 + row_height.0 + 6.0);
    let mut button = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(card_height),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(SemanticsRole::RadioButton),
                label: Some(a11y_label),
                checked: Some(selected),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            cx.pressable_add_on_activate(on_activate.clone());
            let (bg, fg, border) = {
                let theme = Theme::global(&*cx.app);
                let palette = editor_popup_list_row_palette(
                    theme,
                    st.hovered || st.hovered_raw,
                    EditorPopupListRowState {
                        active: selected,
                        disabled: !enabled,
                    },
                );
                let border = if selected {
                    editor_accent(theme)
                } else {
                    editor_border(theme)
                };
                (palette.bg, palette.fg, border)
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
                    padding: Edges::all(Px(4.0)).into(),
                    background: bg,
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border),
                    corner_radii: Corners::all(editor_popup_list_row_radius()),
                    ..Default::default()
                },
                {
                    let label = label.clone();
                    move |cx| {
                        vec![cx.flex(
                            FlexProps {
                                layout: LayoutStyle {
                                    size: SizeStyle {
                                        width: Length::Fill,
                                        height: Length::Fill,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                direction: Axis::Vertical,
                                gap: SpacingLength::Px(Px(4.0)),
                                padding: Edges::all(Px(0.0)).into(),
                                justify: MainAlign::Start,
                                align: CrossAlign::Center,
                                wrap: false,
                            },
                            {
                                let label = label.clone();
                                move |cx| {
                                    vec![
                                        picker_option_thumbnail(cx, picker, hsv),
                                        cx.text_props(editor_popup_list_option_caption_text_props(
                                            label.clone(),
                                            fg,
                                            row_height,
                                        )),
                                    ]
                                }
                            },
                        )]
                    }
                },
            )]
        },
    );

    if let Some(test_id) = test_id {
        button = button.test_id(test_id);
    }
    button
}
