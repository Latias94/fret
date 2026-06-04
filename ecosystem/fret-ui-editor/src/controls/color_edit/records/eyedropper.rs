use std::sync::Arc;

use fret_core::Color;
use fret_ui::action::{ActionCx, UiActionHost};

/// App-owned eyedropper activation request emitted from an editor `ColorEdit` popup.
///
/// Fret does not currently expose a portable platform screen-sampling contract. This request keeps
/// the editor control useful for apps that already own an eyedropper implementation while avoiding
/// an implicit runtime or renderer readback dependency.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorEditEyedropperRequest {
    current: Color,
    show_alpha: bool,
}

impl ColorEditEyedropperRequest {
    pub fn new(current: Color, show_alpha: bool) -> Self {
        Self {
            current,
            show_alpha,
        }
    }

    pub fn current(self) -> Color {
        self.current
    }

    pub fn show_alpha(self) -> bool {
        self.show_alpha
    }

    pub fn apply_sample(self, sampled: Color) -> Color {
        if self.show_alpha {
            sampled
        } else {
            let mut next = sampled;
            next.a = self.current.a;
            next
        }
    }
}

/// App-owned eyedropper activation hook for editor `ColorEdit`.
///
/// Return `Some(sampled_color)` for synchronous sampling and the control will update its color
/// model, draft text, and validation state. Return `None` for asynchronous app/platform flows.
pub type OnColorEditEyedropper = Arc<
    dyn Fn(&mut dyn UiActionHost, ActionCx, ColorEditEyedropperRequest) -> Option<Color> + 'static,
>;
