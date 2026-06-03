mod activation;
mod context_menu;
mod visual;

use std::sync::Arc;

use fret_core::{Color, Corners, Px, SemanticsRole};
use fret_runtime::Model;
use fret_ui::element::{AnyElement, LayoutStyle, Length, PressableA11y, PressableProps, SizeStyle};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::style::EditorStyle;
use crate::primitives::{EditorDensity, EditorTokenKeys};

use self::activation::{ColorSwatchActivateInput, color_swatch_activate};
use self::context_menu::{
    install_context_menu_keyboard_handler, install_context_menu_pointer_handler,
};
use self::visual::{ColorSwatchVisualArgs, color_swatch_visual};
use super::drag_drop::{ColorDragDropStore, install_color_drag_source, update_color_drop_target};
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

    let open_for_paint = open.clone();
    let tooltip_open_for_paint = tooltip_open.clone();
    let copy_menu_open_for_pointer = copy_menu_open.clone();
    let copy_menu_open_for_paint = copy_menu_open.clone();
    let open_for_pointer = open.clone();
    let tooltip_open_for_pointer = tooltip_open.clone();
    let drag_drop_store_for_swatch = drag_drop_store.clone();
    let on_activate = color_swatch_activate(ColorSwatchActivateInput {
        model: model.clone(),
        open: open.clone(),
        copy_menu_open: copy_menu_open.clone(),
        reference: reference.clone(),
        popup_has_visible_content,
        popup_options,
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

            vec![color_swatch_visual(
                cx,
                ColorSwatchVisualArgs {
                    open: open_for_paint.clone(),
                    tooltip_open: tooltip_open_for_paint.clone(),
                    copy_menu_open: copy_menu_open_for_paint.clone(),
                    current,
                    alpha_preview,
                    frame_chrome,
                    swatch_size,
                    enabled,
                    popup_has_visible_content,
                    tooltip_options,
                    hovered: st.hovered,
                    hovered_raw: st.hovered_raw,
                    pressed: st.pressed,
                    focused: st.focused,
                    drop_over,
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
