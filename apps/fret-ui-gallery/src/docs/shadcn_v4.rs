// --- shadcn/ui v4 component coverage (additional pages) ---

pub(crate) const DOC_ALERT: &str = r#"
## Alert

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/base/alert.mdx

This page aligns with the shadcn examples:

- Basic
- Destructive
- Action
- Custom Colors
- RTL

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::Alert`, `AlertVariant`
- `fret_ui_shadcn::AlertTitle`, `AlertDescription`

Notes:

- shadcn/ui includes an `AlertAction` slot; Fret currently composes actions inline (see Preview).
"#;

pub(crate) const USAGE_ALERT: &str = r#"
```rust
let alert = shadcn::Alert::new(vec![
    shadcn::icon::icon(cx, fret_icons::IconId::new_static("lucide.terminal")),
    shadcn::AlertTitle::new("Heads up!").into_element(cx),
    shadcn::AlertDescription::new("You can add components to your app.").into_element(cx),
])
.into_element(cx);
```
"#;

pub(crate) const DOC_ALERT_DIALOG: &str = r#"
## Alert Dialog

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/alert-dialog.mdx
- Radix docs: https://www.radix-ui.com/primitives/docs/components/alert-dialog

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::AlertDialog`, `AlertDialogContent`, `AlertDialogHeader`, `AlertDialogFooter`
- `fret_ui_shadcn::AlertDialogTitle`, `AlertDialogDescription`
- `fret_ui_shadcn::AlertDialogCancel`, `AlertDialogAction`

Upstream API reference:

- https://www.radix-ui.com/primitives/docs/components/alert-dialog#api-reference
"#;

pub(crate) const USAGE_ALERT_DIALOG: &str = r#"
```rust
// See the gallery preview for the recommended composition shape.
```
"#;

pub(crate) const DOC_ASPECT_RATIO: &str = r#"
## Aspect Ratio

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/aspect-ratio.mdx
- Radix docs: https://www.radix-ui.com/primitives/docs/components/aspect-ratio

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::AspectRatio`

Upstream API reference:

- https://www.radix-ui.com/primitives/docs/components/aspect-ratio#api-reference
"#;

pub(crate) const USAGE_ASPECT_RATIO: &str = r#"
```rust
let ratio = shadcn::AspectRatio::new(16.0 / 9.0, vec![/* content */]).into_element(cx);
```
"#;

pub(crate) const DOC_BREADCRUMB: &str = r#"
## Breadcrumb

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/breadcrumb.mdx

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::breadcrumb::*` (or `fret_ui_shadcn::Breadcrumb` recipes)
"#;

pub(crate) const USAGE_BREADCRUMB: &str = r#"
```rust
// See the gallery preview for `Breadcrumb`, `BreadcrumbItem`, and separators.
```
"#;

pub(crate) const DOC_BUTTON_GROUP: &str = r#"
## Button Group

Reference: https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/button-group.mdx.
"#;

pub(crate) const USAGE_BUTTON_GROUP: &str = r#"
```rust
// ButtonGroup is intended for segmented controls and grouped actions.
```
"#;

pub(crate) const DOC_CALENDAR: &str = r#"
## Calendar

Reference: https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/calendar.mdx.
"#;

pub(crate) const USAGE_CALENDAR: &str = r#"
```rust
// See the gallery preview for a minimal Calendar configuration.
```
"#;

pub(crate) const DOC_CAROUSEL: &str = r#"
## Carousel

Reference: https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/carousel.mdx.
"#;

pub(crate) const USAGE_CAROUSEL: &str = r#"
```rust
let carousel = shadcn::Carousel::new([
    cx.text("Slide 1"),
    cx.text("Slide 2"),
    cx.text("Slide 3"),
])
.item_basis_main_px(Px(260.0))
.refine_layout(LayoutRefinement::default().w_px(Px(360.0)))
.into_element(cx);

// Vertical carousel.
let vertical = shadcn::Carousel::new([
    cx.text("A"),
    cx.text("B"),
    cx.text("C"),
])
.orientation(shadcn::CarouselOrientation::Vertical)
.item_basis_main_px(Px(120.0))
.refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
.into_element(cx);
```
"#;

pub(crate) const DOC_CHART: &str = r#"
## Chart

Reference: https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/chart.mdx.
"#;

pub(crate) const USAGE_CHART: &str = r#"
```rust
// Gallery preview is a smoke stub; see `fret-ui-shadcn` tests for web parity.
```
"#;

pub(crate) const DOC_CHECKBOX: &str = r#"
## Checkbox

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/checkbox.mdx
- Radix docs: https://www.radix-ui.com/docs/primitives/components/checkbox

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::Checkbox`

Upstream API reference:

- https://www.radix-ui.com/docs/primitives/components/checkbox#api-reference
"#;

pub(crate) const USAGE_CHECKBOX: &str = r#"
```rust
let checked: Model<bool> = cx.app.models_mut().insert(false);
let checkbox = shadcn::Checkbox::new(checked)
    .a11y_label("Accept terms")
    .into_element(cx);
```
"#;

pub(crate) const DOC_COLLAPSIBLE: &str = r#"
## Collapsible

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/collapsible.mdx
- Radix docs: https://www.radix-ui.com/docs/primitives/components/collapsible

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::Collapsible`

Upstream API reference:

- https://www.radix-ui.com/docs/primitives/components/collapsible#api-reference
"#;

pub(crate) const USAGE_COLLAPSIBLE: &str = r#"
```rust
// See the gallery preview for the recommended structure.
```
"#;

pub(crate) const DOC_CONTEXT_MENU: &str = r#"
## Context Menu

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/context-menu.mdx
- Radix docs: https://www.radix-ui.com/docs/primitives/components/context-menu

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::ContextMenu`
- `fret_ui_shadcn::ContextMenuContent`, `ContextMenuItem`, `ContextMenuSub`, `ContextMenuSeparator`

Upstream API reference:

- https://www.radix-ui.com/docs/primitives/components/context-menu#api-reference
"#;

pub(crate) const USAGE_CONTEXT_MENU: &str = r#"
```rust
// See the "Menus" page (Dropdown/Context) for the full demo.
```
"#;

pub(crate) const DOC_DIALOG: &str = r#"
## Dialog

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/dialog.mdx
- Radix docs: https://www.radix-ui.com/docs/primitives/components/dialog

## API Reference

Upstream API reference:

- https://www.radix-ui.com/docs/primitives/components/dialog#api-reference
"#;

pub(crate) const USAGE_DIALOG: &str = r#"
```rust
// See the gallery preview for the recommended composition shape.
```
"#;

pub(crate) const DOC_DRAWER: &str = r#"
## Drawer

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/drawer.mdx
- Vaul docs: https://vaul.emilkowal.ski/getting-started

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::Drawer` (+ content/header/footer helpers)

Notes:

- Upstream shadcn Drawer is Vaul-based; there is no single Radix API reference page.
"#;

pub(crate) const USAGE_DRAWER: &str = r#"
```rust
// Gallery preview is a smoke stub.
```
"#;

pub(crate) const DOC_DROPDOWN_MENU: &str = r#"
## Dropdown Menu

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/dropdown-menu.mdx
- Radix docs: https://www.radix-ui.com/docs/primitives/components/dropdown-menu

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::DropdownMenu`
- `fret_ui_shadcn::DropdownMenuContent`, `DropdownMenuItem`, `DropdownMenuSub`, `DropdownMenuSeparator`

Upstream API reference:

- https://www.radix-ui.com/docs/primitives/components/dropdown-menu#api-reference
"#;

pub(crate) const USAGE_DROPDOWN_MENU: &str = r#"
```rust
// See the "Menus" page (Dropdown/Context) for the full demo.
```
"#;

pub(crate) const DOC_EMPTY: &str = r#"
## Empty

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/empty.mdx

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::Empty`
"#;

pub(crate) const USAGE_EMPTY: &str = r#"
```rust
let empty = shadcn::Empty::new([]).into_element(cx);
```
"#;

pub(crate) const DOC_FORM: &str = r#"
## Form

Fret's `Form` is a convenience layer for field composition and message visibility.

Notes:

- shadcn/ui "Form" examples are React + `react-hook-form`-centric; there is no 1:1 upstream API page in shadcn docs.
- Use `Field` when you want a lightweight, renderer-native form layout.

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::Form`, `FormField`, `FormItem`, `FormLabel`, `FormControl`, `FormMessage`
"#;

pub(crate) const USAGE_FORM: &str = r#"
```rust
// Fret favors builder-style ergonomic form composition; see `Field` + "Forms" pages.
```
"#;

pub(crate) const DOC_HOVER_CARD: &str = r#"
## Hover Card

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/hover-card.mdx
- Radix docs: https://www.radix-ui.com/docs/primitives/components/hover-card

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::HoverCard`
- `fret_ui_shadcn::HoverCardContent`, `HoverCardTrigger`

Upstream API reference:

- https://www.radix-ui.com/docs/primitives/components/hover-card#api-reference
"#;

pub(crate) const USAGE_HOVER_CARD: &str = r#"
```rust
// Gallery preview is a smoke stub.
```
"#;

pub(crate) const DOC_INPUT: &str = r#"
## Input

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/input.mdx

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::Input`
"#;

pub(crate) const USAGE_INPUT: &str = r#"
```rust
let value: Model<String> = cx.app.models_mut().insert(String::new());
let input = shadcn::Input::new(value)
    .a11y_label("Email")
    .placeholder("name@example.com")
    .into_element(cx);
```
"#;

pub(crate) const DOC_INPUT_GROUP: &str = r#"
## Input Group

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/input-group.mdx

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::InputGroup`, `InputGroupText`, `InputGroupButton`
"#;

pub(crate) const USAGE_INPUT_GROUP: &str = r#"
```rust
// Gallery preview is a smoke stub; expand as needed.
```
"#;

pub(crate) const DOC_INPUT_OTP: &str = r#"
## Input OTP

Reference: https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/input-otp.mdx.
"#;

pub(crate) const USAGE_INPUT_OTP: &str = r#"
```rust
// Gallery preview is a smoke stub; expand as needed.
```
"#;

pub(crate) const DOC_ITEM: &str = r#"
## Item

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/item.mdx

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::Item` (+ `ItemHeader`, `ItemTitle`, `ItemDescription`, `ItemActions`, ...)
"#;

pub(crate) const USAGE_ITEM: &str = r#"
```rust
// See the gallery preview for the basic Item surface.
```
"#;

pub(crate) const DOC_KBD: &str = r#"
## Kbd

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/kbd.mdx

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::Kbd`, `KbdGroup`
"#;

pub(crate) const USAGE_KBD: &str = r#"
```rust
let kbd = shadcn::Kbd::new("Ctrl+K").into_element(cx);
```
"#;

pub(crate) const DOC_LABEL: &str = r#"
## Label

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/label.mdx
- Radix docs: https://www.radix-ui.com/docs/primitives/components/label

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::Label`

Upstream API reference:

- https://www.radix-ui.com/docs/primitives/components/label#api-reference
"#;

pub(crate) const USAGE_LABEL: &str = r#"
```rust
let label = shadcn::Label::new("Email").into_element(cx);
```
"#;

pub(crate) const DOC_MENUBAR: &str = r#"
## Menubar

Reference: https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/base/menubar.mdx.
"#;

pub(crate) const USAGE_MENUBAR: &str = r#"
```rust
use fret_ui_shadcn as shadcn;

let file = shadcn::MenubarMenu::new("File").entries([
    shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("New Tab")),
    shadcn::MenubarEntry::Separator,
    shadcn::MenubarEntry::Item(shadcn::MenubarItem::new("Print...")),
]);

let bar = shadcn::Menubar::new([file]).into_element(cx);
```
"#;

pub(crate) const DOC_NATIVE_SELECT: &str = r#"
## Native Select

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/native-select.mdx

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::NativeSelect`
"#;

pub(crate) const USAGE_NATIVE_SELECT: &str = r#"
```rust
// Gallery preview is a smoke stub.
```
"#;

pub(crate) const DOC_NAVIGATION_MENU: &str = r#"
## Navigation Menu

Reference: https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/base/navigation-menu.mdx.
"#;

pub(crate) const USAGE_NAVIGATION_MENU: &str = r#"
```rust
use fret_ui_shadcn as shadcn;
use std::sync::Arc;

let value = cx.app.models_mut().insert(None::<Arc<str>>);

let item = shadcn::NavigationMenuItem::new(
    "getting_started",
    "Getting started",
    [
        shadcn::NavigationMenuLink::new(value.clone(), [cx.text("Introduction")])
            .on_click("app.open")
            .into_element(cx),
    ],
);

let menu = shadcn::NavigationMenu::new(value.clone())
    .list(shadcn::NavigationMenuList::new([
        item,
        // Items with empty content behave like the shadcn `navigationMenuTriggerStyle()` link.
        shadcn::NavigationMenuItem::new("docs", "Docs", std::iter::empty()),
    ]))
    .into_element(cx);
```
"#;

pub(crate) const DOC_PAGINATION: &str = r#"
## Pagination

Reference: https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/pagination.mdx.
"#;

pub(crate) const USAGE_PAGINATION: &str = r#"
```rust
// Gallery preview is a smoke stub.
```
"#;

pub(crate) const DOC_POPOVER: &str = r#"
## Popover

Reference: https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/popover.mdx.
"#;

pub(crate) const USAGE_POPOVER: &str = r#"
```rust
// See the gallery preview for the recommended composition shape.
```
"#;

pub(crate) const DOC_RADIO_GROUP: &str = r#"
## Radio Group

Reference: https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radio-group.mdx.
"#;

pub(crate) const USAGE_RADIO_GROUP: &str = r#"
```rust
let group = shadcn::RadioGroup::uncontrolled(Some("comfortable"))
    .a11y_label("Options")
    .item(shadcn::RadioGroupItem::new("default", "Default"))
    .item(shadcn::RadioGroupItem::new("comfortable", "Comfortable"))
    .item(shadcn::RadioGroupItem::new("compact", "Compact"))
    .into_element(cx);
```
"#;

pub(crate) const DOC_SEPARATOR: &str = r#"
## Separator

Reference: https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/separator.mdx.
"#;

pub(crate) const USAGE_SEPARATOR: &str = r#"
```rust
let sep = shadcn::Separator::new().into_element(cx);
```
"#;

pub(crate) const DOC_SHEET: &str = r#"
## Sheet

Reference: https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/sheet.mdx.
"#;

pub(crate) const USAGE_SHEET: &str = r#"
```rust
// See the gallery preview for the recommended composition shape.
```
"#;

pub(crate) const DOC_SIDEBAR: &str = r#"
## Sidebar

Reference: https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/sidebar.mdx.
"#;

pub(crate) const USAGE_SIDEBAR: &str = r#"
```rust
// Gallery preview is a smoke stub; the gallery shell itself is already sidebar-shaped.
```
"#;

pub(crate) const DOC_SONNER: &str = r#"
## Sonner

Reference: https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/sonner.mdx.
"#;

pub(crate) const USAGE_SONNER: &str = r#"
```rust
// See the "Toast" page (Sonner-backed) for the full demo.
```
"#;

pub(crate) const DOC_SPINNER: &str = r#"
## Spinner

Reference: https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/spinner.mdx.
"#;

pub(crate) const USAGE_SPINNER: &str = r#"
```rust
let spinner = shadcn::Spinner::new().into_element(cx);
```
"#;

pub(crate) const DOC_SWITCH: &str = r#"
## Switch

Reference: https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/switch.mdx.
"#;

pub(crate) const USAGE_SWITCH: &str = r#"
```rust
let checked: Model<bool> = cx.app.models_mut().insert(false);
let switch = shadcn::Switch::new(checked)
    .a11y_label("Enable feature")
    .into_element(cx);
```
"#;

pub(crate) const DOC_TEXTAREA: &str = r#"
## Textarea

Reference: https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/textarea.mdx.
"#;

pub(crate) const USAGE_TEXTAREA: &str = r#"
```rust
let value: Model<String> = cx.app.models_mut().insert(String::new());
let textarea = shadcn::Textarea::new(value).a11y_label("Message").into_element(cx);
```
"#;

pub(crate) const DOC_TOGGLE: &str = r#"
## Toggle

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/toggle.mdx
- Radix docs: https://www.radix-ui.com/docs/primitives/components/toggle

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::Toggle`

Upstream API reference:

- https://www.radix-ui.com/docs/primitives/components/toggle#api-reference
"#;

pub(crate) const USAGE_TOGGLE: &str = r#"
```rust
// See the gallery preview for a minimal Toggle configuration.
```
"#;

pub(crate) const DOC_TOGGLE_GROUP: &str = r#"
## Toggle Group

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/toggle-group.mdx
- Radix docs: https://www.radix-ui.com/docs/primitives/components/toggle-group

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::ToggleGroup`

Upstream API reference:

- https://www.radix-ui.com/docs/primitives/components/toggle-group#api-reference
"#;

pub(crate) const USAGE_TOGGLE_GROUP: &str = r#"
```rust
// See the gallery preview for the recommended ToggleGroup structure.
```
"#;

pub(crate) const DOC_TYPOGRAPHY: &str = r#"
## Typography

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/typography.mdx

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::typography::*` helpers
"#;

pub(crate) const USAGE_TYPOGRAPHY: &str = r#"
```rust
let h1 = shadcn::typography::h1(cx, "The Joke Tax Chronicles");
```
"#;
