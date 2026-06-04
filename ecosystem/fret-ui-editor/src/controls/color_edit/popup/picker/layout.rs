use std::sync::Arc;

use fret_core::{Axis, Color, Edges, Px};
use fret_runtime::Model;
use fret_ui::element::{
    AnyElement, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};

use crate::primitives::input_group::derived_test_id;

use super::alpha::vertical_alpha_bar;
use super::hue_bar::hue_bar;
use super::hue_wheel_picker::hue_wheel_picker;
use super::sv::sv_picker;

pub(in crate::controls::color_edit::popup) fn hsv_picker<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    show_alpha: bool,
    show_alpha_bar: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let sv_test_id = derived_test_id(test_id.as_ref(), "sv");
    let hue_test_id = derived_test_id(test_id.as_ref(), "hue");
    let alpha_test_id = derived_test_id(test_id.as_ref(), "alpha");
    let sv = sv_picker(
        cx,
        current,
        model.clone(),
        draft.clone(),
        error.clone(),
        show_alpha,
        enabled,
        sv_test_id,
    );
    let hue = hue_bar(
        cx,
        current,
        model.clone(),
        draft.clone(),
        error.clone(),
        show_alpha,
        enabled,
        hue_test_id,
    );
    let alpha = show_alpha_bar
        .then(|| vertical_alpha_bar(cx, current, model, draft, error, enabled, alpha_test_id));

    let mut picker = cx.flex(
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
            gap: SpacingLength::Px(Px(6.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |_cx| {
            let mut out = vec![sv, hue];
            if let Some(alpha) = alpha {
                out.push(alpha);
            }
            out
        },
    );

    if let Some(test_id) = test_id {
        picker = picker.test_id(test_id);
    }
    picker
}

pub(in crate::controls::color_edit::popup) fn hsv_hue_wheel_picker<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    show_alpha: bool,
    show_alpha_bar: bool,
    enabled: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let wheel_test_id = derived_test_id(test_id.as_ref(), "wheel");
    let alpha_test_id = derived_test_id(test_id.as_ref(), "alpha");
    let wheel = hue_wheel_picker(
        cx,
        current,
        model.clone(),
        draft.clone(),
        error.clone(),
        show_alpha,
        enabled,
        wheel_test_id,
    );
    let alpha = show_alpha_bar
        .then(|| vertical_alpha_bar(cx, current, model, draft, error, enabled, alpha_test_id));

    let mut picker = cx.flex(
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
            gap: SpacingLength::Px(Px(6.0)),
            padding: Edges::all(Px(0.0)).into(),
            justify: MainAlign::Start,
            align: CrossAlign::Stretch,
            wrap: false,
        },
        move |_cx| {
            let mut out = vec![wheel];
            if let Some(alpha) = alpha {
                out.push(alpha);
            }
            out
        },
    );

    if let Some(test_id) = test_id {
        picker = picker.test_id(test_id);
    }
    picker
}
