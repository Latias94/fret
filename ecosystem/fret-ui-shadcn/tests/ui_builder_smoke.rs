use std::sync::Arc;

use fret_runtime::ModelStore;
use fret_ui_shadcn::prelude::*;
use fret_ui_shadcn::{
    AlertDialogContent, Button, Card, DialogContent, DrawerContent, HoverCardContent,
    PopoverContent, Select, SheetContent, Slider, Switch, TooltipContent,
};

#[test]
fn ui_builder_smoke_applies_supported_patches() {
    let mut store = ModelStore::default();
    let switch_model = store.insert(false);
    let slider_model = store.insert(vec![0.0_f32]);
    let select_model = store.insert(None::<Arc<str>>);
    let select_open = store.insert(false);

    let _ = Button::new("OK").ui().px_3().w_full().build();
    let _ = Card::new(Vec::new())
        .ui()
        .p_4()
        .border_1()
        .rounded_md()
        .build();

    let _ = Switch::new(switch_model).ui().px_2().build();
    let _ = Slider::new(slider_model).ui().w_full().build();
    let _ = Select::new(select_model, select_open).ui().w_full().build();

    let _ = PopoverContent::new(Vec::new()).ui().p_4().build();
    let _ = TooltipContent::new(Vec::new()).ui().p_4().build();

    let _ = DialogContent::new(Vec::new()).ui().p_4().build();
    let _ = AlertDialogContent::new(Vec::new()).ui().p_4().build();
    let _ = SheetContent::new(Vec::new()).ui().p_4().build();
    let _ = HoverCardContent::new(Vec::new()).ui().p_4().build();
    let _ = DrawerContent::new(Vec::new()).ui().p_4().build();
}
