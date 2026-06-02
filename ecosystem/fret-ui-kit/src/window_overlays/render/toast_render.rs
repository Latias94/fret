use core::time::Duration;
use std::collections::HashMap;
use std::sync::Arc;

use fret_core::{Color, FontId, FontWeight, Px, TextStyle};
use fret_ui::element::{AnyElement, PressableState};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::typography::{self, TextIntent};

use super::super::requests::{
    ToastButtonStyle, ToastIconButtonStyle, ToastIconOverride, ToastTextStyle,
};
use super::super::toast::{ToastEntry, ToastId};
use super::super::{ToastPosition, ToastVariant};

#[derive(Default)]
pub(super) struct ToastViewportPauseState {
    pub(super) paused: bool,
}

pub(super) fn toast_part_test_id(base: Option<&Arc<str>>, part: &str) -> Option<Arc<str>> {
    base.map(|base| Arc::<str>::from(format!("{}.{}", base.as_ref(), part)))
}

pub(super) fn toast_icon_from_override<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    fg: Color,
    _size: Px,
    icon: &ToastIconOverride,
) -> Option<AnyElement> {
    match icon {
        ToastIconOverride::Hidden => None,
        ToastIconOverride::Glyph(glyph) => Some(cx.text_props(fret_ui::element::TextProps {
            layout: fret_ui::element::LayoutStyle::default(),
            text: glyph.clone(),
            style: None,
            color: Some(fg),
            wrap: fret_core::TextWrap::None,
            overflow: fret_core::TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: fret_ui::element::TextInkOverflow::None,
        })),
        #[cfg(feature = "icons")]
        ToastIconOverride::IconId(icon) => Some(crate::declarative::icon::icon_with(
            cx,
            icon.clone(),
            Some(_size),
            Some(crate::ColorRef::Color(fg)),
        )),
    }
}

pub(super) fn alpha_mul(mut c: Color, mul: f32) -> Color {
    c.a = (c.a * mul).clamp(0.0, 1.0);
    c
}

pub(super) fn sonner_toast_title_style() -> TextStyle {
    typography::with_intent(
        TextStyle {
            font: FontId::default(),
            size: Px(13.0),
            weight: FontWeight(500),
            line_height: Some(Px(13.0 * 1.5)),
            ..Default::default()
        },
        TextIntent::Control,
    )
}

pub(super) fn sonner_toast_description_style() -> TextStyle {
    typography::with_intent(
        TextStyle {
            font: FontId::default(),
            size: Px(13.0),
            weight: FontWeight(400),
            line_height: Some(Px(13.0 * 1.4)),
            ..Default::default()
        },
        TextIntent::Control,
    )
}

pub(super) fn resolve_toast_color_key(
    theme: &fret_ui::Theme,
    key: Option<&str>,
    fallback: Color,
) -> Color {
    key.and_then(|key| theme.color_by_key(key))
        .unwrap_or(fallback)
}

pub(super) fn resolve_toast_text_style(
    theme: &fret_ui::Theme,
    style: &ToastTextStyle,
    fallback: TextStyle,
) -> TextStyle {
    style
        .style_key
        .as_deref()
        .and_then(|key| theme.text_style_by_key(key))
        .unwrap_or(fallback)
}

pub(super) fn resolve_toast_text_color(
    theme: &fret_ui::Theme,
    style: &ToastTextStyle,
    fallback: Color,
) -> Color {
    style
        .color
        .unwrap_or_else(|| resolve_toast_color_key(theme, style.color_key.as_deref(), fallback))
}

fn resolve_toast_number_key(theme: &fret_ui::Theme, key: Option<&str>, fallback: f32) -> f32 {
    key.and_then(|key| theme.number_by_key(key))
        .unwrap_or(fallback)
}

pub(super) fn toast_button_state_layer(
    theme: &fret_ui::Theme,
    style: &ToastButtonStyle,
    st: PressableState,
    fallback_color: Color,
) -> Option<Color> {
    let (key, fallback_opacity) = if st.pressed {
        (
            style.pressed_state_layer_opacity_key.as_deref(),
            style.pressed_state_layer_opacity,
        )
    } else if st.focused {
        (
            style.focus_state_layer_opacity_key.as_deref(),
            style.focus_state_layer_opacity,
        )
    } else if st.hovered {
        (
            style.hover_state_layer_opacity_key.as_deref(),
            style.hover_state_layer_opacity,
        )
    } else {
        return None;
    };

    let color = style.state_layer_color.unwrap_or_else(|| {
        resolve_toast_color_key(
            theme,
            style.state_layer_color_key.as_deref(),
            fallback_color,
        )
    });
    Some(alpha_mul(
        color,
        resolve_toast_number_key(theme, key, fallback_opacity),
    ))
}

pub(super) fn toast_icon_button_state_layer(
    theme: &fret_ui::Theme,
    style: &ToastIconButtonStyle,
    st: PressableState,
    fallback_color: Color,
) -> Option<Color> {
    let (key, fallback_opacity) = if st.pressed {
        (
            style.pressed_state_layer_opacity_key.as_deref(),
            style.pressed_state_layer_opacity,
        )
    } else if st.focused {
        (
            style.focus_state_layer_opacity_key.as_deref(),
            style.focus_state_layer_opacity,
        )
    } else if st.hovered {
        (
            style.hover_state_layer_opacity_key.as_deref(),
            style.hover_state_layer_opacity,
        )
    } else {
        return None;
    };

    let color = style.state_layer_color.unwrap_or_else(|| {
        resolve_toast_color_key(
            theme,
            style.state_layer_color_key.as_deref(),
            fallback_color,
        )
    });
    Some(alpha_mul(
        color,
        resolve_toast_number_key(theme, key, fallback_opacity),
    ))
}

pub(super) fn toast_description_text<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text: Arc<str>,
    style: &TextStyle,
    foreground: Color,
) -> AnyElement {
    typography::scope_text_style_with_color(
        cx.text_props(fret_ui::element::TextProps {
            layout: fret_ui::element::LayoutStyle::default(),
            text,
            style: None,
            color: None,
            wrap: fret_core::TextWrap::Word,
            overflow: fret_core::TextOverflow::Clip,
            align: fret_core::TextAlign::Start,
            ink_overflow: fret_ui::element::TextInkOverflow::None,
        }),
        typography::composable_refinement_from_style(style),
        foreground,
    )
}

fn toast_position_key(position: ToastPosition) -> u8 {
    match position {
        ToastPosition::TopLeft => 0,
        ToastPosition::TopCenter => 1,
        ToastPosition::TopRight => 2,
        ToastPosition::BottomLeft => 3,
        ToastPosition::BottomCenter => 4,
        ToastPosition::BottomRight => 5,
    }
}

#[derive(Default)]
struct ToastStackShiftState {
    generation: u64,
    active: bool,
    last_targets_y: HashMap<ToastId, Px>,
    last_targets_scale: HashMap<ToastId, f32>,
    last_visual_y: HashMap<ToastId, Px>,
    last_visual_scale: HashMap<ToastId, f32>,
    deltas_y: HashMap<ToastId, Px>,
    deltas_scale: HashMap<ToastId, f32>,
}

#[derive(Clone)]
struct ToastStackShiftSnapshot {
    generation: u64,
    active: bool,
    deltas_y: HashMap<ToastId, Px>,
    deltas_scale: HashMap<ToastId, f32>,
}

pub(super) struct ToastStackShiftOutput {
    pub(super) stack_offset_y: Vec<Px>,
    pub(super) stack_scale: Vec<f32>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn toast_stack_shift_output<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    toaster_key: GlobalElementId,
    stack_position: ToastPosition,
    stack_toasts: &[ToastEntry],
    target_stack_offset_y: &[Px],
    target_stack_scale: &[f32],
    expanded: bool,
    duration: Duration,
    each_delay: Duration,
    bezier: fret_ui::theme::CubicBezier,
) -> ToastStackShiftOutput {
    debug_assert_eq!(stack_toasts.len(), target_stack_offset_y.len());
    debug_assert_eq!(stack_toasts.len(), target_stack_scale.len());

    let count = stack_toasts.len();
    if count == 0 {
        return ToastStackShiftOutput {
            stack_offset_y: Vec::new(),
            stack_scale: Vec::new(),
        };
    }

    let stack_key = (
        "toast_stack_shift",
        toaster_key,
        toast_position_key(stack_position),
    );
    cx.keyed(stack_key, |cx| {
        let stack_shift_state_slot = cx.slot_id();
        let mut current_targets_y: HashMap<ToastId, Px> = HashMap::with_capacity(count);
        let mut current_targets_scale: HashMap<ToastId, f32> = HashMap::with_capacity(count);
        for (idx, toast) in stack_toasts.iter().enumerate() {
            current_targets_y.insert(toast.id, target_stack_offset_y[idx]);
            current_targets_scale.insert(toast.id, target_stack_scale[idx]);
        }

        let snapshot: ToastStackShiftSnapshot = cx.state_for(
            stack_shift_state_slot,
            ToastStackShiftState::default,
            |st| {
                if expanded {
                    st.active = false;
                    st.deltas_y.clear();
                    st.deltas_scale.clear();
                    st.last_targets_y.clone_from(&current_targets_y);
                    st.last_targets_scale.clone_from(&current_targets_scale);
                    st.last_visual_y.clone_from(&current_targets_y);
                    st.last_visual_scale.clone_from(&current_targets_scale);
                    return ToastStackShiftSnapshot {
                        generation: st.generation,
                        active: false,
                        deltas_y: HashMap::new(),
                        deltas_scale: HashMap::new(),
                    };
                }

                let mut changed = st.last_targets_y.len() != current_targets_y.len();
                if !changed {
                    for (id, curr) in &current_targets_y {
                        if let Some(prev) = st.last_targets_y.get(id)
                            && (prev.0 - curr.0).abs() > 0.5
                        {
                            changed = true;
                            break;
                        }
                    }
                }

                if changed {
                    st.active = true;
                    st.generation = st.generation.wrapping_add(1);
                    st.deltas_y.clear();
                    st.deltas_scale.clear();

                    for (id, curr_y) in &current_targets_y {
                        let from_y = st
                            .last_visual_y
                            .get(id)
                            .copied()
                            .or_else(|| st.last_targets_y.get(id).copied())
                            .unwrap_or(*curr_y);
                        st.deltas_y.insert(*id, Px(from_y.0 - curr_y.0));

                        let target_scale = current_targets_scale.get(id).copied().unwrap_or(1.0);
                        let from_scale = st
                            .last_visual_scale
                            .get(id)
                            .copied()
                            .or_else(|| st.last_targets_scale.get(id).copied())
                            .unwrap_or(target_scale);
                        st.deltas_scale.insert(*id, from_scale - target_scale);
                    }
                }

                st.last_targets_y.clone_from(&current_targets_y);
                st.last_targets_scale.clone_from(&current_targets_scale);

                ToastStackShiftSnapshot {
                    generation: st.generation,
                    active: st.active,
                    deltas_y: st.deltas_y.clone(),
                    deltas_scale: st.deltas_scale.clone(),
                }
            },
        );

        let shift = cx.keyed(snapshot.generation, |cx| {
            crate::declarative::transition::drive_transition_with_durations_and_cubic_bezier_duration(
                cx,
                snapshot.active,
                duration,
                duration,
                bezier,
            )
        });

        let bezier_headless = crate::headless::easing::CubicBezier::new(
            bezier.x1, bezier.y1, bezier.x2, bezier.y2,
        );

        let mut out_y: Vec<Px> = Vec::with_capacity(count);
        let mut out_scale: Vec<f32> = Vec::with_capacity(count);

        for idx in 0..count {
            let toast_id = stack_toasts[idx].id;
            let target_y = target_stack_offset_y[idx];
            let target_scale = target_stack_scale[idx];

            if !snapshot.active {
                out_y.push(target_y);
                out_scale.push(target_scale);
                continue;
            }

            let delta_y = snapshot.deltas_y.get(&toast_id).copied().unwrap_or(Px(0.0));
            let delta_scale = snapshot
                .deltas_scale
                .get(&toast_id)
                .copied()
                .unwrap_or(0.0);

            let local_linear = crate::headless::stagger::staggered_progress_for_duration(
                shift.linear,
                idx,
                count,
                each_delay,
                duration,
                crate::headless::stagger::StaggerFrom::First,
            );
            let local = bezier_headless.sample(local_linear);

            out_y.push(Px(target_y.0 + delta_y.0 * (1.0 - local)));
            out_scale.push((target_scale + delta_scale * (1.0 - local)).clamp(0.0, 2.0));
        }

        cx.state_for(stack_shift_state_slot, ToastStackShiftState::default, |st| {
            st.last_visual_y.clear();
            st.last_visual_scale.clear();
            for idx in 0..count {
                let id = stack_toasts[idx].id;
                st.last_visual_y.insert(id, out_y[idx]);
                st.last_visual_scale.insert(id, out_scale[idx]);
            }

            if snapshot.active
                && !shift.animating
                && (shift.progress - 1.0).abs() <= f32::EPSILON
            {
                st.active = false;
                st.deltas_y.clear();
                st.deltas_scale.clear();
                st.last_visual_y.clone_from(&current_targets_y);
                st.last_visual_scale.clone_from(&current_targets_scale);
            }
        });

        ToastStackShiftOutput {
            stack_offset_y: out_y,
            stack_scale: out_scale,
        }
    })
}

pub(super) fn toast_icon_glyph(variant: ToastVariant) -> Option<&'static str> {
    match variant {
        ToastVariant::Success => Some("\u{2713}"),
        ToastVariant::Info => Some("i"),
        ToastVariant::Warning => Some("!"),
        ToastVariant::Error | ToastVariant::Destructive => Some("\u{00D7}"),
        ToastVariant::Loading => None,
        ToastVariant::Default => None,
    }
}
