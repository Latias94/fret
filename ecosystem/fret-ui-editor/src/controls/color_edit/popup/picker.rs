use std::sync::Arc;

use fret_core::{Color, Px};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, UiPointerActionHost};
use fret_ui::element::{AnyElement, ContainerProps, FlexItemStyle, LayoutStyle, Length, SizeStyle};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::colors::{editor_border, editor_focus_ring};

use super::super::model::{HsvColor, format_hex, hsv_to_color_preserving_alpha};

pub(in crate::controls::color_edit) mod alpha;
mod hue_bar;
mod hue_wheel;
mod hue_wheel_picker;
mod layout;
mod sv;

pub(super) use alpha::alpha_bar;
pub(super) use hue_bar::hue_bar_preview_stack;
pub(super) use hue_wheel::hue_wheel_canvas;
pub(super) use layout::{hsv_hue_wheel_picker, hsv_picker};
pub(super) use sv::sv_picker_preview_stack;

const HSV_PICKER_SIZE: Px = Px(120.0);
const HUE_WHEEL_PICKER_WIDTH: Px = Px(138.0);

fn picker_border_and_ring(theme: &Theme) -> (Color, Color) {
    (editor_border(theme), editor_focus_ring(theme))
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
