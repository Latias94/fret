//! shadcn/ui v4-aligned component facade.
//!
//! This crate is a **naming + taxonomy surface** intended to mirror shadcn/ui (v4) so users can
//! transfer knowledge and recipes directly.
//!
//! Note: This crate is now declarative-only. Retained-widget authoring is intentionally not part of
//! the public component surface (see ADR 0066 / declarative-only migration).

pub mod alert;
pub mod aspect_ratio;
pub mod avatar;
pub mod badge;
pub mod breadcrumb;
pub mod button;
pub mod card;
pub mod checkbox;
pub mod data_table;
pub mod empty;
pub mod field;
pub mod form;
pub mod hover_card;
pub mod input;
pub mod item;
pub mod kbd;
pub mod label;
pub mod pagination;
pub mod progress;
pub mod radio_group;
pub mod select;
pub mod sidebar;
pub mod skeleton;
pub mod spinner;
pub mod switch;
pub mod table;
pub mod tabs;
pub mod toggle;
pub mod toggle_group;

pub use alert::{Alert, AlertDescription, AlertTitle, AlertVariant};
pub use aspect_ratio::AspectRatio;
pub use avatar::{Avatar, AvatarFallback, AvatarImage};
pub use badge::{Badge, BadgeVariant};
pub use breadcrumb::{Breadcrumb, BreadcrumbItem};
pub use button::{Button, ButtonSize, ButtonVariant};
pub use card::{Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle};
pub use checkbox::{Checkbox, checkbox};
pub use data_table::{DataTable, DataTableRowState};
pub use empty::Empty;
pub use field::{
    Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, FieldLegend,
    FieldLegendVariant, FieldOrientation, FieldSeparator, FieldSet, FieldTitle,
};
pub use form::{Form, FormControl, FormDescription, FormItem, FormLabel, FormMessage, form};
pub use hover_card::{HoverCard, HoverCardAlign, HoverCardContent, HoverCardTrigger};
pub use input::{Input, input};
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
pub use progress::{Progress, progress};
pub use radio_group::{RadioGroup, RadioGroupItem, radio_group};
pub use select::{Select, SelectItem, select};
pub use sidebar::{
    Sidebar, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupLabel, SidebarHeader,
    SidebarMenu, SidebarMenuButton, SidebarMenuItem,
};
pub use skeleton::Skeleton;
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

pub use fret_components_ui::{ChromeRefinement, LayoutRefinement, Radius, Size, Space, StyledExt};
