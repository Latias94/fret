//! shadcn/ui v4-aligned component facade.
//!
//! This crate is a **naming + taxonomy surface** intended to mirror shadcn/ui (v4) so users can
//! transfer knowledge and recipes directly.
//!
//! Implementation-wise, it currently re-exports the underlying building blocks from
//! `fret-components-ui`. Over time, shadcn-specific variants/sizes/policies should converge here,
//! while `fret-components-ui` remains the reusable component infrastructure (StyledExt, tokens,
//! recipes glue, etc.).

pub mod accordion;
pub mod alert;
pub mod alert_dialog;
pub mod aspect_ratio;
pub mod avatar;
pub mod badge;
pub mod breadcrumb;
pub mod button;
pub mod button_group;
pub mod calendar;
pub mod card;
pub mod collapsible;
pub mod empty;
pub mod field;
pub mod hover_card;
pub mod input_group;
pub mod input_otp;
pub mod item;
pub mod kbd;
pub mod label;
pub mod pagination;
pub mod radio_group;
pub mod skeleton;
pub mod spinner;
pub mod table;
pub mod toggle;
pub mod toggle_group;

pub use accordion::{Accordion, AccordionContent, AccordionItem, AccordionTrigger};
pub use alert::{Alert, AlertDescription, AlertTitle, AlertVariant};
pub use alert_dialog::{AlertDialogDefaultAction, AlertDialogRequest};
pub use aspect_ratio::AspectRatio;
pub use avatar::{Avatar, AvatarFallback, AvatarImage};
pub use badge::{Badge, BadgeVariant};
pub use breadcrumb::{Breadcrumb, BreadcrumbItem};
pub use button::{Button, ButtonSize, ButtonVariant};
pub use button_group::{ButtonGroup, ButtonGroupItem, ButtonGroupOrientation};
pub use calendar::{Calendar, Date};
pub use card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};
pub use collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger};
pub use empty::Empty;
pub use field::{
    Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend,
    FieldLegendVariant, FieldOrientation, FieldSeparator, FieldSet, FieldTitle,
};
pub use hover_card::{HoverCard, HoverCardContent, HoverCardTrigger};
pub use input_group::InputGroup;
pub use input_otp::{InputOTP, InputOTPGroup, InputOTPSeparator, InputOTPSlot, InputOtpPattern};
pub use item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader, ItemMedia,
    ItemMediaVariant, ItemSeparator, ItemSize, ItemTitle, ItemVariant, item_group,
};
pub use kbd::Kbd;
pub use label::Label;
pub use pagination::{
    Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink,
    PaginationLinkSize, PaginationNext, PaginationPrevious,
};
pub use radio_group::{RadioGroup, RadioGroupItem};
pub use skeleton::Skeleton;
pub use spinner::Spinner;
pub use table::{
    Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow,
};
pub use toggle::{Toggle, ToggleSize, ToggleVariant};
pub use toggle_group::{ToggleGroup, ToggleGroupItem, ToggleGroupType};

pub mod input {
    pub use fret_components_ui::text_field::TextField as Input;
}

pub mod textarea {
    pub use fret_components_ui::text_area_field::TextAreaField as Textarea;
}

pub mod separator {
    pub use fret_components_ui::separator::Separator;
}

pub mod menubar {
    pub use fret_components_ui::AppMenuBar as Menubar;
    pub use fret_runtime::{Menu, MenuBar, MenuItem};
}

pub mod context_menu {
    pub use fret_components_ui::context_menu::*;
}

pub mod combobox {
    pub use fret_components_ui::Combobox;
}

pub mod scroll_area {
    pub use fret_components_ui::scroll_area::*;
}

pub mod progress {
    pub use fret_components_ui::progress::ProgressBar as Progress;
}

pub mod slider {
    pub use fret_components_ui::slider::Slider;
}

pub mod sonner {
    pub use fret_components_ui::sonner::*;
    pub use fret_components_ui::toast::{ToastOverlay, ToastRequest, ToastService};
}

pub mod resizable {
    pub use fret_components_ui::ResizablePanelGroup;
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

#[cfg(test)]
mod test_host;

pub mod popover {
    pub use fret_components_ui::popover::*;
}

pub mod dialog {
    pub use fret_components_ui::dialog::*;
    pub use fret_components_ui::dialog_overlay::*;
}

pub mod drawer;
pub mod sheet;

pub mod dropdown_menu {
    pub use fret_components_ui::dropdown_menu::*;
}

pub mod command {
    pub use fret_components_ui::command::*;
    pub use fret_components_ui::command_palette::*;
    pub use fret_components_ui::command_palette_overlay::*;
    pub use fret_components_ui::declarative::command_palette::command_palette_list;
}

pub mod overlays {
    pub use fret_components_ui::WindowOverlays;
}

pub use input::Input;
pub use menubar::Menubar;
pub use overlays::WindowOverlays;
pub use separator::Separator;
pub use textarea::Textarea;

pub use drawer::open_drawer;
pub use sheet::{SheetOverlay, SheetRequest, SheetService, SheetSide, SheetStyle, open_sheet};

// Common infra re-exports so typical shadcn usage only needs one dependency.
pub use fret_components_ui::{ChromeRefinement, LayoutRefinement, Radius, Size, Space, StyledExt};
