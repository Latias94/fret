#![deny(deprecated)]
//! shadcn/ui v4-aligned component facade.
//!
//! This crate is a **naming + taxonomy surface** intended to mirror shadcn/ui (v4) so users can
//! transfer knowledge and recipes directly.
//!
//! Note: This crate is now declarative-only. Retained-widget authoring is intentionally not part of
//! the public component surface (see ADR 0066 / declarative-only migration).

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
pub mod card;
pub mod checkbox;
pub mod collapsible;
pub mod combobox;
pub mod command;
pub mod context_menu;
#[cfg(feature = "datagrid")]
pub mod data_grid;
#[cfg(feature = "datagrid")]
pub mod data_table;
pub mod dialog;
pub mod drawer;
pub mod dropdown_menu;
pub mod empty;
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
pub mod menubar;
pub mod navigation_menu;
mod overlay_motion;
pub mod pagination;
pub mod popover;
mod popper_arrow;
pub mod progress;
pub mod radio_group;
pub mod resizable;
pub mod scroll_area;
pub mod select;
pub mod separator;
pub mod shadcn_themes;
pub mod sheet;
pub mod sidebar;
pub mod skeleton;
pub mod slider;
pub mod sonner;
pub mod spinner;
pub mod switch;
pub mod table;
pub mod tabs;
pub mod textarea;
pub mod toggle;
pub mod toggle_group;
pub mod tooltip;

pub use accordion::{
    Accordion, AccordionContent, AccordionItem, AccordionKind, AccordionTrigger,
    accordion_multiple, accordion_multiple_uncontrolled, accordion_single,
    accordion_single_uncontrolled,
};
pub use alert::{Alert, AlertDescription, AlertTitle, AlertVariant};
pub use alert_dialog::{
    AlertDialog, AlertDialogAction, AlertDialogCancel, AlertDialogContent, AlertDialogDescription,
    AlertDialogFooter, AlertDialogHeader, AlertDialogTitle, AlertDialogTrigger,
};
pub use aspect_ratio::AspectRatio;
pub use avatar::{Avatar, AvatarFallback, AvatarImage};
pub use badge::{Badge, BadgeVariant};
pub use breadcrumb::{Breadcrumb, BreadcrumbItem};
pub use button::{Button, ButtonSize, ButtonVariant};
pub use button_group::{
    ButtonGroup, ButtonGroupItem, ButtonGroupKind, button_group_multiple, button_group_single,
};
pub use card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};
pub use checkbox::{Checkbox, checkbox};
pub use collapsible::{
    Collapsible, CollapsibleContent, CollapsibleTrigger, collapsible, collapsible_uncontrolled,
};
pub use combobox::{Combobox, ComboboxItem, combobox};
pub use command::{
    Command, CommandDialog, CommandEmpty, CommandEntry, CommandGroup, CommandInput, CommandItem,
    CommandList, CommandPalette, CommandSeparator, CommandShortcut, command,
};
pub use context_menu::{
    ContextMenu, ContextMenuCheckboxItem, ContextMenuEntry, ContextMenuGroup, ContextMenuItem,
    ContextMenuLabel, ContextMenuRadioGroup, ContextMenuRadioItem, ContextMenuRadioItemSpec,
    ContextMenuShortcut,
};
#[cfg(feature = "datagrid")]
pub use data_grid::{DataGrid, DataGridRowState};
#[cfg(feature = "datagrid")]
pub use data_table::{
    DataTable, DataTableColumnOption, DataTableGlobalFilterInput, DataTableRowState,
    DataTableViewOptions,
};
pub use dialog::{
    Dialog, DialogClose, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
pub use drawer::{
    Drawer, DrawerContent, DrawerDescription, DrawerFooter, DrawerHeader, DrawerSide, DrawerTitle,
    drawer,
};
pub use dropdown_menu::{
    DropdownMenu, DropdownMenuAlign, DropdownMenuCheckboxItem, DropdownMenuEntry,
    DropdownMenuGroup, DropdownMenuItem, DropdownMenuLabel, DropdownMenuRadioGroup,
    DropdownMenuRadioItem, DropdownMenuRadioItemSpec, DropdownMenuShortcut, DropdownMenuSide,
};
pub use empty::Empty;
pub use field::{
    Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend,
    FieldLegendVariant, FieldOrientation, FieldSeparator, FieldSet, FieldTitle,
};
pub use form::{Form, FormControl, FormDescription, FormItem, FormLabel, FormMessage, form};
pub use hover_card::{
    HoverCard, HoverCardAlign, HoverCardAnchor, HoverCardContent, HoverCardTrigger,
};
pub use input::{Input, input};
pub use input_group::{InputGroup, input_group};
pub use input_otp::{InputOtp, input_otp};
pub use item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemFooter, ItemGroup, ItemHeader, ItemMedia,
    ItemMediaVariant, ItemSeparator, ItemSize, ItemTitle, ItemVariant, item_group,
};
pub use kbd::Kbd;
pub use label::Label;
pub use menubar::{
    Menubar, MenubarCheckboxItem, MenubarEntry, MenubarGroup, MenubarItem, MenubarLabel,
    MenubarMenu, MenubarMenuEntries, MenubarRadioGroup, MenubarRadioItem, MenubarRadioItemSpec,
    MenubarShortcut, menubar,
};
pub use navigation_menu::{
    NavigationMenu, NavigationMenuContent, NavigationMenuIndicator, NavigationMenuItem,
    NavigationMenuLink, NavigationMenuList, NavigationMenuRoot, NavigationMenuTrigger,
    NavigationMenuViewport, navigation_menu, navigation_menu_list, navigation_menu_uncontrolled,
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
pub use radio_group::{RadioGroup, RadioGroupItem, radio_group, radio_group_uncontrolled};
pub use resizable::{
    ResizableEntry, ResizableHandle, ResizablePanel, ResizablePanelGroup, resizable_panel_group,
};
pub use scroll_area::{
    ScrollArea, ScrollAreaCorner, ScrollAreaRoot, ScrollAreaScrollbar,
    ScrollAreaScrollbarOrientation, ScrollAreaViewport, scroll_area,
};
pub use select::{
    Select, SelectAlign, SelectEntry, SelectGroup, SelectItem, SelectLabel, SelectSeparator,
    SelectSide, select,
};
pub use separator::{Separator, SeparatorOrientation, separator};
pub use sheet::{
    Sheet, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetSide, SheetTitle,
};
pub use sidebar::{
    Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupLabel, SidebarHeader,
    SidebarMenu, SidebarMenuButton, SidebarMenuItem,
};
pub use skeleton::Skeleton;
pub use slider::{Slider, slider};
pub use sonner::{
    Sonner, ToastAction, ToastId, ToastMessageOptions, ToastPosition, ToastPromise, ToastRequest,
    ToastVariant, Toaster,
};
pub use spinner::Spinner;
pub use switch::{Switch, switch};
pub use table::{
    Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow,
};
pub use tabs::{
    Tabs, TabsContent, TabsItem, TabsList, TabsRoot, TabsTrigger, tabs, tabs_uncontrolled,
};
pub use textarea::{Textarea, textarea};
pub use toggle::{Toggle, ToggleRoot, ToggleSize, ToggleVariant, toggle, toggle_uncontrolled};
pub use toggle_group::{
    ToggleGroup, ToggleGroupItem, ToggleGroupKind, toggle_group_multiple,
    toggle_group_multiple_uncontrolled, toggle_group_single, toggle_group_single_uncontrolled,
};
pub use tooltip::{
    Tooltip, TooltipAlign, TooltipAnchor, TooltipContent, TooltipProvider, TooltipSide,
    TooltipTrigger,
};

pub use fret_ui_kit::declarative::style as decl_style;
/// Re-exported “authoring glue” for app/component code.
///
/// shadcn/ui recipes assume a lightweight layout/styling vocabulary (Tailwind on the web).
/// In Fret, the closest analogue lives in `fret-ui-kit::declarative`. Re-exporting these keeps
/// the common “app + components” story down to `fret-ui-shadcn` + `fret-bootstrap`.
pub use fret_ui_kit::declarative::{icon, stack};
pub use fret_ui_kit::{
    ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Size, Space, StyledExt,
};

/// Common imports for application code using `fret-ui-shadcn`.
///
/// This keeps the “golden path” small: app code can typically depend on `fret-bootstrap` +
/// `fret-ui-shadcn` and `use fret_ui_shadcn::prelude::*;`.
pub mod prelude {
    pub use crate::{
        ChromeRefinement, ColorRef, LayoutRefinement, MetricRef, Radius, Size, Space, StyledExt,
    };
    pub use crate::{decl_style, icon, stack};

    pub use fret_core::{AppWindowId, Px, TextOverflow, TextWrap, UiServices};
    pub use fret_icons::IconId;
    pub use fret_runtime::Model;
    pub use fret_ui::element::{AnyElement, TextProps};
    pub use fret_ui::{ElementContext, Invalidation, Theme, UiHost, UiTree};
}
