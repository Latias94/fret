use fret_core::{Modifiers, Point};

use super::ResponseExt;

impl ResponseExt {
    pub(crate) fn set_secondary_clicked(&mut self, secondary_clicked: bool) {
        self.secondary_clicked = secondary_clicked;
    }

    pub(crate) fn set_double_clicked(&mut self, double_clicked: bool) {
        self.double_clicked = double_clicked;
    }

    pub(crate) fn set_long_pressed(&mut self, long_pressed: bool) {
        self.long_pressed = long_pressed;
    }

    pub(crate) fn set_press_holding(&mut self, press_holding: bool) {
        self.press_holding = press_holding;
    }

    pub(crate) fn set_context_menu_requested(&mut self, context_menu_requested: bool) {
        self.context_menu_requested = context_menu_requested;
    }

    pub(crate) fn set_context_menu_anchor(&mut self, context_menu_anchor: Option<Point>) {
        self.context_menu_anchor = context_menu_anchor;
    }

    pub(crate) fn set_pointer_clicked(&mut self, pointer_clicked: bool) {
        self.pointer_clicked = pointer_clicked;
    }

    pub(crate) fn set_pointer_click_modifiers(&mut self, pointer_click_modifiers: Modifiers) {
        self.pointer_click_modifiers = pointer_click_modifiers;
    }

    pub(crate) fn clear_press_context_signals(&mut self) {
        self.secondary_clicked = false;
        self.double_clicked = false;
        self.long_pressed = false;
        self.press_holding = false;
        self.context_menu_requested = false;
        self.context_menu_anchor = None;
        self.pointer_clicked = false;
        self.pointer_click_modifiers = Modifiers::default();
    }

    pub fn secondary_clicked(self) -> bool {
        self.secondary_clicked
    }

    pub fn double_clicked(self) -> bool {
        self.double_clicked
    }

    pub fn long_pressed(self) -> bool {
        self.long_pressed
    }

    pub fn press_holding(self) -> bool {
        self.press_holding
    }

    pub fn context_menu_requested(self) -> bool {
        self.context_menu_requested
    }

    pub fn pointer_clicked(self) -> bool {
        self.pointer_clicked
    }

    pub fn pointer_click_modifiers(self) -> Option<Modifiers> {
        self.pointer_clicked.then_some(self.pointer_click_modifiers)
    }

    pub fn context_menu_anchor(self) -> Option<Point> {
        self.context_menu_anchor
    }
}
