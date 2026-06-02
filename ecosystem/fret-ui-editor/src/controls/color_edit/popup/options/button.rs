use std::sync::Arc;

use fret_core::{Corners, Edges, Px, SemanticsRole};
use fret_ui::action::OnActivate;
use fret_ui::element::{
    AnyElement, ContainerProps, LayoutStyle, Length, PressableA11y, PressableProps, SizeStyle,
};
use fret_ui::{ElementContext, Theme, UiHost};

use crate::primitives::colors::{editor_accent, editor_border};
use crate::primitives::popup_list::{
    EditorPopupListRowState, editor_popup_list_row_palette, editor_popup_list_row_radius,
};
use crate::primitives::readout::editor_popup_list_centered_row_text_props;

pub(in crate::controls::color_edit::popup) fn option_button<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    label: &'static str,
    role: SemanticsRole,
    selected: bool,
    enabled: bool,
    row_height: Px,
    test_id: Option<Arc<str>>,
    on_activate: OnActivate,
) -> AnyElement {
    let label = Arc::<str>::from(label);
    let a11y_label = label.clone();
    let mut button = cx.pressable(
        PressableProps {
            layout: LayoutStyle {
                size: SizeStyle {
                    width: Length::Fill,
                    height: Length::Px(row_height),
                    ..Default::default()
                },
                ..Default::default()
            },
            enabled,
            focusable: enabled,
            a11y: PressableA11y {
                role: Some(role),
                label: Some(a11y_label),
                checked: matches!(role, SemanticsRole::Checkbox | SemanticsRole::RadioButton)
                    .then_some(selected),
                ..Default::default()
            },
            ..Default::default()
        },
        move |cx, st| {
            cx.pressable_add_on_activate(on_activate.clone());
            let (bg, fg, border) = {
                let theme = Theme::global(&*cx.app);
                let palette = editor_popup_list_row_palette(
                    theme,
                    st.hovered || st.hovered_raw,
                    EditorPopupListRowState {
                        active: selected,
                        disabled: !enabled,
                    },
                );
                let border = if selected {
                    editor_accent(theme)
                } else {
                    editor_border(theme)
                };
                (palette.bg, palette.fg, border)
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
                    border: Edges::all(Px(1.0)),
                    border_color: Some(border),
                    corner_radii: Corners::all(editor_popup_list_row_radius()),
                    ..Default::default()
                },
                {
                    let label = label.clone();
                    move |cx| {
                        vec![cx.text_props(editor_popup_list_centered_row_text_props(
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
        button = button.test_id(test_id);
    }
    button
}
