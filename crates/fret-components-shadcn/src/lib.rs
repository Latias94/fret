//! shadcn/ui v4-aligned component facade.
//!
//! This crate is a **naming + taxonomy surface** intended to mirror shadcn/ui (v4) so users can
//! transfer knowledge and recipes directly.
//!
//! Implementation-wise, it currently re-exports the underlying building blocks from
//! `fret-components-ui`. Over time, shadcn-specific variants/sizes/policies should converge here,
//! while `fret-components-ui` remains the reusable component infrastructure (StyledExt, tokens,
//! recipes glue, etc.).

pub mod button;

pub use button::{Button, ButtonSize, ButtonVariant};

pub mod input {
    pub use fret_components_ui::text_field::TextField as Input;
}

pub mod textarea {
    pub use fret_components_ui::text_area_field::TextAreaField as Textarea;
}

pub mod separator {
    pub use fret_components_ui::separator::Separator;
}

pub mod checkbox {
    pub use fret_components_ui::checkbox::*;
}

pub mod switch {
    pub use fret_components_ui::switch::*;
}

pub mod tabs {
    pub use fret_components_ui::tabs::*;
}

pub mod select {
    pub use fret_components_ui::select::*;
}

pub mod tooltip {
    pub use fret_components_ui::tooltip::*;
}

pub mod toast {
    pub use fret_components_ui::toast::*;
}

pub mod popover {
    pub use fret_components_ui::popover::*;
}

pub mod dialog {
    pub use fret_components_ui::dialog::*;
    pub use fret_components_ui::dialog_overlay::*;
}

pub mod dropdown_menu {
    pub use fret_components_ui::dropdown_menu::*;
}

pub mod command {
    pub use fret_components_ui::command::*;
    pub use fret_components_ui::command_palette::*;
    pub use fret_components_ui::command_palette_overlay::*;
}

pub use input::Input;
pub use separator::Separator;
pub use textarea::Textarea;

// Common infra re-exports so typical shadcn usage only needs one dependency.
pub use fret_components_ui::{ChromeRefinement, LayoutRefinement, Radius, Size, Space, StyledExt};
