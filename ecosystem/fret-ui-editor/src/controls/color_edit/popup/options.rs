mod button;
mod thumbnail;

use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::colors::{editor_accent, editor_border};
use crate::primitives::input_group::derived_test_id;
use crate::primitives::popup_list::{
    EditorPopupListRowState, editor_popup_list_row_palette, editor_popup_list_row_radius,
};
use crate::primitives::readout::editor_popup_list_option_caption_text_props;

pub(super) use button::option_button;
use thumbnail::{PICKER_OPTION_THUMBNAIL_HEIGHT, picker_option_thumbnail};

use super::super::model::hsv_from_color;
use super::super::{ColorEditPopupOptions, ColorEditPopupPicker, ColorEditPopupRuntimeOptions};

pub(super) fn color_picker_options<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    popup_options: ColorEditPopupOptions,
    runtime_options: ColorEditPopupRuntimeOptions,
    runtime_model: Model<ColorEditPopupRuntimeOptions>,
    show_alpha: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let density = {
        let theme = Theme::global(&*cx.app);
        EditorDensity::resolve(theme)
    };
    let picker_test_id = derived_test_id(test_id.as_ref(), "picker");
    let alpha_test_id = derived_test_id(test_id.as_ref(), "alpha-bar");

    let mut options = cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: Axis::Vertical,
            gap: SpacingLength::Px(Px(4.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |cx| {
            let mut out = Vec::new();
            if popup_options.picker != ColorEditPopupPicker::Hidden {
                out.push(picker_options_row(
                    cx,
                    current,
                    runtime_options,
                    runtime_model.clone(),
                    enabled,
                    density.row_height,
                    picker_test_id.clone(),
                ));
            }
            if show_alpha {
                out.push(alpha_bar_option(
                    cx,
                    runtime_options,
                    runtime_model.clone(),
                    enabled,
                    density.row_height,
                    alpha_test_id.clone(),
                ));
            }
            out
        },
    );

    if let Some(test_id) = test_id {
        options = options.test_id(test_id);
    }
    options
}

fn picker_options_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    runtime_options: ColorEditPopupRuntimeOptions,
    runtime_model: Model<ColorEditPopupRuntimeOptions>,
    enabled: bool,
    row_height: Px,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let hue_bar_test_id = derived_test_id(test_id.as_ref(), "hue-bar");
    let hue_wheel_test_id = derived_test_id(test_id.as_ref(), "hue-wheel");
    let mut row = cx.flex(
        FlexProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            direction: Axis::Horizontal,
            gap: SpacingLength::Px(Px(4.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Center,
            wrap: false,
        },
        move |cx| {
            vec![
                picker_option_button(
                    cx,
                    "Hue Bar",
                    ColorEditPopupPicker::HsvHueBar,
                    current,
                    runtime_options.picker == ColorEditPopupPicker::HsvHueBar,
                    runtime_model.clone(),
                    enabled,
                    row_height,
                    hue_bar_test_id.clone(),
                ),
                picker_option_button(
                    cx,
                    "Hue Wheel",
                    ColorEditPopupPicker::HsvHueWheel,
                    current,
                    runtime_options.picker == ColorEditPopupPicker::HsvHueWheel,
                    runtime_model.clone(),
                    enabled,
                    row_height,
                    hue_wheel_test_id.clone(),
                ),
            ]
        },
    );

    if let Some(test_id) = test_id {
        row = row.test_id(test_id);
    }
    row
}

fn picker_option_button<H: UiHost>(
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

fn alpha_bar_option<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    runtime_options: ColorEditPopupRuntimeOptions,
    runtime_model: Model<ColorEditPopupRuntimeOptions>,
    enabled: bool,
    row_height: Px,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let selected = runtime_options.alpha_bar;
    let on_activate: OnActivate =
        Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
            let _ = host.models_mut().update(&runtime_model, |runtime| {
                runtime.alpha_bar = !runtime.alpha_bar
            });
            host.request_redraw(action_cx.window);
        });

    option_button(
        cx,
        "Alpha Bar",
        SemanticsRole::Checkbox,
        selected,
        enabled,
        row_height,
        test_id,
        on_activate,
    )
}
