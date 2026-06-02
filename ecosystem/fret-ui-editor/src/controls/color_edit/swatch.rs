mod context_menu;

use std::sync::Arc;

use fret_core::{Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate};
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, Overflow, PressableA11y, PressableProps,
    SizeStyle,
};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};

use crate::primitives::style::EditorStyle;
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState, EditorWidgetVisuals};
use crate::primitives::{EditorDensity, EditorTokenKeys};

use self::context_menu::{
    install_context_menu_keyboard_handler, install_context_menu_pointer_handler,
};
use super::drag_drop::{ColorDragDropStore, install_color_drag_source, update_color_drop_target};
use super::popup::color_preview_stack;
use super::{
    ColorEditAlphaPreview, ColorEditCopyOptions, ColorEditDragDropOptions,
    ColorEditDragDropPayload, ColorEditPopupOptions, ColorEditTooltipOptions,
};

pub(super) struct ColorEditSwatchArgs {
    pub(super) model: Model<Color>,
    pub(super) open: Model<bool>,
    pub(super) tooltip_open: Model<bool>,
    pub(super) copy_menu_open: Model<bool>,
    pub(super) reference: Model<Option<Color>>,
    pub(super) drag_drop_store: Model<ColorDragDropStore>,
    pub(super) current: Color,
    pub(super) current_hex: Arc<str>,
    pub(super) show_alpha: bool,
    pub(super) alpha_preview: ColorEditAlphaPreview,
    pub(super) enabled: bool,
    pub(super) swatch_enabled: bool,
    pub(super) swatch_focusable: bool,
    pub(super) popup_has_visible_content: bool,
    pub(super) popup_options: ColorEditPopupOptions,
    pub(super) tooltip_options: ColorEditTooltipOptions,
    pub(super) copy_options: ColorEditCopyOptions,
    pub(super) copy_enabled: bool,
    pub(super) drag_drop_enabled: bool,
    pub(super) drag_drop_options: ColorEditDragDropOptions,
    pub(super) drag_threshold: Px,
    pub(super) test_id: Option<Arc<str>>,
}

pub(super) fn color_swatch<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: ColorEditSwatchArgs,
) -> AnyElement {
    let ColorEditSwatchArgs {
        model,
        open,
        tooltip_open,
        copy_menu_open,
        reference,
        drag_drop_store,
        current,
        current_hex,
        show_alpha,
        alpha_preview,
        enabled,
        swatch_enabled,
        swatch_focusable,
        popup_has_visible_content,
        popup_options,
        tooltip_options,
        copy_options,
        copy_enabled,
        drag_drop_enabled,
        drag_drop_options,
        drag_threshold,
        test_id,
    } = args;

    let (frame_chrome, hit_thickness, swatch_size, ring) = {
        let theme = Theme::global(&*cx.app);
        let density = EditorDensity::resolve(theme);
        let frame_chrome = EditorStyle::resolve(theme).frame_chrome_small();
        let swatch_size = theme
            .metric_by_key(EditorTokenKeys::COLOR_SWATCH_SIZE)
            .unwrap_or(density.icon_size);
        let ring = theme
            .color_by_key("ring")
            .unwrap_or_else(|| theme.color_token("primary"));
        (frame_chrome, density.hit_thickness, swatch_size, ring)
    };

    let open_for_activate = open.clone();
    let open_for_paint = open.clone();
    let tooltip_open_for_paint = tooltip_open.clone();
    let copy_menu_open_for_activate = copy_menu_open.clone();
    let copy_menu_open_for_pointer = copy_menu_open.clone();
    let copy_menu_open_for_paint = copy_menu_open.clone();
    let open_for_pointer = open.clone();
    let tooltip_open_for_pointer = tooltip_open.clone();
    let reference_for_activate = reference.clone();
    let model_for_activate = model.clone();
    let drag_drop_store_for_swatch = drag_drop_store.clone();
    let on_activate: OnActivate =
        Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
            if !popup_has_visible_content {
                return;
            }
            let prev = host
                .models_mut()
                .get_copied(&open_for_activate)
                .unwrap_or(false);
            let opening = !prev;
            if opening && popup_options.side_preview.shows_original() {
                let current = host
                    .models_mut()
                    .get_copied(&model_for_activate)
                    .unwrap_or(Color::TRANSPARENT);
                let _ = host
                    .models_mut()
                    .update(&reference_for_activate, |reference| {
                        *reference = Some(current)
                    });
            }
            let _ = host
                .models_mut()
                .update(&open_for_activate, |v| *v = opening);
            let _ = host
                .models_mut()
                .update(&copy_menu_open_for_activate, |v| *v = false);
            host.request_redraw(action_cx.window);
        });

    let mut swatch = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(hit_thickness),
                    height: Length::Px(hit_thickness),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: swatch_enabled,
            focusable: swatch_enabled && swatch_focusable,
            a11y: PressableA11y {
                role: Some(SemanticsRole::Button),
                label: Some(Arc::from("Color swatch")),
                ..Default::default()
            },
            focus_ring: Some(fret_ui::element::RingStyle {
                placement: fret_ui::element::RingPlacement::Outset,
                width: Px(2.0),
                offset: Px(2.0),
                color: ring,
                offset_color: None,
                corner_radii: Corners::all(frame_chrome.radius),
            }),
            ..Default::default()
        },
        move |cx, st| {
            cx.pressable_add_on_activate(on_activate.clone());
            install_context_menu_pointer_handler(
                cx,
                copy_options.enabled,
                open_for_pointer.clone(),
                tooltip_open_for_pointer.clone(),
                copy_menu_open_for_pointer.clone(),
            );
            let swatch_id = cx.root_id();
            install_color_drag_source(
                cx,
                swatch_id,
                drag_drop_store_for_swatch.clone(),
                ColorEditDragDropPayload::from_color(current, show_alpha),
                drag_drop_options,
                drag_threshold,
            );
            let drop_over = update_color_drop_target(
                cx,
                &drag_drop_store_for_swatch,
                swatch_id,
                st.hovered_raw,
                drag_drop_enabled,
            );

            let is_open = cx
                .get_model_copied(&open_for_paint, Invalidation::Paint)
                .unwrap_or(false);
            let copy_menu_is_open = cx
                .get_model_copied(&copy_menu_open_for_paint, Invalidation::Paint)
                .unwrap_or(false);
            let tooltip_visible = tooltip_options.enabled
                && enabled
                && !is_open
                && !copy_menu_is_open
                && st.hovered_raw;
            let tooltip_open_now = cx
                .get_model_copied(&tooltip_open_for_paint, Invalidation::Paint)
                .unwrap_or(false);
            if tooltip_open_now != tooltip_visible {
                let _ = cx
                    .app
                    .models_mut()
                    .update(&tooltip_open_for_paint, |value| *value = tooltip_visible);
            }
            let visuals = {
                let theme = Theme::global(&*cx.app);
                EditorWidgetVisuals::new(theme).frame_visuals(
                    frame_chrome,
                    EditorFrameState {
                        enabled,
                        hovered: st.hovered || st.hovered_raw,
                        pressed: st.pressed || drop_over,
                        focused: st.focused,
                        open: (is_open && popup_has_visible_content) || copy_menu_is_open,
                        semantic: EditorFrameSemanticState::default(),
                    },
                )
            };

            vec![cx.container(
                ContainerProps {
                    layout: LayoutStyle {
                        size: SizeStyle {
                            width: Length::Px(swatch_size),
                            height: Length::Px(swatch_size),
                            ..Default::default()
                        },
                        overflow: Overflow::Clip,
                        ..Default::default()
                    },
                    border: Edges::all(frame_chrome.border_width),
                    border_color: Some(visuals.border),
                    corner_radii: Corners::all(frame_chrome.radius),
                    padding: Edges::all(frame_chrome.border_width).into(),
                    ..Default::default()
                },
                move |cx| {
                    vec![color_preview_stack(
                        cx,
                        current,
                        frame_chrome.radius,
                        alpha_preview,
                    )]
                },
            )]
        },
    );

    if let Some(test_id) = test_id.as_ref() {
        swatch = swatch.test_id(test_id.clone());
    }
    swatch = swatch.a11y_value(current_hex.clone());
    install_context_menu_keyboard_handler(
        cx,
        swatch.id,
        copy_enabled,
        open.clone(),
        tooltip_open.clone(),
        copy_menu_open.clone(),
    );
    swatch
}
