use std::sync::Arc;

use fret_runtime::ModelStore;
use fret_ui_shadcn::prelude::*;
use fret_ui_shadcn::{
    Alert, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription,
    AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, Badge, Breadcrumb, Button, Card,
    CardContent, CardDescription, CardFooter, CardHeader, CardTitle, Command, CommandInput,
    CommandItem, CommandPalette, DialogContent, DialogDescription, DialogFooter, DialogHeader,
    DialogTitle, DrawerContent, DrawerFooter, DrawerHeader, HoverCardContent, Kbd, PopoverContent,
    PopoverDescription, PopoverHeader, PopoverTitle, Progress, Select, SheetContent,
    SheetDescription, SheetFooter, SheetHeader, SheetTitle, Slider, Switch, TableBody,
    TableCaption, TableFooter, TableHead, TableHeader, TableRow, TabsRoot, TooltipContent,
};

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

    let _ = Button::new("OK").ui().px_3().w_full().build();
    let _ = Alert::new(Vec::new()).ui().build();
    let _ = Badge::new("x").ui().build();
    let _ = Kbd::new("x").ui().build();
    let _ = Breadcrumb::new().ui().build();
    let _ = Card::new(Vec::new())
        .ui()
        .p_4()
        .border_1()
        .rounded_md()
        .build();
    let _ = CardHeader::new(Vec::new()).ui().build();
    let _ = CardContent::new(Vec::new()).ui().build();
    let _ = CardFooter::new(Vec::new()).ui().build();
    let _ = CardTitle::new("x").ui().build();
    let _ = CardDescription::new("x").ui().build();

    let _ = Switch::new(switch_model).ui().px_2().build();
    let _ = Slider::new(slider_model).ui().w_full().build();
    let _ = Select::new(select_model, select_open).ui().w_full().build();

    let _ = Progress::new(progress_model).ui().w_full().build();
    let _ = TabsRoot::uncontrolled::<Arc<str>>(None).ui().p_4().build();

    let _ = PopoverContent::new(Vec::new()).ui().p_4().build();
    let _ = TooltipContent::new(Vec::new()).ui().p_4().build();

    let _ = Command::new(Vec::new()).ui().p_4().build();
    let _ = CommandInput::new(command_input_model).ui().w_full().build();
    let _ = CommandPalette::new(command_palette_model, Vec::<CommandItem>::new())
        .ui()
        .p_4()
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
