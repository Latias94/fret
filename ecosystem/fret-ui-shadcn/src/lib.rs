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
//! environment or `UiServices`-boundary hooks, and treat `fret_ui_shadcn::raw::*` plus explicit
//! module paths as the raw escape hatch rather than the flat crate root.
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
pub mod calendar_hijri;
pub mod calendar_multiple;
pub mod calendar_range;
pub mod card;
pub mod carousel;
pub mod chart;
pub mod checkbox;
pub mod collapsible;
pub mod collapsible_primitives;
pub mod combobox;
pub mod combobox_chips;
pub mod command;
mod command_gating;
pub mod context_menu;
mod data_grid;
pub mod data_grid_canvas;
pub mod data_table;
mod data_table_controls;
mod data_table_recipes;
pub mod date_picker;
pub mod date_picker_with_presets;
pub mod date_range_picker;
pub mod dialog;
pub mod direction;
pub mod drawer;
pub mod dropdown_menu;
pub mod empty;
pub mod experimental;
pub mod extras;
pub mod field;
pub mod form;
pub mod hover_card;
pub mod input;
pub mod input_group;
pub mod input_otp;
pub mod item;
pub mod kbd;
pub mod label;
mod layout;
pub mod media_image;
mod menu_authoring;
pub mod menubar;
pub mod native_select;
pub mod navigation_menu;
mod overlay_motion;
pub mod pagination;
pub mod popover;
mod popper_arrow;
pub mod progress;
pub mod radio_group;
#[doc(hidden)]
pub mod recharts_geometry;
pub mod resizable;
mod rtl;
pub mod scroll_area;
pub mod select;
pub mod separator;
pub mod shadcn_themes;
pub mod sheet;
mod shortcut_display;
pub mod shortcut_hint;
pub mod sidebar;
pub mod skeleton;
pub mod slider;
pub mod sonner;
pub mod spinner;
#[cfg(any(feature = "state-selector", feature = "state-query"))]
pub mod state;
pub mod switch;
pub mod table;
pub mod tabs;
mod text_edit_context_menu;
pub mod text_value_model;
pub mod textarea;
pub mod toast;
pub mod toggle;
pub mod toggle_group;
pub mod tooltip;
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

/// Hidden flat compatibility exports retained while first-party code finishes migrating off the
/// flat crate root.
#[doc(hidden)]
pub use accordion::{
    Accordion, AccordionContent, AccordionItem, AccordionKind, AccordionOrientation,
    AccordionTrigger, accordion_multiple, accordion_multiple_uncontrolled, accordion_single,
    accordion_single_uncontrolled,
};
#[doc(hidden)]
pub use alert::{Alert, AlertAction, AlertDescription, AlertTitle, AlertVariant, alert};
#[doc(hidden)]
pub use alert_dialog::{
    AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogContentSize,
    AlertDialogDescription, AlertDialogFooter, AlertDialogHandle, AlertDialogHeader,
    AlertDialogMedia, AlertDialogOverlay, AlertDialogPortal, AlertDialogTitle, AlertDialogTrigger,
};
#[doc(hidden)]
pub use aspect_ratio::AspectRatio;
#[doc(hidden)]
pub use avatar::{
    Avatar, AvatarBadge, AvatarFallback, AvatarGroup, AvatarGroupCount, AvatarImage, AvatarSize,
    avatar_sized,
};
#[doc(hidden)]
pub use badge::{Badge, BadgeRender, BadgeVariant, BadgeVariants, badge, badge_variants};
#[doc(hidden)]
pub use breadcrumb::primitives::{
    Breadcrumb as BreadcrumbRoot, BreadcrumbEllipsis, BreadcrumbItem as BreadcrumbItemPart,
    BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator as BreadcrumbSeparatorPart,
};
#[doc(hidden)]
pub use breadcrumb::{Breadcrumb, BreadcrumbItem, BreadcrumbSeparator};
#[doc(hidden)]
pub use button::{
    Button, ButtonRender, ButtonSize, ButtonVariant, ButtonVariants, button_variants,
};
#[doc(hidden)]
pub use button_group::{
    ButtonGroup, ButtonGroupItem, ButtonGroupOrientation, ButtonGroupSeparator, ButtonGroupText,
    ButtonGroupVariants, button_group_variants,
};
#[doc(hidden)]
pub use calendar::{Calendar, CalendarCaptionLayout, CalendarDayButton};
#[doc(hidden)]
pub use calendar_hijri::CalendarHijri;
#[doc(hidden)]
pub use calendar_multiple::CalendarMultiple;
#[doc(hidden)]
pub use calendar_range::CalendarRange;
#[doc(hidden)]
pub use card::{
    Card, CardAction, CardContent, CardDescription, CardFooter, CardFooterDirection, CardHeader,
    CardSize, CardTitle, card, card_action, card_content, card_description,
    card_description_children, card_footer, card_header, card_sized, card_title,
};
#[doc(hidden)]
pub use carousel::{
    Carousel, CarouselAlign, CarouselApi, CarouselApiSnapshot, CarouselAutoplayApi,
    CarouselAutoplayApiSnapshot, CarouselAutoplayConfig, CarouselBreakpoint, CarouselContainScroll,
    CarouselContent, CarouselContext, CarouselEvent, CarouselEventCursor, CarouselItem,
    CarouselNext, CarouselOptions, CarouselOptionsPatch, CarouselOrientation, CarouselPlugin,
    CarouselPrevious, CarouselSlidesInViewSnapshot, CarouselSlidesToScroll,
    CarouselWheelGesturesConfig, carousel_context, use_carousel,
};
#[doc(hidden)]
pub use chart::{
    ChartConfig, ChartConfigItem, ChartContainer, ChartContext, ChartLegend, ChartLegendContent,
    ChartLegendItem, ChartLegendVerticalAlign, ChartStyle, ChartTooltip, ChartTooltipContent,
    ChartTooltipContentKind, ChartTooltipIndicator, ChartTooltipItem, chart_context, use_chart,
};
#[doc(hidden)]
pub use checkbox::{Checkbox, checkbox};
#[doc(hidden)]
pub use collapsible::{
    Collapsible, CollapsibleContent, CollapsibleTrigger, collapsible, collapsible_uncontrolled,
};
#[doc(hidden)]
pub use combobox::{
    Combobox, ComboboxChip, ComboboxChipsInput, ComboboxClear, ComboboxCollection, ComboboxContent,
    ComboboxContentPart, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem, ComboboxLabel,
    ComboboxList, ComboboxPart, ComboboxSeparator, ComboboxTrigger, ComboboxTriggerVariant,
    ComboboxValue,
};
#[doc(hidden)]
pub use combobox_chips::{ComboboxChips, ComboboxChipsPart};
#[doc(hidden)]
pub use command::{
    Command, CommandDialog, CommandEmpty, CommandEntry, CommandGroup, CommandInput, CommandItem,
    CommandList, CommandLoading, CommandPalette, CommandSeparator, CommandShortcut, command,
};
#[doc(hidden)]
pub use context_menu::{
    ContextMenu, ContextMenuCheckboxItem, ContextMenuContent, ContextMenuEntry, ContextMenuGroup,
    ContextMenuItem, ContextMenuLabel, ContextMenuPortal, ContextMenuRadioGroup,
    ContextMenuRadioItem, ContextMenuRadioItemSpec, ContextMenuSeparator, ContextMenuShortcut,
    ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger, ContextMenuTrigger,
};
#[doc(hidden)]
pub use data_grid_canvas::{DataGridCanvas, DataGridCanvasAxis};
#[doc(hidden)]
pub use fret_ui_headless::calendar::{DateRange, DateRangeSelection};
#[doc(hidden)]
pub use media_image::MediaImage;
#[doc(hidden)]
pub use text_edit_context_menu::{
    text_edit_context_menu, text_edit_context_menu_controllable, text_edit_context_menu_entries,
    text_selection_context_menu, text_selection_context_menu_controllable,
    text_selection_context_menu_entries,
};
/// Default high-performance data grid surface (canvas-rendered).
///
/// This is the "performance ceiling" option for spreadsheet-scale density:
/// prefer it when you need to scroll/render very large grids while keeping UI node count ~constant.
///
/// For business tables that need typical shadcn recipes (toolbar, column visibility, pagination),
/// prefer [`DataTable`].
///
/// For rich per-cell UI experiments, use [`experimental::DataGridElement`].
#[doc(hidden)]
pub type DataGrid = DataGridCanvas;
#[doc(hidden)]
pub use data_grid_canvas::DataGridCanvasOutput;
#[doc(hidden)]
pub use data_table::DataTable;
#[doc(hidden)]
pub use data_table_controls::{
    DataTableColumnOption, DataTableGlobalFilterInput, DataTableRowState, DataTableViewOptionItem,
    DataTableViewOptions,
};
#[doc(hidden)]
pub use data_table_recipes::DataTableToolbarResponsiveQuery;
#[doc(hidden)]
pub use data_table_recipes::{DataTableFacetedFilterOption, DataTablePagination, DataTableToolbar};
#[doc(hidden)]
pub use date_picker::DatePicker;
#[doc(hidden)]
pub use date_picker_with_presets::DatePickerWithPresets;
#[doc(hidden)]
pub use date_range_picker::DateRangePicker;
#[doc(hidden)]
pub use dialog::{
    Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader,
    DialogOverlay, DialogPortal, DialogTitle, DialogTrigger,
};
#[doc(hidden)]
pub use direction::{DirectionProvider, LayoutDirection, use_direction, with_direction_provider};
#[doc(hidden)]
pub use drawer::{
    Drawer, DrawerClose, DrawerContent, DrawerDescription, DrawerDirection, DrawerFooter,
    DrawerHeader, DrawerOverlay, DrawerPortal, DrawerSide, DrawerSnapPoint, DrawerTitle,
    DrawerTrigger,
};
#[doc(hidden)]
pub use dropdown_menu::{
    DropdownMenu, DropdownMenuAlign, DropdownMenuCheckboxItem, DropdownMenuContent,
    DropdownMenuEntry, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel, DropdownMenuPortal,
    DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuRadioItemSpec,
    DropdownMenuSeparator, DropdownMenuShortcut, DropdownMenuSide, DropdownMenuSub,
    DropdownMenuSubContent, DropdownMenuSubTrigger, DropdownMenuTrigger,
};
#[doc(hidden)]
pub use empty::{
    Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
    empty, empty_content, empty_description, empty_header, empty_media, empty_title,
};
#[doc(hidden)]
pub use field::{
    Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend,
    FieldLegendVariant, FieldOrientation, FieldSeparator, FieldSet, FieldTitle, field_group,
    field_set,
};
#[doc(hidden)]
pub use form::{
    Form, FormControl, FormDescription, FormErrorVisibility, FormField, FormItem, FormLabel,
    FormMessage, form,
};
#[doc(hidden)]
pub use fret_ui_kit::declarative::table::TableViewOutput as DataTableViewOutput;
#[doc(hidden)]
pub use hover_card::{
    HoverCard, HoverCardAlign, HoverCardAnchor, HoverCardContent, HoverCardSide, HoverCardTrigger,
};
#[doc(hidden)]
pub use input::{Input, OnInputSubmit, input};
#[doc(hidden)]
pub use input_group::{
    InputGroup, InputGroupAddon, InputGroupAddonAlign, InputGroupButton, InputGroupButtonSize,
    InputGroupInput, InputGroupPart, InputGroupText, InputGroupTextSize, InputGroupTextarea,
    input_group,
};
#[doc(hidden)]
pub use input_otp::{
    InputOTP, InputOTPGroup, InputOTPSeparator, InputOTPSlot, InputOtp, InputOtpGroup,
    InputOtpPart, InputOtpPattern, InputOtpSeparator, InputOtpSlot, input_otp,
};
#[doc(hidden)]
pub use item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader, ItemMedia,
    ItemMediaVariant, ItemRender, ItemSeparator, ItemSize, ItemTitle, ItemVariant, item_group,
    item_sized,
};
#[doc(hidden)]
pub use kbd::{Kbd, KbdGroup};
#[doc(hidden)]
pub use label::Label;
#[doc(hidden)]
pub use menubar::{
    Menubar, MenubarCheckboxItem, MenubarContent, MenubarEntry, MenubarGroup, MenubarItem,
    MenubarLabel, MenubarMenu, MenubarMenuEntries, MenubarPortal, MenubarRadioGroup,
    MenubarRadioItem, MenubarRadioItemSpec, MenubarSeparator, MenubarShortcut, MenubarSub,
    MenubarSubContent, MenubarSubTrigger, MenubarTrigger,
};
#[doc(hidden)]
pub use native_select::{
    NativeSelect, NativeSelectOptGroup, NativeSelectOption, NativeSelectSize, native_select,
};
#[doc(hidden)]
pub use navigation_menu::{
    NavigationMenu, NavigationMenuContent, NavigationMenuIndicator, NavigationMenuItem,
    NavigationMenuLink, NavigationMenuList, NavigationMenuMdBreakpointQuery, NavigationMenuRoot,
    NavigationMenuTrigger, NavigationMenuTriggerStyle, NavigationMenuViewport, navigation_menu,
    navigation_menu_list, navigation_menu_trigger_style, navigation_menu_uncontrolled,
};
#[doc(hidden)]
pub use pagination::{
    Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink,
    PaginationLinkSize, PaginationNext, PaginationPrevious, pagination, pagination_content,
    pagination_item, pagination_link,
};
#[doc(hidden)]
pub use popover::{
    Popover, PopoverAlign, PopoverAnchor, PopoverContent, PopoverDescription, PopoverHeader,
    PopoverSide, PopoverTitle, PopoverTrigger,
};
#[doc(hidden)]
pub use progress::{Progress, progress};
#[doc(hidden)]
pub use radio_group::{
    RadioGroup, RadioGroupItem, RadioGroupItemVariant, radio_group, radio_group_uncontrolled,
};
#[doc(hidden)]
pub use resizable::{
    ResizableEntry, ResizableHandle, ResizablePanel, ResizablePanelGroup, resizable_panel_group,
};
#[doc(hidden)]
pub use scroll_area::{
    ScrollArea, ScrollAreaCorner, ScrollAreaRoot, ScrollAreaScrollbar,
    ScrollAreaScrollbarOrientation, ScrollAreaViewport, ScrollBar, scroll_area,
};
#[doc(hidden)]
pub use select::{
    Select, SelectAlign, SelectContent, SelectEntry, SelectGroup, SelectItem, SelectItemIndicator,
    SelectItemText, SelectLabel, SelectScrollButtons, SelectScrollDownButton, SelectScrollUpButton,
    SelectSeparator, SelectSide, SelectTextRun, SelectTextTone, SelectTrigger,
    SelectTriggerLabelPolicy, SelectTriggerSize, SelectValue,
};
#[doc(hidden)]
pub use separator::{Separator, SeparatorOrientation, separator};
#[doc(hidden)]
pub use sheet::{
    Sheet, SheetClose, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetOverlay,
    SheetPortal, SheetSide, SheetTitle, SheetTrigger,
};
#[doc(hidden)]
pub use shortcut_hint::ShortcutHint;
#[doc(hidden)]
pub use sidebar::{
    Sidebar, SidebarCollapsible, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupAction,
    SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarInput, SidebarInset, SidebarMenu,
    SidebarMenuAction, SidebarMenuBadge, SidebarMenuButton, SidebarMenuButtonVariant,
    SidebarMenuItem, SidebarMenuSkeleton, SidebarMenuSub, SidebarMenuSubButton,
    SidebarMenuSubButtonSize, SidebarMenuSubItem, SidebarProvider, SidebarRail, SidebarSeparator,
    SidebarSide, SidebarTrigger, SidebarVariant, use_sidebar,
};
#[doc(hidden)]
pub use skeleton::Skeleton;
#[doc(hidden)]
pub use slider::{Slider, slider};
#[doc(hidden)]
pub use sonner::{
    Sonner, ToastAction, ToastIconOverride, ToastIconOverrides, ToastId, ToastMessageOptions,
    ToastOffset, ToastPosition, ToastPromise, ToastPromiseAsyncOptions, ToastPromiseHandle,
    ToastPromiseUnwrapError, ToastRequest, ToastVariant, Toaster,
};
#[doc(hidden)]
pub use spinner::Spinner;
#[doc(hidden)]
pub use switch::{Switch, SwitchSize, switch};
#[doc(hidden)]
pub use table::{
    Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow,
    table, table_body, table_caption, table_cell, table_footer, table_head, table_header,
    table_row,
};
#[doc(hidden)]
pub use tabs::{
    Tabs, TabsContent, TabsItem, TabsList, TabsListVariant, TabsListVariants, TabsRoot,
    TabsTrigger, tabs, tabs_list_variants, tabs_uncontrolled,
};
#[doc(hidden)]
pub use text_value_model::IntoTextValueModel;
#[doc(hidden)]
pub use textarea::{Textarea, textarea};
#[doc(hidden)]
pub use toggle::{
    Toggle, ToggleRoot, ToggleSize, ToggleVariant, ToggleVariants, toggle, toggle_uncontrolled,
    toggle_variants,
};
#[doc(hidden)]
pub use toggle_group::{
    ToggleGroup, ToggleGroupItem, ToggleGroupKind, toggle_group_multiple,
    toggle_group_multiple_uncontrolled, toggle_group_single, toggle_group_single_uncontrolled,
};
#[doc(hidden)]
pub use tooltip::{
    Tooltip, TooltipAlign, TooltipAnchor, TooltipContent, TooltipProvider, TooltipSide,
    TooltipTrigger,
};

/// Explicit raw/module namespace for users who intentionally need the full uncurated shadcn
/// surface.
///
/// This keeps the default discovery lane on `facade + prelude` while preserving module-oriented
/// escape hatches and the hidden flat compatibility exports behind one explicit namespace.
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
    pub use crate::with_direction_provider;
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
    pub use crate::{DirectionProvider, LayoutDirection, use_direction};
    pub use crate::{
        Select, SelectAlign, SelectContent, SelectEntry, SelectGroup, SelectItem,
        SelectItemIndicator, SelectItemText, SelectLabel, SelectScrollButtons,
        SelectScrollDownButton, SelectScrollUpButton, SelectSeparator, SelectSide, SelectTextRun,
        SelectTextTone, SelectTrigger, SelectTriggerLabelPolicy, SelectTriggerSize, SelectValue,
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
