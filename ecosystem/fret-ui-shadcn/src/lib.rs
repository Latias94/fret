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
//! use fret_ui_shadcn::prelude::*;
//! ```
//!
//! ## Feature flags
//!
//! - `app-integration`: helpers for installing the default shadcn theme into `fret_app::App` and
//!   syncing light/dark from `WindowMetricsService`.
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
pub mod combobox;
pub mod combobox_chips;
pub mod combobox_data;
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
pub mod textarea;
pub mod toast;
pub mod toggle;
pub mod toggle_group;
pub mod tooltip;
pub mod typography;

#[cfg(feature = "app-integration")]
pub mod app_integration;

mod surface_slot;
mod test_id;
mod theme_variants;
mod ui_builder_ext;
mod ui_ext;

#[cfg(test)]
mod test_support;

pub use accordion::{
    Accordion, AccordionContent, AccordionItem, AccordionKind, AccordionOrientation,
    AccordionTrigger, accordion_multiple, accordion_multiple_uncontrolled, accordion_single,
    accordion_single_uncontrolled,
};
pub use alert::{Alert, AlertAction, AlertDescription, AlertTitle, AlertVariant};
pub use alert_dialog::{
    AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogContentSize,
    AlertDialogDescription, AlertDialogFooter, AlertDialogHeader, AlertDialogMedia,
    AlertDialogOverlay, AlertDialogPortal, AlertDialogTitle, AlertDialogTrigger,
};
pub use aspect_ratio::AspectRatio;
pub use avatar::{
    Avatar, AvatarBadge, AvatarFallback, AvatarGroup, AvatarGroupCount, AvatarImage, AvatarSize,
    avatar_sized,
};
pub use badge::{Badge, BadgeRender, BadgeVariant, BadgeVariants, badge, badgeVariants, badge_variants};
pub use breadcrumb::primitives::{
    BreadcrumbEllipsis, BreadcrumbLink, BreadcrumbList, BreadcrumbPage,
};
pub use breadcrumb::{Breadcrumb, BreadcrumbItem, BreadcrumbSeparator};
pub use button::{
    Button, ButtonRender, ButtonSize, ButtonVariant, ButtonVariants, buttonVariants,
    button_variants,
};
pub use button_group::{
    ButtonGroup, ButtonGroupItem, ButtonGroupOrientation, ButtonGroupSeparator, ButtonGroupText,
    ButtonGroupVariants, buttonGroupVariants, button_group_variants,
};
pub use calendar::{Calendar, CalendarCaptionLayout, CalendarDayButton};
pub use calendar_hijri::CalendarHijri;
pub use calendar_multiple::CalendarMultiple;
pub use calendar_range::CalendarRange;
pub use card::{
    Card, CardAction, CardContent, CardDescription, CardFooter, CardFooterDirection, CardHeader,
    CardSize, CardTitle,
};
pub use carousel::{
    Carousel, CarouselAlign, CarouselApi, CarouselApiSnapshot, CarouselAutoplayConfig,
    CarouselBreakpoint, CarouselContainScroll, CarouselContent, CarouselContext, CarouselEvent,
    CarouselEventCursor, CarouselItem, CarouselNext, CarouselOptions, CarouselOptionsPatch,
    CarouselOrientation, CarouselPrevious, CarouselSlidesInViewSnapshot, CarouselSlidesToScroll,
    carousel_context, use_carousel, useCarousel,
};
pub use chart::{
    ChartConfig, ChartConfigItem, ChartContainer, ChartContext, ChartLegend, ChartLegendContent,
    ChartLegendItem, ChartLegendVerticalAlign, ChartStyle, ChartTooltip, ChartTooltipContent,
    ChartTooltipContentKind, ChartTooltipIndicator, ChartTooltipItem, chart_context, use_chart,
};
pub use checkbox::{Checkbox, checkbox};
pub use collapsible::{
    Collapsible, CollapsibleContent, CollapsibleTrigger, collapsible, collapsible_uncontrolled,
};
pub use combobox::{
    Combobox, ComboboxChip, ComboboxChipsInput, ComboboxCollection, ComboboxContent,
    ComboboxContentPart, ComboboxEmpty, ComboboxGroup, ComboboxInput, ComboboxItem, ComboboxLabel,
    ComboboxList, ComboboxPart, ComboboxSeparator, ComboboxTrigger, ComboboxTriggerVariant,
    ComboboxValue, combobox, combobox_option, combobox_option_group, useComboboxAnchor,
};
pub use combobox_chips::{ComboboxChips, ComboboxChipsPart};
pub use combobox_data::{ComboboxOption, ComboboxOptionGroup};
pub use command::{
    Command, CommandDialog, CommandEmpty, CommandEntry, CommandGroup, CommandInput, CommandItem,
    CommandList, CommandLoading, CommandPalette, CommandSeparator, CommandShortcut, command,
};
pub use context_menu::{
    ContextMenu, ContextMenuCheckboxItem, ContextMenuContent, ContextMenuEntry, ContextMenuGroup,
    ContextMenuItem, ContextMenuLabel, ContextMenuPortal, ContextMenuRadioGroup,
    ContextMenuRadioItem, ContextMenuRadioItemSpec, ContextMenuSeparator, ContextMenuShortcut,
    ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger, ContextMenuTrigger,
};
pub use data_grid_canvas::{DataGridCanvas, DataGridCanvasAxis};
pub use fret_ui_headless::calendar::{DateRange, DateRangeSelection};
pub use media_image::MediaImage;
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
pub type DataGrid = DataGridCanvas;
pub use data_grid_canvas::DataGridCanvasOutput;
pub use data_table::DataTable;
pub use data_table_controls::{
    DataTableColumnOption, DataTableGlobalFilterInput, DataTableRowState, DataTableViewOptionItem,
    DataTableViewOptions,
};
pub use data_table_recipes::DataTableToolbarResponsiveQuery;
pub use data_table_recipes::{DataTableFacetedFilterOption, DataTablePagination, DataTableToolbar};
pub use date_picker::DatePicker;
pub use date_picker_with_presets::DatePickerWithPresets;
pub use date_range_picker::DateRangePicker;
pub use dialog::{
    Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader,
    DialogOverlay, DialogPortal, DialogTitle, DialogTrigger,
};
pub use direction::{DirectionProvider, LayoutDirection, use_direction, useDirection};
pub use drawer::{
    Drawer, DrawerClose, DrawerContent, DrawerDescription, DrawerFooter, DrawerHeader,
    DrawerOverlay, DrawerPortal, DrawerSide, DrawerSnapPoint, DrawerTitle, DrawerTrigger, drawer,
};
pub use dropdown_menu::{
    DropdownMenu, DropdownMenuAlign, DropdownMenuCheckboxItem, DropdownMenuContent,
    DropdownMenuEntry, DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel, DropdownMenuPortal,
    DropdownMenuRadioGroup, DropdownMenuRadioItem, DropdownMenuRadioItemSpec,
    DropdownMenuSeparator, DropdownMenuShortcut, DropdownMenuSide, DropdownMenuSub,
    DropdownMenuSubContent, DropdownMenuSubTrigger, DropdownMenuTrigger,
};
pub use empty::{
    Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyMediaVariant, EmptyTitle,
};
pub use field::{
    Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend,
    FieldLegendVariant, FieldOrientation, FieldSeparator, FieldSet, FieldTitle,
};
pub use form::{
    Form, FormControl, FormDescription, FormErrorVisibility, FormField, FormItem, FormLabel,
    FormMessage, form,
};
pub use fret_ui_kit::declarative::table::TableViewOutput as DataTableViewOutput;
pub use hover_card::{
    HoverCard, HoverCardAlign, HoverCardAnchor, HoverCardContent, HoverCardSide, HoverCardTrigger,
};
pub use input::{Input, OnInputSubmit, input};
pub use input_group::{
    InputGroup, InputGroupAddon, InputGroupAddonAlign, InputGroupButton, InputGroupButtonSize,
    InputGroupInput, InputGroupPart, InputGroupText, InputGroupTextSize, InputGroupTextarea,
    input_group,
};
pub use input_otp::{
    InputOTP, InputOTPGroup, InputOTPSeparator, InputOTPSlot, InputOtp, InputOtpGroup,
    InputOtpPart, InputOtpSeparator, InputOtpSlot, input_otp,
};
pub use item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader, ItemMedia,
    ItemMediaVariant, ItemRender, ItemSeparator, ItemSize, ItemTitle, ItemVariant, item_group,
};
pub use kbd::{Kbd, KbdGroup};
pub use label::Label;
pub use menubar::{
    Menubar, MenubarCheckboxItem, MenubarContent, MenubarEntry, MenubarGroup, MenubarItem,
    MenubarLabel, MenubarMenu, MenubarMenuEntries, MenubarPortal, MenubarRadioGroup,
    MenubarRadioItem, MenubarRadioItemSpec, MenubarSeparator, MenubarShortcut, MenubarSub,
    MenubarSubContent, MenubarSubTrigger, MenubarTrigger, menubar,
};
pub use native_select::{
    NativeSelect, NativeSelectOptGroup, NativeSelectOption, NativeSelectSize, native_select,
};
pub use navigation_menu::{
    NavigationMenu, NavigationMenuContent, NavigationMenuIndicator, NavigationMenuItem,
    NavigationMenuLink, NavigationMenuList, NavigationMenuRoot, NavigationMenuTrigger,
    NavigationMenuTriggerStyle, NavigationMenuViewport, navigation_menu, navigation_menu_list,
    navigationMenuTriggerStyle, navigation_menu_trigger_style, navigation_menu_uncontrolled,
};
pub use pagination::{
    Pagination, PaginationContent, PaginationEllipsis, PaginationItem, PaginationLink,
    PaginationLinkSize, PaginationNext, PaginationPrevious,
};
pub use popover::{
    Popover, PopoverAlign, PopoverAnchor, PopoverContent, PopoverDescription, PopoverHeader,
    PopoverSide, PopoverTitle, PopoverTrigger,
};
pub use progress::{Progress, progress};
pub use radio_group::{
    RadioGroup, RadioGroupItem, RadioGroupItemVariant, radio_group, radio_group_uncontrolled,
};
pub use resizable::{
    ResizableEntry, ResizableHandle, ResizablePanel, ResizablePanelGroup, resizable_panel_group,
};
pub use scroll_area::{
    ScrollArea, ScrollAreaCorner, ScrollAreaRoot, ScrollAreaScrollbar,
    ScrollAreaScrollbarOrientation, ScrollAreaViewport, ScrollBar, scroll_area,
};
pub use select::{
    Select, SelectAlign, SelectContent, SelectEntry, SelectGroup, SelectItem, SelectItemIndicator,
    SelectItemText, SelectLabel, SelectScrollButtons, SelectScrollDownButton, SelectScrollUpButton,
    SelectSeparator, SelectSide, SelectTextRun, SelectTextTone, SelectTrigger,
    SelectTriggerLabelPolicy, SelectTriggerSize, SelectValue, select,
};
pub use separator::{Separator, SeparatorOrientation, separator};
pub use sheet::{
    Sheet, SheetClose, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetOverlay,
    SheetPortal, SheetSide, SheetTitle, SheetTrigger,
};
pub use shortcut_hint::ShortcutHint;
pub use sidebar::{
    Sidebar, SidebarCollapsible, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupAction,
    SidebarGroupContent, SidebarGroupLabel, SidebarHeader, SidebarInput, SidebarInset, SidebarMenu,
    SidebarMenuAction, SidebarMenuBadge, SidebarMenuButton, SidebarMenuButtonVariant,
    SidebarMenuItem, SidebarMenuSkeleton, SidebarMenuSub, SidebarMenuSubButton,
    SidebarMenuSubButtonSize, SidebarMenuSubItem, SidebarProvider, SidebarRail, SidebarSeparator,
    SidebarSide, SidebarTrigger, SidebarVariant, use_sidebar, useSidebar,
};
pub use skeleton::Skeleton;
pub use slider::{Slider, slider};
pub use sonner::{
    Sonner, ToastAction, ToastIconOverride, ToastIconOverrides, ToastId, ToastMessageOptions,
    ToastOffset, ToastPosition, ToastPromise, ToastPromiseAsyncOptions, ToastPromiseHandle,
    ToastPromiseUnwrapError, ToastRequest, ToastVariant, Toaster,
};
pub use spinner::Spinner;
pub use switch::{Switch, SwitchSize, switch};
pub use table::{
    Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow,
};
pub use tabs::{
    Tabs, TabsContent, TabsItem, TabsList, TabsListVariant, TabsListVariants, TabsRoot,
    TabsTrigger, tabs, tabsListVariants, tabs_list_variants, tabs_uncontrolled,
};
pub use textarea::{Textarea, textarea};
pub use toggle::{
    Toggle, ToggleRoot, ToggleSize, ToggleVariant, ToggleVariants, toggle, toggleVariants,
    toggle_uncontrolled, toggle_variants,
};
pub use toggle_group::{
    ToggleGroup, ToggleGroupItem, ToggleGroupKind, toggle_group_multiple,
    toggle_group_multiple_uncontrolled, toggle_group_single, toggle_group_single_uncontrolled,
};
pub use tooltip::{
    Tooltip, TooltipAlign, TooltipAnchor, TooltipContent, TooltipProvider, TooltipSide,
    TooltipTrigger,
};

#[cfg(feature = "app-integration")]
pub use app_integration::{
    ShadcnInstallConfig, install, install_app, install_app_with, install_app_with_theme,
    sync_theme_from_environment,
};

pub use ::fret_ui_kit::declarative::style as decl_style;
/// Re-exported “authoring glue” for app/component code.
///
/// shadcn/ui recipes assume a lightweight layout/styling vocabulary (Tailwind on the web).
/// In Fret, the closest analogue lives in `fret-ui-kit::declarative`. Re-exporting these keeps
/// the common “app + components” story down to `fret-ui-shadcn` + `fret-bootstrap`.
pub use ::fret_ui_kit::declarative::{icon, stack};
pub use ::fret_ui_kit::ui;
pub use ::fret_ui_kit::{
    ChromeRefinement, ColorRef, Corners4, Edges4, LayoutRefinement, MarginEdge, MetricRef, Radius,
    ShadowPreset, SignedMetricRef, Size, Space, StyledExt, UiExt,
};
pub use ui_builder_ext::*;

/// Common imports for application code using `fret-ui-shadcn`.
///
/// This keeps the “golden path” small: app code can typically depend on `fret-bootstrap` +
/// `fret-ui-shadcn` and `use fret_ui_shadcn::prelude::*;`.
pub mod prelude {
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
    pub use crate::{decl_style, icon, stack, ui};

    #[cfg(feature = "state-selector")]
    pub use crate::state::use_selector_badge;
    #[cfg(feature = "state-query")]
    pub use crate::state::{query_error_alert, query_status_badge};

    pub use fret_core::{AppWindowId, Px, TextOverflow, TextWrap, UiServices};
    pub use fret_icons::IconId;
    pub use fret_runtime::Model;
    pub use fret_ui::element::{AnyElement, TextProps};
    pub use fret_ui::{ElementContext, Invalidation, Theme, UiHost, UiTree};
    pub use fret_ui_kit::declarative::{CachedSubtreeExt, CachedSubtreeProps};
}
