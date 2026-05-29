use std::sync::Arc;

use fret_core::{Axis, Color, Edges, Px};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiPointerActionHost};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexItemStyle, FlexProps, LayoutStyle, Length,
    MainAlign, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::colors::{editor_border, editor_focus_ring};
use crate::primitives::input_group::derived_test_id;

use super::super::model::{HsvColor, format_hex, hsv_to_color_preserving_alpha};

pub(in crate::controls::color_edit) mod alpha;
mod hue_bar;
mod hue_wheel;
mod hue_wheel_picker;
mod sv;

pub(super) use alpha::alpha_bar;
use alpha::vertical_alpha_bar;
use hue_bar::hue_bar;
pub(super) use hue_bar::hue_bar_preview_stack;
pub(super) use hue_wheel::hue_wheel_canvas;
use hue_wheel_picker::hue_wheel_picker;
use sv::sv_picker;
pub(super) use sv::sv_picker_preview_stack;

const HSV_PICKER_SIZE: Px = Px(120.0);
const HUE_WHEEL_PICKER_WIDTH: Px = Px(138.0);

fn picker_border_and_ring(theme: &Theme) -> (Color, Color) {
    (editor_border(theme), editor_focus_ring(theme))
}

pub(super) fn hsv_picker<H: UiHost>(
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

pub(super) fn hsv_hue_wheel_picker<H: UiHost>(
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

fn horizontal_bar_thumb_spacer<H: UiHost>(cx: &mut ElementContext<'_, H>, grow: f32) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Auto,
                    height: Length::Fill,
                    ..Default::default()
                },
                flex: FlexItemStyle {
                    grow,
                    shrink: 1.0,
                    basis: Length::Px(Px(0.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        |_cx| vec![],
    )
}

fn apply_hsv_color(
    host: &mut dyn UiPointerActionHost,
    action_cx: ActionCx,
    model: &Model<Color>,
    draft: &Model<String>,
    error: &Model<Option<Arc<str>>>,
    show_alpha: bool,
    current: Color,
    next_hsv: HsvColor,
) {
    let next = hsv_to_color_preserving_alpha(next_hsv, current.a);
    let formatted = format_hex(next, show_alpha);

    let _ = host.models_mut().update(model, |c| *c = next);
    let _ = host
        .models_mut()
        .update(draft, |s| *s = formatted.as_ref().to_string());
    let _ = host.models_mut().update(error, |e| *e = None);
    host.request_redraw(action_cx.window);
}
