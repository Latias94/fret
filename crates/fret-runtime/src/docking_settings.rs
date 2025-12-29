use fret_core::Modifiers;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct DockingInteractionSettings {
    pub drag_inversion: DockDragInversionSettings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DockDragInversionSettings {
    pub modifier: DockDragInversionModifier,
    pub policy: DockDragInversionPolicy,
}

impl Default for DockDragInversionSettings {
    fn default() -> Self {
        Self {
            modifier: DockDragInversionModifier::Shift,
            policy: DockDragInversionPolicy::DockByDefault,
        }
    }
}

impl DockDragInversionSettings {
    pub fn wants_dock_previews(self, modifiers: Modifiers) -> bool {
        let modifier_down = self.modifier.is_down(modifiers);
        match self.policy {
            DockDragInversionPolicy::DockByDefault => !modifier_down,
            DockDragInversionPolicy::DockOnlyWhenModifier => modifier_down,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockDragInversionModifier {
    None,
    Shift,
    Ctrl,
    Alt,
    AltGr,
    Meta,
}

impl DockDragInversionModifier {
    pub fn is_down(self, modifiers: Modifiers) -> bool {
        match self {
            Self::None => false,
            Self::Shift => modifiers.shift,
            Self::Ctrl => modifiers.ctrl,
            Self::Alt => modifiers.alt,
            Self::AltGr => modifiers.alt_gr,
            Self::Meta => modifiers.meta,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockDragInversionPolicy {
    DockByDefault,
    DockOnlyWhenModifier,
}
