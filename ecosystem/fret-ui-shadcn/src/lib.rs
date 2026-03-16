#![deny(deprecated)]
//! shadcn/ui v4-aligned component facade.
//!
//! This crate is a **naming + taxonomy surface** intended to mirror shadcn/ui (v4) so users can
//! transfer knowledge and recipes directly.
//!
//! Note: This crate is now declarative-only. Retained-widget authoring is intentionally not part of
//! the public component surface (see ADR 0066 / declarative-only migration).
//!
//! ## Getting started
//!
//! Recommended imports for application/component code:
//!
//! ```rust
//! use fret_ui_shadcn::{facade as shadcn, prelude::*};
//! ```
//!
//! Use `fret_ui_shadcn::app::*` for default app setup, `fret_ui_shadcn::advanced::*` for
//! environment or `UiServices`-boundary hooks, and treat `fret_ui_shadcn::raw::*` as the raw
//! escape hatch rather than the doc-hidden flat crate-root compatibility modules.
//!
//! Most thin public helper constructors now stay on the typed `IntoUiElement<H>` lane (for
//! example `badge`, `checkbox`, `input`, `textarea`, `slider`, `progress`, `switch`, `toggle`,
//! `tabs`, `accordion_single(...)`, `accordion_single_uncontrolled(...)`,
//! `accordion_multiple(...)`, `accordion_multiple_uncontrolled(...)`, `separator`,
//! `toggle_group_single(...)`, `toggle_group_single_uncontrolled(...)`,
//! `toggle_group_multiple(...)`, `toggle_group_multiple_uncontrolled(...)`, `input_group`,
//! `input_otp`, `avatar_sized(...)`, `item_sized(...)`, `item_group(...)`,
//! `scroll_area(...)`, `resizable_panel_group(...)`, `navigation_menu(...)`,
//! `navigation_menu_uncontrolled(...)`, `native_select(...)`, `command`, the `card(...)` /
//! `card_header(...)` / `card_content(...)` wrapper family, and the
//! `table(...)` / `table_header(...)` / `table_body(...)` / `table_row(...)` helper family,
//! `field_set(...)` / `field_group(...)` for grouped form authoring, and the
//! `empty(...)` / `empty_header(...)` / `empty_media(...)` / `empty_content(...)` wrapper family,
//! plus the `pagination(...)` / `pagination_content(...)` / `pagination_item(...)` /
//! `pagination_link(...)` wrappers, along with `radio_group(...)` /
//! `radio_group_uncontrolled(...)` returning a typed `RadioGroup` so fluent configuration stays
//! open until the explicit landing seam).
//! Remaining raw escape hatches are intentionally rare and explicitly documented where the
//! underlying storage still owns a concrete landed child (for example `kbd_icon(...)`), while
//! overlay/controller helpers keep their final landing seams explicit only where wrapper assembly
//! truly requires it (for example `text_edit_context_menu(...)`).
//!
//! ## Feature flags
//!
//! - `app-integration`: explicit app-surface helpers under `fret_ui_shadcn::app::{install, ...}`
//!   plus advanced hooks under `fret_ui_shadcn::advanced::{...}` for environment syncing and
//!   `UiServices`-boundary integration.
//! - `state-selector`, `state-query`: opt-in state helpers used by some recipes/demos.

mod a11y_modal;
#[doc(hidden)]
pub mod accordion;
#[doc(hidden)]
pub mod alert;
#[doc(hidden)]
pub mod alert_dialog;
#[doc(hidden)]
pub mod aspect_ratio;
#[doc(hidden)]
pub mod avatar;
#[doc(hidden)]
pub mod badge;
#[doc(hidden)]
pub mod breadcrumb;
#[doc(hidden)]
pub mod button;
#[doc(hidden)]
pub mod button_group;
#[doc(hidden)]
pub mod calendar;
#[doc(hidden)]
pub mod calendar_hijri;
#[doc(hidden)]
pub mod calendar_multiple;
#[doc(hidden)]
pub mod calendar_range;
#[doc(hidden)]
pub mod card;
#[doc(hidden)]
pub mod carousel;
#[doc(hidden)]
pub mod chart;
#[doc(hidden)]
pub mod checkbox;
#[doc(hidden)]
pub mod collapsible;
#[doc(hidden)]
pub mod collapsible_primitives;
#[doc(hidden)]
pub mod combobox;
#[doc(hidden)]
pub mod combobox_chips;
#[doc(hidden)]
pub mod command;
mod command_gating;
#[doc(hidden)]
pub mod context_menu;
mod data_grid;
#[doc(hidden)]
pub mod data_grid_canvas;
#[doc(hidden)]
pub mod data_table;
mod data_table_controls;
mod data_table_recipes;
#[doc(hidden)]
pub mod date_picker;
#[doc(hidden)]
pub mod date_picker_with_presets;
#[doc(hidden)]
pub mod date_range_picker;
#[doc(hidden)]
pub mod dialog;
#[doc(hidden)]
pub mod direction;
#[doc(hidden)]
pub mod drawer;
#[doc(hidden)]
pub mod dropdown_menu;
#[doc(hidden)]
pub mod empty;
#[doc(hidden)]
pub mod experimental;
#[doc(hidden)]
pub mod extras;
#[doc(hidden)]
pub mod field;
#[doc(hidden)]
pub mod form;
#[doc(hidden)]
pub mod hover_card;
#[doc(hidden)]
pub mod input;
#[doc(hidden)]
pub mod input_group;
#[doc(hidden)]
pub mod input_otp;
#[doc(hidden)]
pub mod item;
#[doc(hidden)]
pub mod kbd;
#[doc(hidden)]
pub mod label;
mod layout;
#[doc(hidden)]
pub mod media_image;
mod menu_authoring;
#[doc(hidden)]
pub mod menubar;
#[doc(hidden)]
pub mod native_select;
#[doc(hidden)]
pub mod navigation_menu;
mod overlay_motion;
#[doc(hidden)]
pub mod pagination;
#[doc(hidden)]
pub mod popover;
mod popper_arrow;
#[doc(hidden)]
pub mod progress;
#[doc(hidden)]
pub mod radio_group;
#[doc(hidden)]
pub mod recharts_geometry;
#[doc(hidden)]
pub mod resizable;
mod rtl;
#[doc(hidden)]
pub mod scroll_area;
#[doc(hidden)]
pub mod select;
#[doc(hidden)]
pub mod separator;
#[doc(hidden)]
pub mod shadcn_themes;
#[doc(hidden)]
pub mod sheet;
mod shortcut_display;
#[doc(hidden)]
pub mod shortcut_hint;
#[doc(hidden)]
pub mod sidebar;
#[doc(hidden)]
pub mod skeleton;
#[doc(hidden)]
pub mod slider;
#[doc(hidden)]
pub mod sonner;
#[doc(hidden)]
pub mod spinner;
#[cfg(any(feature = "state-selector", feature = "state-query"))]
#[doc(hidden)]
pub mod state;
#[doc(hidden)]
pub mod switch;
#[doc(hidden)]
pub mod table;
#[doc(hidden)]
pub mod tabs;
mod text_edit_context_menu;
#[doc(hidden)]
pub mod text_value_model;
#[doc(hidden)]
pub mod textarea;
#[doc(hidden)]
pub mod toast;
#[doc(hidden)]
pub mod toggle;
#[doc(hidden)]
pub mod toggle_group;
#[doc(hidden)]
pub mod tooltip;
#[doc(hidden)]
pub mod typography;

#[cfg(feature = "app-integration")]
pub mod advanced;

#[cfg(feature = "app-integration")]
pub mod app;

mod surface_slot;
mod test_id;
mod theme_variants;
mod ui_builder_ext;
mod ui_ext;

#[cfg(test)]
mod test_support;

#[cfg(test)]
mod surface_policy_tests;

/// Explicit raw/module namespace for users who intentionally need the full uncurated shadcn
/// surface.
///
/// This keeps the default discovery lane on `facade + prelude` while preserving explicit module
/// escape hatches for advanced or source-alignment work. Direct root component modules remain
/// doc-hidden compatibility residue rather than the preferred discovery lane.
pub mod raw {
    pub use crate::accordion;
    #[cfg(feature = "app-integration")]
    pub use crate::advanced;
    pub use crate::alert;
    pub use crate::alert_dialog;
    #[cfg(feature = "app-integration")]
    pub use crate::app;
    pub use crate::aspect_ratio;
    pub use crate::avatar;
    pub use crate::badge;
    pub use crate::breadcrumb;
    pub use crate::button;
    pub use crate::button_group;
    pub use crate::calendar;
    pub use crate::calendar_hijri;
    pub use crate::calendar_multiple;
    pub use crate::calendar_range;
    pub use crate::card;
    pub use crate::carousel;
    pub use crate::chart;
    pub use crate::checkbox;
    pub use crate::collapsible;
    pub use crate::collapsible_primitives;
    pub use crate::combobox;
    pub use crate::combobox_chips;
    pub use crate::command;
    pub use crate::context_menu;
    pub use crate::data_grid_canvas;
    pub use crate::data_table;
    pub use crate::date_picker;
    pub use crate::date_picker_with_presets;
    pub use crate::date_range_picker;
    pub use crate::dialog;
    pub use crate::direction;
    pub use crate::drawer;
    pub use crate::dropdown_menu;
    pub use crate::empty;
    pub use crate::experimental;
    pub use crate::extras;
    pub use crate::field;
    pub use crate::form;
    pub use crate::hover_card;
    pub use crate::input;
    pub use crate::input_group;
    pub use crate::input_otp;
    pub use crate::item;
    pub use crate::kbd;
    pub use crate::label;
    pub use crate::media_image;
    pub use crate::menubar;
    pub use crate::native_select;
    pub use crate::navigation_menu;
    pub use crate::pagination;
    pub use crate::popover;
    pub use crate::progress;
    pub use crate::radio_group;
    pub use crate::resizable;
    pub use crate::scroll_area;
    pub use crate::select;
    pub use crate::separator;
    pub use crate::shadcn_themes;
    pub use crate::sheet;
    pub use crate::shortcut_hint;
    pub use crate::sidebar;
    pub use crate::skeleton;
    pub use crate::slider;
    pub use crate::sonner;
    pub use crate::spinner;
    #[cfg(any(feature = "state-selector", feature = "state-query"))]
    pub use crate::state;
    pub use crate::switch;
    pub use crate::table;
    pub use crate::tabs;
    pub use crate::text_value_model;
    pub use crate::textarea;
    pub use crate::toast;
    pub use crate::toggle;
    pub use crate::toggle_group;
    pub use crate::tooltip;
    pub use crate::typography;
    pub use crate::{
        ChromeRefinement, ColorRef, Corners4, Edges4, LayoutRefinement, MarginEdge, MetricRef,
        Radius, ShadowPreset, SignedMetricRef, Size, Space, StyledExt, UiExt, decl_style, icon, ui,
    };
}

/// Curated app-facing shadcn surface for higher-level facades such as `fret::shadcn`.
///
/// This keeps the common component names directly reachable while making app setup, theme presets,
/// and fully raw escape hatches explicit submodules instead of whatever happens to exist on the
/// full crate root.
pub mod facade {
    pub use crate::accordion::{
        Accordion, AccordionContent, AccordionItem, AccordionKind, AccordionOrientation,
        AccordionTrigger, accordion_multiple, accordion_multiple_uncontrolled, accordion_single,
        accordion_single_uncontrolled,
    };
    pub use crate::alert::{Alert, AlertAction, AlertDescription, AlertTitle, AlertVariant, alert};
    pub use crate::alert_dialog::{
        AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent,
        AlertDialogContentSize, AlertDialogDescription, AlertDialogFooter, AlertDialogHandle,
        AlertDialogHeader, AlertDialogMedia, AlertDialogOverlay, AlertDialogPortal,
        AlertDialogTitle, AlertDialogTrigger,
    };
    pub use crate::aspect_ratio::AspectRatio;
    pub use crate::avatar::{
        Avatar, AvatarBadge, AvatarFallback, AvatarGroup, AvatarGroupCount, AvatarImage,
        AvatarSize, avatar_sized,
    };
    pub use crate::badge::{
        Badge, BadgeRender, BadgeVariant, BadgeVariants, badge, badge_variants,
    };
    pub use crate::breadcrumb::primitives::{
        Breadcrumb as BreadcrumbRoot, BreadcrumbEllipsis, BreadcrumbItem as BreadcrumbItemPart,
        BreadcrumbLink, BreadcrumbList, BreadcrumbPage,
        BreadcrumbSeparator as BreadcrumbSeparatorPart,
    };
    pub use crate::breadcrumb::{Breadcrumb, BreadcrumbItem, BreadcrumbSeparator};
    pub use crate::button::{
        Button, ButtonRender, ButtonSize, ButtonVariant, ButtonVariants, button_variants,
    };
    pub use crate::button_group::{
        ButtonGroup, ButtonGroupItem, ButtonGroupOrientation, ButtonGroupSeparator,
        ButtonGroupText, ButtonGroupVariants, button_group_variants,
    };
    pub use crate::calendar::{Calendar, CalendarCaptionLayout, CalendarDayButton};
    pub use crate::calendar_hijri::CalendarHijri;
    pub use crate::calendar_multiple::CalendarMultiple;
    pub use crate::calendar_range::CalendarRange;
    pub use crate::card::{
        Card, CardAction, CardContent, CardDescription, CardFooter, CardFooterDirection,
        CardHeader, CardSize, CardTitle, card, card_action, card_content, card_description,
        card_description_children, card_footer, card_header, card_sized, card_title,
    };
    pub use crate::carousel::{
        Carousel, CarouselAlign, CarouselApi, CarouselApiSnapshot, CarouselAutoplayApi,
        CarouselAutoplayApiSnapshot, CarouselAutoplayConfig, CarouselBreakpoint,
        CarouselContainScroll, CarouselContent, CarouselContext, CarouselEvent,
        CarouselEventCursor, CarouselItem, CarouselNext, CarouselOptions, CarouselOptionsPatch,
        CarouselOrientation, CarouselPlugin, CarouselPrevious, CarouselSlidesInViewSnapshot,
        CarouselSlidesToScroll, CarouselWheelGesturesConfig, carousel_context, use_carousel,
    };
    pub use crate::chart::{
        ChartConfig, ChartConfigItem, ChartContainer, ChartContext, ChartLegend,
        ChartLegendContent, ChartLegendItem, ChartLegendVerticalAlign, ChartStyle, ChartTooltip,
        ChartTooltipContent, ChartTooltipContentKind, ChartTooltipIndicator, ChartTooltipItem,
        chart_context, use_chart,
    };
    pub use crate::checkbox::{Checkbox, checkbox};
    pub use crate::collapsible::{
        Collapsible, CollapsibleContent, CollapsibleTrigger, collapsible, collapsible_uncontrolled,
    };
    pub use crate::combobox::{
        Combobox, ComboboxChip, ComboboxChipsInput, ComboboxClear, ComboboxCollection,
        ComboboxContent, ComboboxContentPart, ComboboxEmpty, ComboboxGroup, ComboboxInput,
        ComboboxItem, ComboboxLabel, ComboboxList, ComboboxPart, ComboboxSeparator,
        ComboboxTrigger, ComboboxTriggerVariant, ComboboxValue,
    };
    pub use crate::combobox_chips::{ComboboxChips, ComboboxChipsPart};
    pub use crate::command::{
        Command, CommandDialog, CommandEmpty, CommandEntry, CommandGroup, CommandInput,
        CommandItem, CommandList, CommandLoading, CommandPalette, CommandSeparator,
        CommandShortcut, command,
    };
    pub use crate::context_menu::{
        ContextMenu, ContextMenuCheckboxItem, ContextMenuContent, ContextMenuEntry,
        ContextMenuGroup, ContextMenuItem, ContextMenuLabel, ContextMenuPortal,
        ContextMenuRadioGroup, ContextMenuRadioItem, ContextMenuRadioItemSpec,
        ContextMenuSeparator, ContextMenuShortcut, ContextMenuSub, ContextMenuSubContent,
        ContextMenuSubTrigger, ContextMenuTrigger,
    };
    pub use crate::data_grid_canvas::{DataGridCanvas, DataGridCanvasAxis, DataGridCanvasOutput};
    pub use crate::data_table::DataTable;
    pub use crate::data_table_controls::{
        DataTableColumnOption, DataTableGlobalFilterInput, DataTableRowState,
        DataTableViewOptionItem, DataTableViewOptions,
    };
    pub use crate::data_table_recipes::{
        DataTableFacetedFilterOption, DataTablePagination, DataTableToolbar,
        DataTableToolbarResponsiveQuery,
    };
    pub use crate::date_picker::DatePicker;
    pub use crate::date_picker_with_presets::DatePickerWithPresets;
    pub use crate::date_range_picker::DateRangePicker;
    pub use crate::dialog::{
        Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader,
        DialogOverlay, DialogPortal, DialogTitle, DialogTrigger,
    };
    pub use crate::direction::{
        DirectionProvider, LayoutDirection, use_direction, with_direction_provider,
    };
    pub use crate::drawer::{
        Drawer, DrawerClose, DrawerContent, DrawerDescription, DrawerDirection, DrawerFooter,
        DrawerHeader, DrawerOverlay, DrawerPortal, DrawerSide, DrawerSnapPoint, DrawerTitle,
        DrawerTrigger,
    };
    pub use crate::dropdown_menu::{
        DropdownMenu, DropdownMenuAlign, DropdownMenuCheckboxItem, DropdownMenuContent,
        DropdownMenuEntry, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel,
        DropdownMenuPortal, DropdownMenuRadioGroup, DropdownMenuRadioItem,
        DropdownMenuRadioItemSpec, DropdownMenuSeparator, DropdownMenuShortcut, DropdownMenuSide,
        DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger, DropdownMenuTrigger,
    };
    pub use crate::empty::{
        Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant,
        EmptyTitle, empty, empty_content, empty_description, empty_header, empty_media,
        empty_title,
    };
    pub use crate::field::{
        Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend,
        FieldLegendVariant, FieldOrientation, FieldSeparator, FieldSet, FieldTitle, field_group,
        field_set,
    };
    pub use crate::form::{
        Form, FormControl, FormDescription, FormErrorVisibility, FormField, FormItem, FormLabel,
        FormMessage, form,
    };
    pub use crate::hover_card::{
        HoverCard, HoverCardAlign, HoverCardAnchor, HoverCardContent, HoverCardSide,
        HoverCardTrigger,
    };
    pub use crate::input::{Input, OnInputSubmit, input};
    pub use crate::input_group::{
        InputGroup, InputGroupAddon, InputGroupAddonAlign, InputGroupButton, InputGroupButtonSize,
        InputGroupInput, InputGroupPart, InputGroupText, InputGroupTextSize, InputGroupTextarea,
        input_group,
    };
    pub use crate::input_otp::{
        InputOTP, InputOTPGroup, InputOTPSeparator, InputOTPSlot, InputOtp, InputOtpGroup,
        InputOtpPart, InputOtpPattern, InputOtpSeparator, InputOtpSlot, input_otp,
    };
    pub use crate::item::{
        Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader,
        ItemMedia, ItemMediaVariant, ItemRender, ItemSeparator, ItemSize, ItemTitle, ItemVariant,
        item_group, item_sized,
    };
    pub use crate::kbd::{Kbd, KbdGroup};
    pub use crate::label::Label;
    pub use crate::media_image::MediaImage;
    pub use crate::menubar::{
        Menubar, MenubarCheckboxItem, MenubarContent, MenubarEntry, MenubarGroup, MenubarItem,
        MenubarLabel, MenubarMenu, MenubarMenuEntries, MenubarPortal, MenubarRadioGroup,
        MenubarRadioItem, MenubarRadioItemSpec, MenubarSeparator, MenubarShortcut, MenubarSub,
        MenubarSubContent, MenubarSubTrigger, MenubarTrigger,
    };
    pub use crate::native_select::{
        NativeSelect, NativeSelectOptGroup, NativeSelectOption, NativeSelectSize, native_select,
    };
    pub use crate::navigation_menu::{
        NavigationMenu, NavigationMenuContent, NavigationMenuIndicator, NavigationMenuItem,
        NavigationMenuLink, NavigationMenuList, NavigationMenuMdBreakpointQuery,
        NavigationMenuRoot, NavigationMenuTrigger, NavigationMenuTriggerStyle,
        NavigationMenuViewport, navigation_menu, navigation_menu_list,
        navigation_menu_trigger_style, navigation_menu_uncontrolled,
    };
    pub use crate::pagination::{
        Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink,
        PaginationLinkSize, PaginationNext, PaginationPrevious, pagination, pagination_content,
        pagination_item, pagination_link,
    };
    pub use crate::popover::{
        Popover, PopoverAlign, PopoverAnchor, PopoverContent, PopoverDescription, PopoverHeader,
        PopoverSide, PopoverTitle, PopoverTrigger,
    };
    pub use crate::progress::{Progress, progress};
    pub use crate::radio_group::{
        RadioGroup, RadioGroupItem, RadioGroupItemVariant, radio_group, radio_group_uncontrolled,
    };
    pub use crate::resizable::{
        ResizableEntry, ResizableHandle, ResizablePanel, ResizablePanelGroup, resizable_panel_group,
    };
    pub use crate::scroll_area::{
        ScrollArea, ScrollAreaCorner, ScrollAreaRoot, ScrollAreaScrollbar,
        ScrollAreaScrollbarOrientation, ScrollAreaViewport, ScrollBar, scroll_area,
    };
    pub use crate::select::{
        Select, SelectAlign, SelectContent, SelectEntry, SelectGroup, SelectItem,
        SelectItemIndicator, SelectItemText, SelectLabel, SelectScrollButtons,
        SelectScrollDownButton, SelectScrollUpButton, SelectSeparator, SelectSide, SelectTextRun,
        SelectTextTone, SelectTrigger, SelectTriggerLabelPolicy, SelectTriggerSize, SelectValue,
    };
    pub use crate::separator::{Separator, SeparatorOrientation, separator};
    pub use crate::sheet::{
        Sheet, SheetClose, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetOverlay,
        SheetPortal, SheetSide, SheetTitle, SheetTrigger,
    };
    pub use crate::shortcut_hint::ShortcutHint;
    pub use crate::sidebar::{
        Sidebar, SidebarCollapsible, SidebarContent, SidebarFooter, SidebarGroup,
        SidebarGroupAction, SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarInput,
        SidebarInset, SidebarMenu, SidebarMenuAction, SidebarMenuBadge, SidebarMenuButton,
        SidebarMenuButtonVariant, SidebarMenuItem, SidebarMenuSkeleton, SidebarMenuSub,
        SidebarMenuSubButton, SidebarMenuSubButtonSize, SidebarMenuSubItem, SidebarProvider,
        SidebarRail, SidebarSeparator, SidebarSide, SidebarTrigger, SidebarVariant, use_sidebar,
    };
    pub use crate::skeleton::Skeleton;
    pub use crate::slider::{Slider, slider};
    pub use crate::sonner::{
        Sonner, ToastAction, ToastIconOverride, ToastIconOverrides, ToastId, ToastMessageOptions,
        ToastOffset, ToastPosition, ToastPromise, ToastPromiseAsyncOptions, ToastPromiseHandle,
        ToastPromiseUnwrapError, ToastRequest, ToastVariant, Toaster,
    };
    pub use crate::spinner::Spinner;
    pub use crate::switch::{Switch, SwitchSize, switch};
    pub use crate::table::{
        Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow,
        table, table_body, table_caption, table_cell, table_footer, table_head, table_header,
        table_row,
    };
    pub use crate::tabs::{
        Tabs, TabsContent, TabsItem, TabsList, TabsListVariant, TabsListVariants, TabsRoot,
        TabsTrigger, tabs, tabs_list_variants, tabs_uncontrolled,
    };
    pub use crate::text_edit_context_menu::{
        text_edit_context_menu, text_edit_context_menu_controllable,
        text_edit_context_menu_entries, text_selection_context_menu,
        text_selection_context_menu_controllable, text_selection_context_menu_entries,
    };
    pub use crate::text_value_model::IntoTextValueModel;
    pub use crate::textarea::{Textarea, textarea};
    pub use crate::toggle::{
        Toggle, ToggleRoot, ToggleSize, ToggleVariant, ToggleVariants, toggle, toggle_uncontrolled,
        toggle_variants,
    };
    pub use crate::toggle_group::{
        ToggleGroup, ToggleGroupItem, ToggleGroupKind, toggle_group_multiple,
        toggle_group_multiple_uncontrolled, toggle_group_single, toggle_group_single_uncontrolled,
    };
    pub use crate::tooltip::{
        Tooltip, TooltipAlign, TooltipAnchor, TooltipContent, TooltipProvider, TooltipSide,
        TooltipTrigger,
    };
    pub use fret_ui_headless::calendar::{DateRange, DateRangeSelection};
    pub use fret_ui_kit::declarative::table::TableViewOutput as DataTableViewOutput;

    /// Default high-performance data grid surface (canvas-rendered).
    ///
    /// This is the "performance ceiling" option for spreadsheet-scale density:
    /// prefer it when you need to scroll/render very large grids while keeping UI node count
    /// ~constant.
    ///
    /// For business tables that need typical shadcn recipes (toolbar, column visibility,
    /// pagination), prefer [`DataTable`].
    ///
    /// For rich per-cell UI experiments, use [`experimental::DataGridElement`].
    pub type DataGrid = DataGridCanvas;

    /// Explicit app integration helpers for shadcn defaults.
    #[cfg(feature = "app-integration")]
    pub mod app {
        pub use crate::app::{InstallConfig, install, install_with, install_with_theme};
    }

    /// Explicit built-in theme presets for app-level theme installation.
    pub mod themes {
        pub use crate::shadcn_themes::{
            ShadcnBaseColor, ShadcnColorScheme, apply_shadcn_new_york, shadcn_new_york_config,
        };
    }

    /// Fully raw escape hatch that mirrors the explicit `fret_ui_shadcn::raw::*` namespace.
    pub mod raw {
        pub use crate::raw::*;
    }
}

/// Re-exported “authoring glue” for app/component code.
///
/// shadcn/ui recipes assume a lightweight layout/styling vocabulary (Tailwind on the web).
/// In Fret, the closest analogue lives in `fret-ui-kit::declarative`. Re-exporting these keeps
/// the common “app + components” story down to `fret-ui-shadcn` + `fret-bootstrap`.
#[doc(hidden)]
pub use ::fret_ui_kit::declarative::icon;
#[doc(hidden)]
pub use ::fret_ui_kit::declarative::style as decl_style;
#[doc(hidden)]
pub use ::fret_ui_kit::ui;
#[doc(hidden)]
pub use ::fret_ui_kit::{
    ChromeRefinement, ColorRef, Corners4, Edges4, LayoutRefinement, MarginEdge, MetricRef, Radius,
    ShadowPreset, SignedMetricRef, Size, Space, StyledExt, UiExt,
};
#[doc(hidden)]
pub use ui_builder_ext::*;

/// Common imports for application code using `fret-ui-shadcn`.
///
/// This keeps the “golden path” small: app code can typically depend on `fret-bootstrap` +
/// `fret-ui-shadcn`, then import `use fret_ui_shadcn::{facade as shadcn, prelude::*};`.
pub mod prelude {
    pub use crate::direction::with_direction_provider;
    pub use crate::direction::{DirectionProvider, LayoutDirection, use_direction};
    pub use crate::facade::{
        Select, SelectAlign, SelectContent, SelectEntry, SelectGroup, SelectItem,
        SelectItemIndicator, SelectItemText, SelectLabel, SelectScrollButtons,
        SelectScrollDownButton, SelectScrollUpButton, SelectSeparator, SelectSide, SelectTextRun,
        SelectTextTone, SelectTrigger, SelectTriggerLabelPolicy, SelectTriggerSize, SelectValue,
    };
    pub use crate::{
        AlertDialogUiBuilderExt, BreadcrumbPrimitivesUiBuilderExt, CollapsibleUiBuilderExt,
        CommandDialogUiBuilderExt, ContextMenuUiBuilderExt, DataGridCanvasUiBuilderExt,
        DataGridElementUiBuilderExt, DataTableUiBuilderExt, DialogUiBuilderExt, DrawerUiBuilderExt,
        DropdownMenuUiBuilderExt, PopoverUiBuilderExt, SheetUiBuilderExt, SurfaceUiBuilderExt,
    };
    pub use crate::{
        ChromeRefinement, ColorRef, Corners4, Edges4, LayoutRefinement, MarginEdge, MetricRef,
        Radius, ShadowPreset, SignedMetricRef, Size, Space, UiExt,
    };
    pub use crate::{decl_style, icon, ui};

    #[cfg(feature = "state-selector")]
    pub use crate::state::use_selector_badge;
    #[cfg(feature = "state-query")]
    pub use crate::state::{query_error_alert, query_status_badge};

    pub use fret_core::{AppWindowId, Px, TextOverflow, TextWrap, UiServices};
    pub use fret_icons::IconId;
    pub use fret_runtime::Model;
    pub use fret_ui::element::{AnyElement, TextProps};
    pub use fret_ui::{ElementContext, Invalidation, Theme, UiHost, UiTree};
    pub use fret_ui_kit::IntoUiElement;
    pub use fret_ui_kit::declarative::{
        CachedSubtreeExt, CachedSubtreeProps, UiElementA11yExt, UiElementKeyContextExt,
        UiElementTestIdExt,
    };
}
