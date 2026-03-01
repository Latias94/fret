use std::sync::Arc;

use fret_core::{Color, Corners, Px, SemanticsRole, TextOverflow, TextStyle, TextWrap};
use fret_runtime::CommandId;
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, FlexProps, LayoutStyle, MainAlign, PressableA11y, PressableProps,
    SemanticsDecoration, TextInkOverflow, TextProps,
};
use fret_ui::{ElementContext, UiHost};

use super::intent::{WorkspaceTabStripIntent, dispatch_intent};
use super::layouts::{fill_layout, fixed_square_layout};

pub(super) fn tab_close_button<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    close_command: CommandId,
    pane_activate_cmd: Option<CommandId>,
    hover_bg: Color,
    text_style: TextStyle,
    tab_fg: Color,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    cx.pressable(
        PressableProps {
            layout: fixed_square_layout(Px(18.0)),
            focusable: false,
            a11y: PressableA11y {
                role: Some(SemanticsRole::Button),
                label: Some(Arc::from("Close tab")),
                test_id,
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, close_state| {
            let close_handler: OnActivate = {
                let close_command = close_command.clone();
                let pane_activate_cmd = pane_activate_cmd.clone();
                Arc::new(move |host, acx, _reason| {
                    if let Some(cmd) = pane_activate_cmd.clone() {
                        dispatch_intent(host, acx.window, WorkspaceTabStripIntent::Activate(cmd));
                    }
                    dispatch_intent(
                        host,
                        acx.window,
                        WorkspaceTabStripIntent::Close(close_command.clone()),
                    );
                })
            };
            cx.pressable_on_activate(close_handler);

            let bg = if close_state.hovered || close_state.pressed {
                Some(hover_bg)
            } else {
                None
            };

            vec![cx.container(
                ContainerProps {
                    layout: fill_layout(),
                    background: bg,
                    corner_radii: Corners::all(Px(4.0)),
                    ..Default::default()
                },
                |cx| {
                    vec![cx.text_props(TextProps {
                        layout: LayoutStyle::default(),
                        text: Arc::from("×"),
                        style: Some(text_style.clone()),
                        color: Some(tab_fg),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                        align: fret_core::TextAlign::Start,
                        ink_overflow: TextInkOverflow::None,
                    })]
                },
            )]
        },
    )
}

#[cfg(feature = "shadcn-context-menu")]
pub(super) fn overflow_menu_close_slot<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    text_style: TextStyle,
    tab_fg: Color,
    test_id: Option<Arc<str>>,
) -> AnyElement {
    let mut el = cx.container(
        ContainerProps {
            layout: fixed_square_layout(Px(18.0)),
            corner_radii: Corners::all(Px(4.0)),
            ..Default::default()
        },
        move |cx| {
            vec![cx.text_props(TextProps {
                layout: LayoutStyle::default(),
                text: Arc::from("×"),
                style: Some(text_style.clone()),
                color: Some(tab_fg),
                wrap: TextWrap::None,
                overflow: TextOverflow::Clip,
                align: fret_core::TextAlign::Start,
                ink_overflow: TextInkOverflow::None,
            })]
        },
    );

    el = el.attach_semantics({
        let mut deco = SemanticsDecoration::default()
            .role(SemanticsRole::Button)
            .label("Close tab");
        if let Some(id) = test_id {
            deco = deco.test_id(id);
        }
        deco
    });
    el
}

pub(super) fn tab_dirty_indicator<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    dirty_fg: Color,
    text_style: TextStyle,
) -> AnyElement {
    let mut dot_style = text_style;
    dot_style.size = Px((dot_style.size.0 - 1.0).max(10.0));

    cx.container(
        ContainerProps {
            layout: fixed_square_layout(Px(18.0)),
            ..Default::default()
        },
        move |cx| {
            vec![cx.flex(
                FlexProps {
                    layout: fill_layout(),
                    direction: fret_core::Axis::Horizontal,
                    justify: MainAlign::Center,
                    align: fret_ui::element::CrossAlign::Center,
                    ..Default::default()
                },
                move |cx| {
                    vec![cx.text_props(TextProps {
                        layout: LayoutStyle::default(),
                        text: Arc::from("•"),
                        style: Some(dot_style.clone()),
                        color: Some(dirty_fg),
                        wrap: TextWrap::None,
                        overflow: TextOverflow::Clip,
                        align: fret_core::TextAlign::Start,
                        ink_overflow: TextInkOverflow::None,
                    })]
                },
            )]
        },
    )
}

pub(super) fn tab_trailing_slot_placeholder<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
) -> AnyElement {
    cx.container(
        ContainerProps {
            layout: fixed_square_layout(Px(18.0)),
            ..Default::default()
        },
        |_cx| Vec::new(),
    )
}
