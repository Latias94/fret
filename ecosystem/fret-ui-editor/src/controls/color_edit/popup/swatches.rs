use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign, Overflow,
    PressableA11y, PressableProps, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::input_group::derived_test_id;

use super::super::model::{color_from_rgb_preserving_alpha, format_hex};
use super::super::{ColorEditAlphaPreview, ColorEditPaletteEntry};
use super::preview::color_preview_stack;

pub(super) fn preset_swatches<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    current: Color,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    open: Model<bool>,
    show_alpha: bool,
    enabled: bool,
    alpha_preview: ColorEditAlphaPreview,
    palette: Arc<[ColorEditPaletteEntry]>,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let current_rgb = fret_ui_kit::colors::hex_rgb_from_linear(current);
    cx.flex(
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
            align: CrossAlign::Center,
            wrap: true,
        },
        move |cx| {
            palette
                .iter()
                .enumerate()
                .map(|(idx, entry)| {
                    preset_swatch(
                        cx,
                        entry.name.clone(),
                        entry.rgb,
                        current_rgb == entry.rgb,
                        current.a,
                        model.clone(),
                        draft.clone(),
                        error.clone(),
                        open.clone(),
                        show_alpha,
                        enabled,
                        alpha_preview,
                        derived_test_id(test_id.as_ref(), format!("preset.{idx}").as_str()),
                    )
                })
                .collect::<Vec<_>>()
        },
    )
}

fn preset_swatch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    name: Arc<str>,
    rgb: u32,
    selected: bool,
    current_alpha: f32,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    open: Model<bool>,
    show_alpha: bool,
    enabled: bool,
    alpha_preview: ColorEditAlphaPreview,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let color = color_from_rgb_preserving_alpha(rgb, current_alpha);
    let on_activate: OnActivate =
        Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
            let current = host.models_mut().get_copied(&model).unwrap_or(color);
            let color = color_from_rgb_preserving_alpha(rgb, current.a);
            let formatted = format_hex(color, show_alpha);
            let _ = host.models_mut().update(&model, |c| *c = color);
            let _ = host
                .models_mut()
                .update(&draft, |s| *s = formatted.as_ref().to_string());
            let _ = host.models_mut().update(&error, |e| *e = None);
            let _ = host.models_mut().update(&open, |v| *v = false);
            host.request_redraw(action_cx.window);
        });

    let (border_color, ring) = {
        let theme = Theme::global(&*cx.app);
        let ring = theme
            .color_by_key("ring")
            .unwrap_or_else(|| theme.color_token("primary"));
        let border_color = if selected {
            ring
        } else {
            theme
                .color_by_key("border")
                .unwrap_or_else(|| theme.color_token("border"))
        };
        (border_color, ring)
    };

    let mut swatch = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(Px(28.0)),
                    height: Length::Px(Px(28.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(fret_core::SemanticsRole::Button),
                label: Some(Arc::from(format!("{} color preset", name.as_ref()))),
                ..Default::default()
            },
            focus_ring: Some(fret_ui::element::RingStyle {
                placement: fret_ui::element::RingPlacement::Outset,
                width: Px(2.0),
                offset: Px(1.0),
                color: ring,
                offset_color: None,
                corner_radii: Corners::all(Px(5.0)),
            }),
            ..Default::default()
        },
        move |cx, _st| {
            cx.pressable_add_on_activate(on_activate.clone());
            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Fill,
                            height: Length::Fill,
                            ..Default::default()
                        },
                        overflow: Overflow::Clip,
                        ..Default::default()
                    },
                    border: Edges::all(if selected { Px(2.0) } else { Px(1.0) }),
                    border_color: Some(border_color),
                    corner_radii: Corners::all(Px(5.0)),
                    padding: Edges::all(if selected { Px(2.0) } else { Px(1.0) }).into(),
                    ..Default::default()
                },
                move |cx| vec![color_preview_stack(cx, color, Px(5.0), alpha_preview)],
            )]
        },
    );

    if let Some(test_id) = test_id {
        swatch = swatch.test_id(test_id);
    }
    swatch.a11y_value(format_hex(color, show_alpha))
}
