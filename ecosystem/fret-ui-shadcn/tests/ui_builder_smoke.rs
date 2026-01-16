use std::sync::Arc;

use fret_runtime::ModelStore;
use fret_ui_shadcn::prelude::*;
use fret_ui_shadcn::{
    Alert, AlertDialogContent, Badge, Breadcrumb, Button, Card, Command, CommandInput, CommandItem,
    CommandPalette, DialogContent, DrawerContent, HoverCardContent, Kbd, PopoverContent, Progress,
    Select, SheetContent, Slider, Switch, TabsRoot, TooltipContent,
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
    let _ = AlertDialogContent::new(Vec::new()).ui().p_4().build();
    let _ = SheetContent::new(Vec::new()).ui().p_4().build();
    let _ = HoverCardContent::new(Vec::new()).ui().p_4().build();
    let _ = DrawerContent::new(Vec::new()).ui().p_4().build();
}
