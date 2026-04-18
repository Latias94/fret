use super::super::super::super::*;
use fret::AppComponentCx;

pub(in crate::ui) fn preview_menus(
    cx: &mut AppComponentCx<'_>,
    dropdown_open: Model<bool>,
    context_menu_open: Model<bool>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    let dropdown = shadcn::DropdownMenu::from_open(dropdown_open.clone()).build(
        cx,
        shadcn::Button::new("DropdownMenu")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-menus-dropdown-trigger"),
        |_cx| {
            vec![
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Apple")
                        .test_id("ui-gallery-menus-dropdown-item-apple")
                        .action(CMD_MENU_DROPDOWN_APPLE),
                ),
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Orange").action(CMD_MENU_DROPDOWN_ORANGE),
                ),
                shadcn::DropdownMenuEntry::Separator,
                shadcn::DropdownMenuEntry::Item(
                    shadcn::DropdownMenuItem::new("Disabled").disabled(true),
                ),
            ]
        },
    );

    let context_menu = shadcn::ContextMenu::from_open(context_menu_open.clone()).build(
        cx,
        shadcn::Button::new("ContextMenu (right click)")
            .variant(shadcn::ButtonVariant::Outline)
            .test_id("ui-gallery-menus-context-trigger"),
        |_cx| {
            vec![
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Action")
                        .test_id("ui-gallery-menus-context-item-action")
                        .action(CMD_MENU_CONTEXT_ACTION),
                ),
                shadcn::ContextMenuEntry::Separator,
                shadcn::ContextMenuEntry::Item(
                    shadcn::ContextMenuItem::new("Disabled").disabled(true),
                ),
            ]
        },
    );

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    vec![
        ui::h_row(|_cx| [dropdown, context_menu])
            .gap(Space::N2)
            .items_center()
            .into_element(cx),
        cx.text(format!("last action: {last}")),
    ]
}
