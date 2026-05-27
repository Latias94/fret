use std::sync::Arc;

use fret_core::{Edges, Px, SemanticsRole};
use fret_runtime::ActionId;
use fret_ui::element::{
    AnyElement, ContainerProps, Length, PressableState, RowProps, SemanticsDecoration, SpacerProps,
    SpacingLength,
};
use fret_ui::elements::GlobalElementId;
use fret_ui::{ElementContext, UiHost};

use crate::imui::{MenuItemOptions, ResponseExt};

use super::interaction;
use super::visual;

pub(super) fn menu_item_element_with_pressable_hook_inner<H: UiHost, F>(
    cx: &mut ElementContext<'_, H>,
    label: Arc<str>,
    options: MenuItemOptions,
    role: SemanticsRole,
    checked: Option<bool>,
    action: Option<ActionId>,
    pressable_hook: F,
    response: &mut ResponseExt,
) -> AnyElement
where
    F: Clone
        + for<'cx> Fn(&mut fret_ui::ElementContext<'cx, H>, PressableState, GlobalElementId, bool),
{
    let pressable_hook = pressable_hook.clone();
    let mut panel = ContainerProps::default();
    panel.layout.size.width = Length::Fill;
    panel.layout.size.height = Length::Auto;
    panel.padding = Edges {
        left: Px(6.0),
        right: Px(6.0),
        top: Px(2.0),
        bottom: Px(2.0),
    }
    .into();

    let test_id = options.test_id.clone();
    let shortcut = options.shortcut.clone();
    let shortcut_test_id = options.shortcut_test_id.clone().or_else(|| {
        test_id
            .as_ref()
            .map(|test_id| Arc::from(format!("{test_id}.shortcut")))
    });
    let submenu = options.submenu;
    let label_for_visuals = label.clone();
    let interaction =
        interaction::resolve_menu_item_interaction(cx, &label, &options, role, checked, action);
    let runtime = interaction.runtime;

    cx.pressable_with_id(interaction.props, move |cx, state, id| {
        let pressable_hook = pressable_hook.clone();
        let behavior = interaction::install_menu_item_interaction(cx, id, &runtime);

        pressable_hook(cx, state, id, runtime.enabled);

        interaction::populate_menu_item_response(
            cx,
            id,
            state,
            &behavior,
            runtime.enabled,
            response,
        );

        let visuals = cx.container(panel, move |cx| {
            let mut row = RowProps::default();
            row.layout.size.width = Length::Fill;
            row.layout.size.height = Length::Auto;
            row.gap = SpacingLength::Px(Px(6.0));

            let indicator = match (role, checked) {
                (SemanticsRole::MenuItemCheckbox, Some(true)) => Some(Arc::from("\u{2713}")),
                (SemanticsRole::MenuItemCheckbox, Some(false)) => Some(Arc::from(" ")),
                (SemanticsRole::MenuItemRadio, Some(true)) => Some(Arc::from("\u{25CF}")),
                (SemanticsRole::MenuItemRadio, Some(false)) => Some(Arc::from(" ")),
                _ => None,
            };

            vec![cx.row(row, move |cx| {
                let mut out: Vec<AnyElement> = Vec::new();
                if let Some(indicator) = indicator.clone() {
                    out.push(visual::menu_item_indicator_text(cx, indicator));
                }
                out.push(visual::menu_item_label_text(cx, label_for_visuals.clone()));

                if let Some(shortcut) = shortcut.clone() {
                    out.push(cx.spacer(SpacerProps::default()));

                    let mut shortcut = visual::menu_item_shortcut_text(cx, shortcut);
                    if let Some(test_id) = shortcut_test_id.clone() {
                        shortcut = shortcut
                            .attach_semantics(SemanticsDecoration::default().test_id(test_id));
                    }
                    out.push(shortcut);
                } else if submenu {
                    out.push(cx.spacer(SpacerProps::default()));
                    out.push(visual::menu_item_indicator_text(cx, Arc::from("\u{203A}")));
                }
                out
            })]
        });

        vec![visuals]
    })
}
