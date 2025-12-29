#![deny(deprecated)]
//! shadcn/ui v4-aligned component facade.
//!
//! This crate is a **naming + taxonomy surface** intended to mirror shadcn/ui (v4) so users can
//! transfer knowledge and recipes directly.
//!
//! Note: This crate is now declarative-only. Retained-widget authoring is intentionally not part of
//! the public component surface (see ADR 0066 / declarative-only migration).

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
pub mod command;
pub mod context_menu;
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
pub mod item;
pub mod kbd;
pub mod label;
pub mod pagination;
pub mod popover;
pub mod progress;
pub mod radio_group;
pub mod scroll_area;
pub mod select;
pub mod separator;
pub mod sheet;
pub mod sidebar;
pub mod skeleton;
pub mod sonner;
pub mod spinner;
pub mod switch;
pub mod table;
pub mod tabs;
pub mod toggle;
pub mod toggle_group;
pub mod tooltip;

pub use accordion::{
    Accordion, AccordionContent, AccordionItem, AccordionKind, AccordionTrigger,
    accordion_multiple, accordion_single,
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
pub use collapsible::{Collapsible, CollapsibleContent, CollapsibleTrigger, collapsible};
pub use command::{Command, CommandInput, CommandItem, CommandList, command};
pub use context_menu::{ContextMenu, ContextMenuEntry, ContextMenuItem};
pub use data_table::{DataTable, DataTableRowState};
pub use dialog::{
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
pub use drawer::{
    Drawer, DrawerContent, DrawerDescription, DrawerFooter, DrawerHeader, DrawerSide, DrawerTitle,
    drawer,
};
pub use dropdown_menu::{
    DropdownMenu, DropdownMenuAlign, DropdownMenuEntry, DropdownMenuItem, DropdownMenuSide,
};
pub use empty::Empty;
pub use field::{
    Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend,
    FieldLegendVariant, FieldOrientation, FieldSeparator, FieldSet, FieldTitle,
};
pub use form::{Form, FormControl, FormDescription, FormItem, FormLabel, FormMessage, form};
pub use hover_card::{HoverCard, HoverCardAlign, HoverCardContent, HoverCardTrigger};
pub use input::{Input, input};
pub use input_group::{InputGroup, input_group};
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
pub use popover::{
    Popover, PopoverAlign, PopoverContent, PopoverHeader, PopoverSide, PopoverTitle, PopoverTrigger,
};
pub use progress::{Progress, progress};
pub use radio_group::{RadioGroup, RadioGroupItem, radio_group};
pub use scroll_area::{ScrollArea, scroll_area};
pub use select::{Select, SelectItem, select};
pub use separator::{Separator, SeparatorOrientation, separator};
pub use sheet::{
    Sheet, SheetContent, SheetDescription, SheetFooter, SheetHeader, SheetSide, SheetTitle,
};
pub use sidebar::{
    Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupLabel, SidebarHeader,
    SidebarMenu, SidebarMenuButton, SidebarMenuItem,
};
pub use skeleton::Skeleton;
pub use sonner::{Sonner, Toaster, ToastAction, ToastId, ToastPosition, ToastRequest, ToastVariant};
pub use spinner::Spinner;
pub use switch::{Switch, switch};
pub use table::{
    Table, TableBody, TableCaption, TableCell, TableFooter, TableHead, TableHeader, TableRow,
};
pub use tabs::{Tabs, TabsItem, tabs};
pub use toggle::{Toggle, ToggleSize, ToggleVariant, toggle};
pub use toggle_group::{
    ToggleGroup, ToggleGroupItem, ToggleGroupKind, toggle_group_multiple, toggle_group_single,
};
pub use tooltip::{Tooltip, TooltipAlign, TooltipContent, TooltipSide, TooltipTrigger};

pub use fret_components_ui::{ChromeRefinement, LayoutRefinement, Radius, Size, Space, StyledExt};
