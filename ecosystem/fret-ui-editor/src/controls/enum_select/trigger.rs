use std::sync::Arc;

use fret_core::{Axis, Color, Corners, Edges, Px};
use fret_runtime::Model;
use fret_ui::action::{ActionCx, ActivateReason, OnActivate};
use fret_ui::element::{
    AnyElement, ContainerProps, CrossAlign, FlexProps, LayoutStyle, Length, MainAlign,
    PressableA11y, PressableProps, RingPlacement, RingStyle, SizeStyle, SpacingLength,
};
use fret_ui::{ElementContext, UiHost};
use fret_ui_kit::primitives::combobox as kit_combobox;

use crate::primitives::EditorDensity;
use crate::primitives::chrome::ResolvedEditorFrameChrome;
use crate::primitives::icons::editor_icon_with;
use crate::primitives::input_group::{
    editor_input_group_divider, editor_input_group_frame, editor_input_group_inset,
    editor_input_group_row, editor_input_value_text,
};
use crate::primitives::visuals::{EditorFrameSemanticState, EditorFrameState};

pub(super) struct EnumSelectTriggerArgs {
    pub(super) layout: LayoutStyle,
    pub(super) enabled: bool,
    pub(super) focusable: bool,
    pub(super) a11y_label: Option<Arc<str>>,
    pub(super) density: EditorDensity,
    pub(super) frame_chrome: ResolvedEditorFrameChrome,
    pub(super) ring: Color,
    pub(super) is_open: bool,
    pub(super) trigger_text: Arc<str>,
    pub(super) open: Model<bool>,
    pub(super) open_change_reason: Model<Option<kit_combobox::ComboboxOpenChangeReason>>,
}

pub(super) fn enum_select_trigger<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    args: EnumSelectTriggerArgs,
) -> AnyElement {
    let EnumSelectTriggerArgs {
        mut layout,
        enabled,
        focusable,
        a11y_label,
        density,
        frame_chrome,
        ring,
        is_open,
        trigger_text,
        open,
        open_change_reason,
    } = args;

    if layout.size.min_height.is_none() {
        layout.size.min_height = Some(Length::Px(density.row_height));
    }

    cx.pressable(
        PressableProps {
            layout,
            enabled,
            focusable,
            a11y: PressableA11y {
                role: Some(fret_core::SemanticsRole::ComboBox),
                label: a11y_label,
                expanded: Some(is_open),
                ..Default::default()
            },
            focus_ring: Some(RingStyle {
                placement: RingPlacement::Outset,
                width: Px(2.0),
                offset: Px(2.0),
                color: ring,
                offset_color: None,
                corner_radii: Corners::all(frame_chrome.radius),
            }),
            ..Default::default()
        },
        move |cx, state| {
            cx.pressable_add_on_activate(kit_combobox::set_open_change_reason_on_activate(
                open_change_reason.clone(),
                kit_combobox::ComboboxOpenChangeReason::TriggerPress,
            ));

            let open = open.clone();
            let on_activate: OnActivate =
                Arc::new(move |host, action_cx: ActionCx, _reason: ActivateReason| {
                    let prev = host.models_mut().get_copied(&open).unwrap_or(false);
                    let _ = host.models_mut().update(&open, |v| *v = !prev);
                    host.request_redraw(action_cx.window);
                });
            cx.pressable_add_on_activate(on_activate);

            let caret_icon = if is_open {
                fret_icons::ids::ui::CHEVRON_UP
            } else {
                fret_icons::ids::ui::CHEVRON_DOWN
            };

            let divider = frame_chrome.border;

            vec![editor_input_group_frame(
                cx,
                LayoutStyle {
                    size: SizeStyle {
                        width: Length::Fill,
                        height: Length::Fill,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                density,
                frame_chrome,
                EditorFrameState {
                    enabled,
                    hovered: state.hovered,
                    pressed: state.pressed,
                    focused: state.focused,
                    open: is_open,
                    semantic: EditorFrameSemanticState::default(),
                },
                move |cx, visuals| {
                    let text_el = editor_input_value_text(
                        cx,
                        density,
                        Px(12.0),
                        trigger_text.clone(),
                        visuals.fg,
                        Length::Auto,
                    );
                    let text = editor_input_group_inset(cx, frame_chrome.padding, text_el);

                    let sep = editor_input_group_divider(cx, divider);

                    let caret = cx.container(
                        ContainerProps {
                            layout: LayoutStyle {
                                size: SizeStyle {
                                    width: Length::Px(density.hit_thickness),
                                    height: Length::Fill,
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                            padding: Edges::all(Px(0.0)).into(),
                            ..Default::default()
                        },
                        move |cx| {
                            vec![cx.flex(
                                FlexProps {
                                    layout: LayoutStyle {
                                        size: SizeStyle {
                                            width: Length::Fill,
                                            height: Length::Fill,
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    direction: Axis::Horizontal,
                                    gap: SpacingLength::Px(Px(0.0)),
                                    padding: Edges::all(Px(0.0)).into(),
                                    justify: MainAlign::Center,
                                    align: CrossAlign::Center,
                                    wrap: false,
                                },
                                move |cx| {
                                    vec![editor_icon_with(
                                        cx,
                                        density,
                                        caret_icon,
                                        Some(Px(12.0)),
                                        Some(fret_ui_kit::ColorRef::Color(visuals.icon)),
                                    )]
                                },
                            )]
                        },
                    );

                    vec![editor_input_group_row(cx, Px(0.0), vec![text, sep, caret])]
                },
            )]
        },
    )
}
