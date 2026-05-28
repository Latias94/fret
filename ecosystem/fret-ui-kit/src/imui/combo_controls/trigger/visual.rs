use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_ui::element::{
    AnyElement, ContainerProps, Length, MainAlign, PressableA11y, PressableProps, PressableState,
};
use fret_ui::{ElementContext, UiHost};

pub(super) struct ComboTriggerPropsInput {
    pub(super) enabled: bool,
    pub(super) focusable: bool,
    pub(super) a11y_label: Option<Arc<str>>,
    pub(super) test_id: Option<Arc<str>>,
    pub(super) open: bool,
    pub(super) label: Arc<str>,
    pub(super) preview: Arc<str>,
}

pub(super) fn combo_trigger_props(input: ComboTriggerPropsInput) -> PressableProps {
    let mut props = PressableProps::default();
    props.enabled = input.enabled;
    props.focusable = input.enabled && input.focusable;
    props.layout.size.width = Length::Fill;
    props.layout.size.min_height = Some(Length::Px(
        super::super::super::control_chrome::FIELD_MIN_HEIGHT,
    ));
    props.a11y = PressableA11y {
        role: Some(SemanticsRole::ComboBox),
        label: input
            .a11y_label
            .or_else(|| Some(combo_trigger_a11y_label(&input.label, &input.preview))),
        test_id: input.test_id,
        expanded: Some(input.open),
        ..Default::default()
    };
    props
}

pub(super) fn combo_trigger_chrome<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    enabled: bool,
    state: PressableState,
) -> (
    super::super::super::control_chrome::ImUiControlPalette,
    ContainerProps,
) {
    super::super::super::control_chrome::field_chrome(cx, enabled, state)
}

pub(super) fn combo_trigger_children<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    preview: Arc<str>,
    open: bool,
    palette: super::super::super::control_chrome::ImUiControlPalette,
) -> Vec<AnyElement> {
    let state_badge = combo_trigger_state_badge(cx, open, palette);

    vec![cx.flex(
        super::super::super::control_chrome::fill_stack_props(),
        move |cx| {
            let mut out = Vec::new();
            if !label.is_empty() {
                out.push(super::super::super::control_chrome::caption_text(
                    cx,
                    label.clone(),
                    palette,
                ));
            }
            out.push(cx.flex(
                super::super::super::control_chrome::fill_row_props(MainAlign::SpaceBetween),
                move |cx| {
                    vec![
                        super::super::super::control_chrome::fill_text(
                            cx,
                            preview.clone(),
                            palette.foreground,
                        ),
                        state_badge,
                    ]
                },
            ));
            out
        },
    )]
}

pub(in crate::imui::combo_controls) fn combo_trigger_a11y_label(
    label: &str,
    preview: &str,
) -> Arc<str> {
    if label.is_empty() {
        Arc::from(preview)
    } else {
        Arc::from(format!("{label}: {preview}"))
    }
}

fn combo_trigger_state_badge<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    open: bool,
    palette: super::super::super::control_chrome::ImUiControlPalette,
) -> AnyElement {
    if open {
        super::super::super::control_chrome::pill(
            cx,
            Arc::from("Open"),
            palette.accent_background,
            palette.accent_foreground,
        )
    } else {
        super::super::super::control_chrome::pill(
            cx,
            Arc::from("Menu"),
            palette.subtle_background,
            palette.muted_foreground,
        )
    }
}
