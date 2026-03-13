use std::sync::Arc;

use fret_core::SemanticsRole;
use fret_runtime::ModelStore;
use fret_ui_headless::calendar::{CalendarMonth, DateRangeSelection};
use fret_ui_headless::table::{ColumnDef, RowKey, TableState};
use fret_ui_kit::prelude::*;
use fret_ui_shadcn::experimental::{DataGridElement, DataGridRowState};
use fret_ui_shadcn::{
    Alert, AlertAction, AlertDescription, AlertDialog, AlertDialogAction, AlertDialogCancel,
    AlertDialogContent, AlertDialogDescription, AlertDialogFooter, AlertDialogHeader,
    AlertDialogTitle, AlertTitle, AvatarImage, Badge, Breadcrumb, Button, Card, CardContent,
    CardDescription, CardFooter, CardHeader, CardTitle, Collapsible, Combobox, Command,
    CommandDialog, CommandEmpty, CommandInput, CommandItem, CommandList, CommandPalette,
    CommandShortcut, ContextMenu, ContextMenuEntry, DataGridCanvas, DataGridCanvasAxis, DataTable,
    DataTableGlobalFilterInput, DataTableViewOptionItem, DataTableViewOptions, Dialog,
    DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, Drawer,
    DrawerContent, DrawerFooter, DrawerHeader, DropdownMenu, DropdownMenuEntry, Empty, Field,
    FieldGroup, FieldLabel, FieldSet, HoverCardContent, Kbd, Menubar, Popover, PopoverContent,
    PopoverDescription, PopoverHeader, PopoverTitle, Progress, ScrollArea, Select, Sheet,
    SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetTitle, Slider, Switch,
    TableBody, TableCaption, TableFooter, TableHead, TableHeader, TableRow, TabsRoot, Toaster,
    TooltipContent,
};
use time::{Date, OffsetDateTime};

#[allow(dead_code, unused_variables)]
fn ui_builder_overlay_roots_compile<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    dialog_open: Model<bool>,
    popover_open: Model<bool>,
    sheet_open: Model<bool>,
    drawer_open: Model<bool>,
    dropdown_menu_open: Model<bool>,
    context_menu_open: Model<bool>,
    alert_dialog_open: Model<bool>,
    command_dialog_open: Model<bool>,
    command_dialog_query: Model<String>,
    collapsible_open: Model<bool>,
    data_table_state: Model<TableState>,
) {
    let _ = fret_ui_shadcn::DialogUiBuilderExt::into_element(
        Dialog::new(dialog_open.clone()).ui(),
        cx,
        |cx| Button::new("trigger").into_element(cx),
        |cx| DialogContent::new(Vec::new()).into_element(cx),
    );

    let _ = fret_ui_shadcn::PopoverUiBuilderExt::into_element(
        Popover::new(popover_open.clone()).ui(),
        cx,
        |cx| Button::new("trigger").into_element(cx),
        |cx| PopoverContent::new(Vec::new()).into_element(cx),
    );

    let _ = fret_ui_shadcn::SheetUiBuilderExt::into_element(
        Sheet::new(sheet_open.clone()).ui(),
        cx,
        |cx| Button::new("trigger").into_element(cx),
        |cx| SheetContent::new(Vec::new()).into_element(cx),
    );

    let _ = fret_ui_shadcn::DrawerUiBuilderExt::into_element(
        Drawer::new(drawer_open.clone()).ui(),
        cx,
        |cx| Button::new("trigger").into_element(cx),
        |cx| DrawerContent::new(Vec::new()).into_element(cx),
    );

    let _ = fret_ui_shadcn::DropdownMenuUiBuilderExt::into_element(
        DropdownMenu::new(dropdown_menu_open.clone()).ui(),
        cx,
        |cx| Button::new("trigger").into_element(cx),
        |_cx| Vec::<DropdownMenuEntry>::new(),
    );

    let _ = fret_ui_shadcn::ContextMenuUiBuilderExt::into_element(
        ContextMenu::new(context_menu_open.clone()).ui(),
        cx,
        |cx| Button::new("trigger").into_element(cx),
        |_cx| Vec::<ContextMenuEntry>::new(),
    );

    let _ = fret_ui_shadcn::AlertDialogUiBuilderExt::into_element(
        AlertDialog::new(alert_dialog_open.clone()).ui(),
        cx,
        |cx| Button::new("trigger").into_element(cx),
        |cx| AlertDialogContent::new(Vec::new()).into_element(cx),
    );

    let _ = fret_ui_shadcn::CommandDialogUiBuilderExt::into_element(
        CommandDialog::new(
            command_dialog_open.clone(),
            command_dialog_query.clone(),
            Vec::<CommandItem>::new(),
        )
        .ui(),
        cx,
        |cx| Button::new("trigger").into_element(cx),
    );

    let _ = fret_ui_shadcn::CollapsibleUiBuilderExt::into_element(
        Collapsible::new(collapsible_open.clone()).ui(),
        cx,
        |cx, _is_open| Button::new("trigger").into_element(cx),
        |cx| Empty::new(vec![cx.text("content")]).into_element(cx),
    );

    let grid_keys = Arc::new(vec![0u64]);
    let rows = DataGridCanvasAxis::new(grid_keys.clone(), 0, Px(24.0));
    let cols = DataGridCanvasAxis::new(grid_keys, 0, Px(80.0));
    let _ = fret_ui_shadcn::DataGridCanvasUiBuilderExt::into_element(
        DataGridCanvas::new(rows, cols).ui(),
        cx,
        |_row, _col| Arc::from("cell"),
    );

    let _ = fret_ui_shadcn::DataGridElementUiBuilderExt::into_element(
        DataGridElement::new(["A"], 1).ui(),
        cx,
        0,
        0,
        |i| i as u64,
        |_i| DataGridRowState::default(),
        |cx, _row, _col| cx.text("cell"),
    );

    let data: Arc<[u32]> = Arc::from(Vec::<u32>::new().into_boxed_slice());
    let columns: Arc<[ColumnDef<u32>]> =
        Arc::from(vec![ColumnDef::<u32>::new("a")].into_boxed_slice());
    let _ = fret_ui_shadcn::DataTableUiBuilderExt::into_element(
        DataTable::new().ui(),
        cx,
        data,
        0,
        data_table_state,
        columns,
        |_row, index, _parent| RowKey::from_index(index),
        |col| Arc::clone(&col.id),
        |cx, _col, _row| cx.text("cell"),
    );

    let _ = Alert::new(Vec::new())
        .ui()
        .p_4()
        .border_1()
        .into_element(cx);
    let _ = Alert::build(|cx, out| {
        out.push_ui(cx, AlertTitle::new("x"));
        out.push_ui(cx, AlertDescription::new("y"));
        out.push_ui(
            cx,
            AlertAction::build(|cx, out| {
                out.push(cx.text("Undo"));
            }),
        );
    })
    .ui()
    .p_4()
    .border_1()
    .into_element(cx);
    let _ = Badge::new("x").ui().px_2().into_element(cx);
    let _ = Kbd::new("x").ui().px_2().into_element(cx);
}

#[allow(dead_code, unused_variables)]
fn ui_builder_nested_surfaces_compile<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    composable_accordion_value: Model<Option<Arc<str>>>,
) {
    // Composable accordion surface (`accordion::composable`).
    {
        use fret_ui_shadcn::accordion::composable;

        let trigger = composable::AccordionTrigger::new(vec![cx.text("Trigger")])
            .ui()
            .p_2()
            .build();

        let content = composable::AccordionContent::new(vec![cx.text("Content")])
            .ui()
            .p_2()
            .build();

        let item = composable::AccordionItem::new("item")
            .trigger(trigger)
            .content(content)
            .ui()
            .p_2()
            .build();

        let _ = composable::AccordionRoot::single(composable_accordion_value)
            .item(item)
            .ui()
            .w_full()
            .into_element(cx);
    }

    // Breadcrumb primitives (`breadcrumb::primitives`).
    {
        use fret_ui_shadcn::breadcrumb::primitives;

        let _ = fret_ui_shadcn::BreadcrumbPrimitivesUiBuilderExt::into_element(
            primitives::Breadcrumb::new().ui().p_2(),
            cx,
            |cx| {
                vec![
                    fret_ui_shadcn::BreadcrumbPrimitivesUiBuilderExt::into_element(
                        primitives::BreadcrumbList::new().ui().p_2(),
                        cx,
                        |cx| {
                            vec![
                                fret_ui_shadcn::BreadcrumbPrimitivesUiBuilderExt::into_element(
                                    primitives::BreadcrumbItem::new().ui().p_2(),
                                    cx,
                                    |cx| {
                                        vec![
                                            primitives::BreadcrumbLink::new("Home")
                                                .ui()
                                                .p_0()
                                                .into_element(cx),
                                        ]
                                    },
                                ),
                            ]
                        },
                    ),
                ]
            },
        );

        let _ = primitives::BreadcrumbSeparator::new().ui().into_element(cx);
        let _ = primitives::BreadcrumbEllipsis::new().ui().into_element(cx);
        let _ = primitives::BreadcrumbPage::new("Page")
            .ui()
            .into_element(cx);
    }
}

#[allow(dead_code, unused_variables)]
fn ui_builder_semantics_decorators_compile<H: UiHost>(cx: &mut ElementContext<'_, H>) {
    let _ = Button::new("Click")
        .ui()
        .test_id("smoke.button")
        .a11y_role(SemanticsRole::Button)
        .a11y_label("Smoke Button")
        .into_element(cx);

    let _ = Badge::new("x")
        .ui()
        .test_id("smoke.badge")
        .a11y_label("Smoke Badge")
        .into_element(cx);
}

#[test]
fn ui_builder_smoke_applies_supported_patches() {
    let mut store = ModelStore::default();
    let switch_model = store.insert(false);
    let slider_model = store.insert(vec![0.0_f32]);
    let select_model = store.insert(None::<Arc<str>>);
    let select_open = store.insert(false);
    let command_input_model = store.insert(String::new());
    let command_palette_model = store.insert(String::new());
    let progress_model = store.insert(0.5_f32);
    let alert_dialog_open = store.insert(false);
    let calendar_month = store.insert(CalendarMonth::from_date(OffsetDateTime::now_utc().date()));
    let calendar_selected = store.insert(None::<Date>);
    let calendar_range_selected = store.insert(DateRangeSelection::default());
    let date_picker_open = store.insert(false);
    let date_range_picker_open = store.insert(false);
    let radio_group_model = store.insert(None::<Arc<str>>);
    let dialog_open = store.insert(false);
    let popover_open = store.insert(false);
    let sheet_open = store.insert(false);
    let drawer_open = store.insert(false);
    let dropdown_menu_open = store.insert(false);
    let context_menu_open = store.insert(false);
    let combobox_open = store.insert(false);
    let combobox_value = store.insert(None::<Arc<str>>);
    let data_table_filter = store.insert(String::new());
    let data_table_view_options_open = store.insert(false);
    let data_table_view_option_checked = store.insert(false);
    let command_dialog_open = store.insert(false);
    let command_dialog_query = store.insert(String::new());
    let _composable_accordion_value = store.insert(None::<Arc<str>>);

    let _ = Button::new("OK").ui().px_3().w_full().build();
    let _ = Alert::new(Vec::new()).ui().p_4().border_1().build();
    let _ = Alert::build::<fret_ui_app::App, _>(|cx, out| {
        out.push_ui(cx, AlertTitle::new("x"));
        out.push_ui(cx, AlertDescription::new("y"));
    })
    .ui()
    .p_4()
    .border_1()
    .build();
    let _ = ScrollArea::build::<fret_ui_app::App, _>(|cx, out: &mut Vec<AnyElement>| {
        out.push_ui(cx, ui::text("row"));
    })
    .ui()
    .w_full()
    .build();
    let _ = FieldSet::build::<fret_ui_app::App, _>(|cx, out: &mut Vec<AnyElement>| {
        out.push_ui(
            cx,
            FieldGroup::build(|cx, out| {
                out.push_ui(
                    cx,
                    Field::build(|cx, out| {
                        out.push_ui(cx, FieldLabel::new("Email"));
                        out.push_ui(cx, ui::text("value"));
                    }),
                );
            }),
        );
    })
    .ui()
    .w_full()
    .build();
    let _ = Badge::new("x").ui().px_2().build();
    let _ = Kbd::new("x").ui().px_2().build();
    let _ = AvatarImage::maybe(None).ui().px_2().build();
    let _ = Breadcrumb::new().ui().px_2().build();
    let _ = Empty::new(Vec::new()).ui().p_4().border_1().build();
    let _ = Card::new(Vec::new())
        .ui()
        .p_4()
        .border_1()
        .rounded_md()
        .build();
    let _ = CardHeader::new(Vec::new()).ui().build();
    let _ = CardContent::new(Vec::new()).ui().build();
    let _ = CardFooter::new(Vec::new()).ui().build();
    let _ = CardFooter::build::<fret_ui_app::App, _>(|_cx, _out| {})
        .ui()
        .build();
    let _ = CardTitle::new("x").ui().build();
    let _ = CardDescription::new("x").ui().build();

    let _ = Switch::new(switch_model).ui().px_2().build();
    let _ = Slider::new(slider_model).ui().px_2().w_full().build();
    let _ = Select::new(select_model, select_open)
        .ui()
        .px_2()
        .w_full()
        .build();

    let _ = Progress::new(progress_model).ui().w_full().build();
    let _ = TabsRoot::uncontrolled::<Arc<str>>(None).ui().p_4().build();

    let _ = PopoverContent::new(Vec::new()).ui().p_4().build();
    let _ = TooltipContent::new(Vec::new()).ui().p_4().build();

    let _ = Dialog::new(dialog_open).ui().build();
    let _ = AlertDialog::new(alert_dialog_open.clone()).ui().build();
    let _ = Popover::new(popover_open).ui().build();
    let _ = Sheet::new(sheet_open).ui().build();
    let _ = Drawer::new(drawer_open).ui().build();
    let _ = DropdownMenu::new(dropdown_menu_open).ui().build();
    let _ = ContextMenu::new(context_menu_open).ui().build();
    let _ = Menubar::new(Vec::new()).ui().px_2().build();
    let _ = Combobox::new(combobox_value, combobox_open)
        .ui()
        .px_2()
        .build();
    let _ = Toaster::new().ui().build();

    let _ = DataTableGlobalFilterInput::new(data_table_filter)
        .ui()
        .build();
    let _ = DataTableViewOptions::new(
        data_table_view_options_open,
        vec![DataTableViewOptionItem::new(
            data_table_view_option_checked,
            "col",
        )],
    )
    .ui()
    .build();

    let _ = fret_ui_shadcn::Calendar::new(calendar_month.clone(), calendar_selected.clone())
        .ui()
        .p_4()
        .build();
    let _ =
        fret_ui_shadcn::CalendarRange::new(calendar_month.clone(), calendar_range_selected.clone())
            .ui()
            .p_4()
            .build();
    let _ = fret_ui_shadcn::DatePicker::new(
        date_picker_open,
        calendar_month.clone(),
        calendar_selected.clone(),
    )
    .ui()
    .p_4()
    .build();
    let _ = fret_ui_shadcn::DateRangePicker::new(
        date_range_picker_open,
        calendar_month,
        calendar_range_selected,
    )
    .ui()
    .p_4()
    .build();
    let _ = fret_ui_shadcn::RadioGroup::new(radio_group_model)
        .ui()
        .p_4()
        .build();

    let _ = Command::new(Vec::new()).ui().p_4().build();
    let _ = CommandInput::new(command_input_model)
        .ui()
        .px_2()
        .w_full()
        .build();
    let _ = CommandPalette::new(command_palette_model, Vec::<CommandItem>::new())
        .ui()
        .p_4()
        .build();
    let _ = CommandEmpty::new("No results.").ui().build();
    let _ = CommandList::new(Vec::<CommandItem>::new()).ui().build();
    let _ = CommandShortcut::new("Ctrl+K").ui().build();
    let _ = CommandDialog::new(
        command_dialog_open,
        command_dialog_query,
        Vec::<CommandItem>::new(),
    )
    .ui()
    .build();

    let _ = DialogContent::new(Vec::new()).ui().p_4().build();
    let _ = DialogHeader::new(Vec::new()).ui().build();
    let _ = DialogFooter::new(Vec::new()).ui().build();
    let _ = DialogTitle::new("x").ui().build();
    let _ = DialogDescription::new("x").ui().build();
    let _ = AlertDialogContent::new(Vec::new()).ui().p_4().build();
    let _ = AlertDialogHeader::new(Vec::new()).ui().build();
    let _ = AlertDialogFooter::new(Vec::new()).ui().build();
    let _ = AlertDialogTitle::new("x").ui().build();
    let _ = AlertDialogDescription::new("x").ui().build();
    let _ = AlertDialogAction::new("x", alert_dialog_open.clone())
        .ui()
        .build();
    let _ = AlertDialogCancel::new("x", alert_dialog_open).ui().build();
    let _ = SheetContent::new(Vec::new()).ui().p_4().build();
    let _ = SheetHeader::new(Vec::new()).ui().build();
    let _ = SheetFooter::new(Vec::new()).ui().build();
    let _ = SheetTitle::new("x").ui().build();
    let _ = SheetDescription::new("x").ui().build();
    let _ = HoverCardContent::new(Vec::new()).ui().p_4().build();
    let _ = DrawerContent::new(Vec::new()).ui().p_4().build();
    let _ = DrawerHeader::new(Vec::new()).ui().build();
    let _ = DrawerFooter::new(Vec::new()).ui().build();
    let _ = PopoverHeader::new(Vec::new()).ui().build();
    let _ = PopoverTitle::new("x").ui().build();
    let _ = PopoverDescription::new("x").ui().build();
    let _ = TableHeader::new(Vec::new()).ui().build();
    let _ = TableBody::new(Vec::new()).ui().build();
    let _ = TableFooter::new(Vec::new()).ui().build();
    let _ = TableRow::new(1, Vec::new()).ui().build();
    let _ = TableHead::new("x").ui().build();
    let _ = TableCaption::new("x").ui().build();
}
