use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px, SemanticsRole};
use fret_runtime::{Effect, Model};
use fret_ui::action::{ActionCx, ActivateReason, OnActivate, OnCloseAutoFocus};
use fret_ui::element::{
    AnchoredProps, AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length,
    MainAlign, PointerRegionProps, PressableA11y, PressableProps, SemanticsDecoration, SizeStyle,
    SpacingLength,
};
use fret_ui::overlay_placement::{Align, Side};
use fret_ui::{ElementContext, Invalidation, Theme, UiHost};
use fret_ui_kit::primitives::popper;
use fret_ui_kit::{OverlayController, OverlayPresence, OverlayRequest};

use crate::primitives::EditorDensity;
use crate::primitives::input_group::derived_test_id;
use crate::primitives::popup_list::{
    EditorPopupListRowState, editor_popup_list_row_gap, editor_popup_list_row_palette,
    editor_popup_list_row_radius, editor_popup_list_row_text_props,
};
use crate::primitives::popup_surface::resolve_editor_popup_surface_chrome;

use super::super::ColorEditCopyOptions;
use super::super::model::format_hex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::controls::color_edit) enum ColorEditCopyFormat {
    FloatTuple,
    IntTuple,
    HexRgb,
    HexRgba,
}

impl ColorEditCopyFormat {
    fn test_suffix(self) -> &'static str {
        match self {
            Self::FloatTuple => "float-tuple",
            Self::IntTuple => "int-tuple",
            Self::HexRgb => "hex-rgb",
            Self::HexRgba => "hex-rgba",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::controls::color_edit) struct ColorEditCopyEntry {
    pub(in crate::controls::color_edit) format: ColorEditCopyFormat,
    pub(in crate::controls::color_edit) text: Arc<str>,
}

pub(in crate::controls::color_edit) fn request_color_copy_menu_overlay<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    swatch_id: fret_ui::elements::GlobalElementId,
    open: Model<bool>,
    current: Color,
    show_alpha: bool,
    copy_options: ColorEditCopyOptions,
    test_id: Option<Arc<str>>,
) {
    if !copy_options.enabled {
        return;
    }

    let overlay_id = cx
        .named("color_edit.copy_menu", |cx| cx.spacer(Default::default()))
        .id;
    let is_open = cx
        .get_model_copied(&open, Invalidation::Layout)
        .unwrap_or(false);
    let presence = OverlayPresence::instant(is_open);

    let close_focus: OnCloseAutoFocus = Arc::new(move |host, _cx, req| {
        req.prevent_default();
        host.request_focus(swatch_id);
    });

    let placement = popper::PopperContentPlacement::new(
        popper::LayoutDirection::Ltr,
        Side::Right,
        Align::Start,
        Px(4.0),
    )
    .with_collision_padding(Edges::all(Px(8.0)))
    .with_shift_cross_axis(true);

    let open_for_content = open.clone();
    let copy_menu = cx.anchored_props(
        AnchoredProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Fill,
                    ..Default::default()
                },
                ..Default::default()
            },
            anchor_element: Some(swatch_id.0),
            side: placement.side,
            align: placement.align,
            side_offset: placement.side_offset,
            options: placement.options(),
            ..Default::default()
        },
        move |cx| {
            vec![color_copy_menu_panel(
                cx,
                open_for_content.clone(),
                current,
                show_alpha,
                test_id.clone(),
            )]
        },
    );

    let mut request = OverlayRequest::dismissible_menu(
        overlay_id,
        swatch_id,
        open,
        presence,
        vec![cx.pointer_region(
            PointerRegionProps {
                layout: LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                enabled: true,
                capture_phase_pointer_moves: false,
            },
            move |_cx| vec![copy_menu],
        )],
    );
    request.close_on_window_focus_lost = true;
    request.close_on_window_resize = true;
    request.on_close_auto_focus = Some(close_focus);

    OverlayController::request(cx, request);
}

pub(in crate::controls::color_edit) fn color_copy_entries(
    color: Color,
    show_alpha: bool,
) -> Vec<ColorEditCopyEntry> {
    let (r, g, b, a) = color_copy_u8_channels(color, show_alpha);
    let mut entries = vec![
        ColorEditCopyEntry {
            format: ColorEditCopyFormat::FloatTuple,
            text: Arc::from(format!(
                "({:.3}f, {:.3}f, {:.3}f, {:.3}f)",
                finite_or_zero(color.r),
                finite_or_zero(color.g),
                finite_or_zero(color.b),
                if show_alpha {
                    finite_or_zero(color.a)
                } else {
                    1.0
                }
            )),
        },
        ColorEditCopyEntry {
            format: ColorEditCopyFormat::IntTuple,
            text: Arc::from(format!("({r},{g},{b},{a})")),
        },
        ColorEditCopyEntry {
            format: ColorEditCopyFormat::HexRgb,
            text: format_hex(color, false),
        },
    ];

    if show_alpha {
        entries.push(ColorEditCopyEntry {
            format: ColorEditCopyFormat::HexRgba,
            text: format_hex(color, true),
        });
    }

    entries
}

fn color_copy_u8_channels(color: Color, show_alpha: bool) -> (u8, u8, u8, u8) {
    let rgb = fret_ui_kit::colors::hex_rgb_from_linear(color);
    let r = ((rgb >> 16) & 0xff) as u8;
    let g = ((rgb >> 8) & 0xff) as u8;
    let b = (rgb & 0xff) as u8;
    let a = if show_alpha {
        (color.a.clamp(0.0, 1.0) * 255.0).round() as u8
    } else {
        255
    };
    (r, g, b, a)
}

fn finite_or_zero(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

fn color_copy_menu_panel<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: Model<bool>,
    color: Color,
    show_alpha: bool,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let density = {
        let theme = Theme::global(&*cx.app);
        EditorDensity::resolve(theme)
    };
    let popup_chrome = {
        let theme = Theme::global(&*cx.app);
        resolve_editor_popup_surface_chrome(theme, true)
    };
    let entries = Arc::<[ColorEditCopyEntry]>::from(color_copy_entries(color, show_alpha));
    let row_test_id_prefix = test_id.clone();

    let mut panel = cx.container(
        ContainerProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Px(Px(196.0)),
                    height: Length::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            padding: Edges::all(Px(4.0)).into(),
            background: Some(popup_chrome.bg),
            border: Edges::all(Px(1.0)),
            border_color: Some(popup_chrome.border),
            corner_radii: Corners::all(popup_chrome.radius),
            shadow: popup_chrome.shadow,
            ..Default::default()
        },
        {
            let entries = entries.clone();
            let row_test_id_prefix = row_test_id_prefix.clone();
            move |cx| {
                vec![cx.flex(
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
                        gap: SpacingLength::Px(editor_popup_list_row_gap()),
                        padding: Edges::all(Px(0.0)).into(),
                        justify: MainAlign::Start,
                        align: CrossAlign::Stretch,
                        wrap: false,
                    },
                    {
                        let entries = entries.clone();
                        let row_test_id_prefix = row_test_id_prefix.clone();
                        move |cx| {
                            entries
                                .iter()
                                .map(|entry| {
                                    color_copy_menu_row(
                                        cx,
                                        entry.clone(),
                                        open.clone(),
                                        density.row_height,
                                        derived_test_id(
                                            row_test_id_prefix.as_ref(),
                                            entry.format.test_suffix(),
                                        ),
                                    )
                                })
                                .collect::<Vec<_>>()
                        }
                    },
                )]
            }
        },
    );

    let mut semantics = SemanticsDecoration::default().role(SemanticsRole::Menu);
    if let Some(test_id) = test_id {
        semantics = semantics.test_id(test_id);
    }
    panel = panel.attach_semantics(semantics);
    panel
}

fn color_copy_menu_row<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    entry: ColorEditCopyEntry,
    open: Model<bool>,
    row_height: Px,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let copy_text = entry.text.clone();
    let label = entry.text.clone();
    let a11y_label = Arc::<str>::from(format!("Copy color as {}", entry.text.as_ref()));
    let on_activate: OnActivate =
        Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
            let token = host.next_clipboard_token();
            host.push_effect(Effect::ClipboardWriteText {
                window: action_cx.window,
                token,
                text: copy_text.to_string(),
            });
            let _ = host.models_mut().update(&open, |value| *value = false);
            host.request_redraw(action_cx.window);
        });

    let mut row = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(row_height),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled: true,
            focusable: true,
            a11y: PressableA11y {
                role: Some(SemanticsRole::MenuItem),
                label: Some(a11y_label),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            cx.pressable_add_on_activate(on_activate.clone());
            let (bg, fg) = {
                let theme = Theme::global(&*cx.app);
                let palette = editor_popup_list_row_palette(
                    theme,
                    st.hovered || st.hovered_raw || st.focused,
                    EditorPopupListRowState::default(),
                );
                (palette.bg, palette.fg)
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
                    padding: Edges::symmetric(Px(8.0), Px(0.0)).into(),
                    background: bg,
                    border: Edges::all(Px(0.0)),
                    corner_radii: Corners::all(editor_popup_list_row_radius()),
                    ..Default::default()
                },
                {
                    let label = label.clone();
                    move |cx| {
                        vec![cx.text_props(editor_popup_list_row_text_props(
                            label.clone(),
                            fg,
                            row_height,
                        ))]
                    }
                },
            )]
        },
    );

    if let Some(test_id) = test_id {
        row = row.test_id(test_id);
    }
    row
}
