use fret_core::{Modifiers, MouseButton};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ModifiersMask {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub alt_gr: bool,
    pub meta: bool,
}

impl ModifiersMask {
    pub const NONE: Self = Self {
        shift: false,
        ctrl: false,
        alt: false,
        alt_gr: false,
        meta: false,
    };

    pub fn matches(self, modifiers: Modifiers, allow_extra: bool) -> bool {
        let required = self;
        if required.shift && !modifiers.shift {
            return false;
        }
        if required.ctrl && !modifiers.ctrl {
            return false;
        }
        if required.alt && !modifiers.alt {
            return false;
        }
        if required.alt_gr && !modifiers.alt_gr {
            return false;
        }
        if required.meta && !modifiers.meta {
            return false;
        }

        if allow_extra {
            return true;
        }

        if modifiers.shift != required.shift {
            return false;
        }
        if modifiers.ctrl != required.ctrl {
            return false;
        }
        if modifiers.alt != required.alt {
            return false;
        }
        if modifiers.alt_gr != required.alt_gr {
            return false;
        }
        if modifiers.meta != required.meta {
            return false;
        }

        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierKey {
    Shift,
    Ctrl,
    Alt,
    AltGr,
    Meta,
}

impl ModifierKey {
    pub fn is_pressed(self, modifiers: Modifiers) -> bool {
        match self {
            Self::Shift => modifiers.shift,
            Self::Ctrl => modifiers.ctrl,
            Self::Alt => modifiers.alt,
            Self::AltGr => modifiers.alt_gr,
            Self::Meta => modifiers.meta,
        }
    }

    pub fn is_required_by(self, required: ModifiersMask) -> bool {
        match self {
            Self::Shift => required.shift,
            Self::Ctrl => required.ctrl,
            Self::Alt => required.alt,
            Self::AltGr => required.alt_gr,
            Self::Meta => required.meta,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PointerChord {
    pub button: MouseButton,
    pub modifiers: ModifiersMask,
    pub allow_extra_modifiers: bool,
}

impl PointerChord {
    pub const fn new(button: MouseButton, modifiers: ModifiersMask) -> Self {
        Self {
            button,
            modifiers,
            allow_extra_modifiers: false,
        }
    }

    pub fn matches(self, button: MouseButton, modifiers: Modifiers) -> bool {
        if self.button != button {
            return false;
        }
        self.modifiers
            .matches(modifiers, self.allow_extra_modifiers)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlotInputMap {
    pub pan: PointerChord,
    pub fit: PointerChord,
    pub box_zoom: PointerChord,
    pub box_zoom_alt: Option<PointerChord>,
    pub box_zoom_cancel: Option<PointerChord>,
    pub box_zoom_expand_x: Option<ModifierKey>,
    pub box_zoom_expand_y: Option<ModifierKey>,
    pub query_drag: Option<PointerChord>,
    pub wheel_zoom_mod: Option<ModifierKey>,
}

impl Default for PlotInputMap {
    fn default() -> Self {
        // ImPlot's default input map (plus a Fret-specific query drag and a Shift+LMB box-zoom
        // alternative for accessibility).
        Self {
            pan: PointerChord::new(MouseButton::Left, ModifiersMask::NONE),
            fit: PointerChord::new(MouseButton::Left, ModifiersMask::NONE),
            box_zoom: PointerChord::new(MouseButton::Right, ModifiersMask::NONE),
            box_zoom_alt: Some(PointerChord::new(
                MouseButton::Left,
                ModifiersMask {
                    shift: true,
                    ..ModifiersMask::NONE
                },
            )),
            box_zoom_cancel: Some(PointerChord::new(MouseButton::Left, ModifiersMask::NONE)),
            box_zoom_expand_x: Some(ModifierKey::Alt),
            box_zoom_expand_y: Some(ModifierKey::Shift),
            query_drag: Some(PointerChord::new(
                MouseButton::Left,
                ModifiersMask {
                    alt: true,
                    ..ModifiersMask::NONE
                },
            )),
            wheel_zoom_mod: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifiers_mask_exact_vs_allow_extra() {
        let required = ModifiersMask {
            shift: true,
            ..ModifiersMask::NONE
        };
        let mods = Modifiers {
            shift: true,
            ctrl: true,
            ..Modifiers::default()
        };
        assert!(!required.matches(mods, false));
        assert!(required.matches(mods, true));
    }
}
