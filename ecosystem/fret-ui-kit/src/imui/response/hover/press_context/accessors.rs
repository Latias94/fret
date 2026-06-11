use fret_core::{Modifiers, Point};

use super::ResponseExt;

impl ResponseExt {
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
