mod button;
mod picker;
mod thumbnail;

use std::sync::Arc;

use fret_core::{Axis, Color, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate};
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::EditorDensity;
use crate::primitives::input_group::derived_test_id;

pub(super) use button::option_button;
use picker::picker_options_row;

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
