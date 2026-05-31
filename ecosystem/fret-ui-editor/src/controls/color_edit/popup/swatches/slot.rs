use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, UiActionHost, UiActionHostAdapter};
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, Overflow, PressableA11y, PressableProps,
    SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::colors::{editor_border, editor_focus_ring};

use super::super::super::drag_drop::{
    ColorDragDropStore, install_color_drag_source, take_delivered_color_drop,
    update_color_drop_target,
};
use super::super::super::model::{color_from_rgb_preserving_alpha, format_hex};
use super::super::super::{
    ColorEditAlphaPreview, ColorEditDragDropOptions, ColorEditDragDropPayload,
    ColorEditPaletteEntry, ColorEditPaletteSlotDrop, OnColorEditPaletteSlotDrop,
};
use super::super::preview::color_preview_stack;

pub(super) fn preset_swatch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    index: usize,
    entry: ColorEditPaletteEntry,
    selected: bool,
    current_alpha: f32,
    model: Model<Color>,
    draft: Model<String>,
    error: Model<Option<Arc<str>>>,
    open: Model<bool>,
    show_alpha: bool,
    enabled: bool,
    alpha_preview: ColorEditAlphaPreview,
    drag_drop_store: Model<ColorDragDropStore>,
    drag_drop_options: ColorEditDragDropOptions,
    drag_threshold: Px,
    on_palette_slot_drop: Option<OnColorEditPaletteSlotDrop>,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let name = entry.name.clone();
    let rgb = entry.rgb;
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

    let theme = Theme::global(&*cx.app);
    let idle_border_color = editor_border(theme);
    let ring = editor_focus_ring(theme);
    let drag_drop_store_for_render = drag_drop_store.clone();
    let on_palette_slot_drop_for_render = on_palette_slot_drop.clone();

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
        move |cx, st| {
            cx.pressable_add_on_activate(on_activate.clone());
            let swatch_id = cx.root_id();
            let source_options = ColorEditDragDropOptions {
                enabled: enabled && drag_drop_options.enabled,
                ..drag_drop_options
            };
            install_color_drag_source(
                cx,
                swatch_id,
                drag_drop_store_for_render.clone(),
                ColorEditDragDropPayload::from_color(color, false),
                source_options,
                drag_threshold,
            );
            let drop_over = update_color_drop_target(
                cx,
                &drag_drop_store_for_render,
                swatch_id,
                st.hovered_raw,
                source_options.enabled && on_palette_slot_drop_for_render.is_some(),
            );
            let active = selected || drop_over;
            let border_width = if active { Px(2.0) } else { Px(1.0) };
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
                    border: Edges::all(border_width),
                    border_color: Some(if active { ring } else { idle_border_color }),
                    corner_radii: Corners::all(Px(5.0)),
                    padding: Edges::all(border_width).into(),
                    ..Default::default()
                },
                move |cx| vec![color_preview_stack(cx, color, Px(5.0), alpha_preview)],
            )]
        },
    );

    if let Some(test_id) = test_id {
        swatch = swatch.test_id(test_id);
    }
    if enabled
        && drag_drop_options.enabled
        && let Some(on_palette_slot_drop) = on_palette_slot_drop
        && let Some(payload) = take_delivered_color_drop(cx, &drag_drop_store, swatch.id)
    {
        let action_cx = ActionCx {
            window: cx.window,
            target: swatch.id,
        };
        let event = ColorEditPaletteSlotDrop::new(index, entry, payload);
        let mut host = UiActionHostAdapter { app: cx.app };
        on_palette_slot_drop(&mut host, action_cx, event);
        host.request_redraw(action_cx.window);
    }
    swatch.a11y_value(format_hex(color, show_alpha))
}
