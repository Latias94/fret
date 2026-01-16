pub(crate) const DOC_INTRO: &str = r#"
## Goals

This is an **editor-grade UI** gallery app used to:

- Validate that `fret-ui-shadcn` / `fret-ui-kit` / ecosystem components work under real composition.
- Provide a component-doc-site browsing experience (left navigation, right preview + docs).

Phase 1 intentionally uses hardcoded doc strings to validate the interaction path end-to-end.
"#;

pub(crate) const USAGE_INTRO: &str = r#"
```rust
// Native
cargo run -p fret-ui-gallery

// Web (dedicated harness)
cd apps/fret-ui-gallery-web
trunk serve --open
// open: http://127.0.0.1:8080/?page=button

// Web (via fret-demo-web host)
cd apps/fret-demo-web
trunk serve --open
// open: http://127.0.0.1:8080/?demo=ui_gallery&page=button
```
"#;

pub(crate) const DOC_LAYOUT: &str = r#"
## LayoutRefinement + stack

The gallery shell is a common editor-like layout:

- Fixed-width left navigation (scrollable)
- Right content area (scrollable)

In Fret, this is typically expressed with:

- `LayoutRefinement`: width/height/min/max/fill constraints
- `stack::{hstack,vstack}`: row/column composition & alignment
- `Theme` tokens: design system values like spacing/color/radius
"#;

pub(crate) const USAGE_LAYOUT: &str = r#"
```rust
let root = stack::hstack(
    cx,
    stack::HStackProps::default()
        .layout(LayoutRefinement::default().w_full().h_full())
        .items_stretch(),
    |_cx| vec![sidebar, content],
);
```
"#;

pub(crate) const DOC_BUTTON: &str = r#"
## Button

Validate `variant` / `size` behaviors and default styling consistency.

This layer is **visual recipes**. Interaction policies (hover intent, focus trap, etc.) should live in `fret-ui-kit` / ecosystem crates.
"#;

pub(crate) const USAGE_BUTTON: &str = r#"
```rust
use fret_ui_shadcn as shadcn;

let btn = shadcn::Button::new("Save")
    .variant(shadcn::ButtonVariant::Default)
    .into_element(cx);
```
"#;

pub(crate) const DOC_FORMS: &str = r#"
## Forms

This page validates the basic form building blocks:

- `Input` / `Textarea`
- `Checkbox` / `Switch`

These are model-bound controls: the UI is driven by `Model<T>` updates.
"#;

pub(crate) const USAGE_FORMS: &str = r#"
```rust
let email = app.models_mut().insert(String::new());
let input = shadcn::Input::new(email).a11y_label("Email");
```
"#;

pub(crate) const DOC_SELECT: &str = r#"
## Select

`Select` is an overlay-driven component (listbox in a popover-like layer).

This page validates:

- value model binding (`Model<Option<Arc<str>>>`)
- open/close model binding (`Model<bool>`)
"#;

pub(crate) const USAGE_SELECT: &str = r#"
```rust
let value = app.models_mut().insert(Some(Arc::<str>::from("apple")));
let open = app.models_mut().insert(false);

let select = shadcn::Select::new(value, open)
    .placeholder("Pick a fruit")
    .items([shadcn::SelectItem::new("apple", "Apple")]);
```
"#;

pub(crate) const DOC_COMBOBOX: &str = r#"
## Combobox

Combobox is a shadcn recipe: Popover + Command list + optional search.

This page validates:

- value model (`Model<Option<Arc<str>>>`)
- open model (`Model<bool>`)
- query model (`Model<String>`)
"#;

pub(crate) const USAGE_COMBOBOX: &str = r#"
```rust
let value = app.models_mut().insert(None::<Arc<str>>);
let open = app.models_mut().insert(false);
let query = app.models_mut().insert(String::new());

let combo = shadcn::Combobox::new(value, open)
    .query_model(query)
    .items([shadcn::ComboboxItem::new("apple", "Apple")]);
```
"#;

pub(crate) const DOC_DATE_PICKER: &str = r#"
## Date Picker

Date picker is a Popover + Calendar integration.

This page validates:

- selected date model (`Model<Option<time::Date>>`)
- month model (`Model<CalendarMonth>`)
- open model (`Model<bool>`)
"#;

pub(crate) const USAGE_DATE_PICKER: &str = r#"
```rust
let open = app.models_mut().insert(false);
let month = app
    .models_mut()
    .insert(fret_ui_headless::calendar::CalendarMonth::from_date(
        time::OffsetDateTime::now_utc().date(),
    ));
let selected = app.models_mut().insert(None::<time::Date>);

let picker = shadcn::DatePicker::new(open, month, selected);
```
"#;

pub(crate) const DOC_RESIZABLE: &str = r#"
## Resizable

Resizable panel groups are runtime-owned drag surfaces (splitter handles).

This page validates:

- fraction model (`Model<Vec<f32>>`) persistence
- nested groups (horizontal + vertical)
"#;

pub(crate) const USAGE_RESIZABLE: &str = r#"
```rust
let fractions = app.models_mut().insert(vec![0.3, 0.7]);

let group = shadcn::ResizablePanelGroup::new(fractions).entries(vec![
    shadcn::ResizablePanel::new(vec![/* ... */]).into(),
    shadcn::ResizableHandle::new().into(),
    shadcn::ResizablePanel::new(vec![/* ... */]).into(),
]);
```
"#;

pub(crate) const DOC_DATA_TABLE: &str = r#"
## DataTable

`DataTable` integrates the TanStack-aligned headless engine (ADR 0101):

- headless: sorting / filtering / selection state (`TableState`)
- UI: fixed header + virtualized body
"#;

pub(crate) const USAGE_DATA_TABLE: &str = r#"
```rust
let state = app.models_mut().insert(fret_ui_headless::table::TableState::default());

let table = shadcn::DataTable::new().into_element(
    cx,
    data,
    data_revision,
    state,
    columns,
    get_row_key,
    header_label,
    cell_at,
);
```
"#;

pub(crate) const DOC_DATA_GRID: &str = r#"
## DataGrid

`DataGrid` is a viewport-driven, virtualized rows/cols surface.

This page validates:

- large row counts without allocating all row widgets
- per-row hover/selected styling
"#;

pub(crate) const USAGE_DATA_GRID: &str = r#"
```rust
let grid = shadcn::DataGrid::new(["A", "B", "C"], 10_000).into_element(
    cx,
    rows_revision,
    cols_revision,
    row_key_at,
    row_state_at,
    cell_at,
);
```
"#;

pub(crate) const DOC_TABS: &str = r#"
## Tabs

Tabs are a roving-focus friendly navigation surface within a page.

This page validates:

- controlled selection model (`Model<Option<Arc<str>>>`)
- tab list layout and content switching
"#;

pub(crate) const USAGE_TABS: &str = r#"
```rust
let tab = app.models_mut().insert(Some(Arc::<str>::from("overview")));

let tabs = shadcn::Tabs::new(tab).items([
    shadcn::TabsItem::new("overview", "Overview", vec![cx.text("...")]),
    shadcn::TabsItem::new("details", "Details", vec![cx.text("...")]),
]);
```
"#;

pub(crate) const DOC_ACCORDION: &str = r#"
## Accordion

Accordion is a collapsible section list with keyboard navigation (roving focus).

This page validates:

- controlled open item model (`Model<Option<Arc<str>>>`)
- `collapsible` (allow close -> `None`)
"#;

pub(crate) const USAGE_ACCORDION: &str = r#"
```rust
let open_item = app.models_mut().insert(Some(Arc::<str>::from("item-1")));

let accordion = shadcn::Accordion::single(open_item)
    .collapsible(true)
    .items([
        shadcn::AccordionItem::new(
            "item-1",
            shadcn::AccordionTrigger::new(vec![cx.text("Item 1")]),
            shadcn::AccordionContent::new(vec![cx.text("...")]),
        ),
    ]);
```
"#;

pub(crate) const DOC_TABLE: &str = r#"
## Table

`Table` is a layout + styling facade (not HTML). `TableRow` is pressable for hover/selected parity.
"#;

pub(crate) const USAGE_TABLE: &str = r#"
```rust
let table = shadcn::Table::new(vec![
    shadcn::TableHeader::new(vec![/* rows */]).into_element(cx),
    shadcn::TableBody::new(vec![/* rows */]).into_element(cx),
]);
```
"#;

pub(crate) const DOC_PROGRESS: &str = r#"
## Progress

`Progress` is a purely visual indicator bound to a numeric model (default 0..=100).
"#;

pub(crate) const USAGE_PROGRESS: &str = r#"
```rust
let progress = app.models_mut().insert(35.0f32);
let bar = shadcn::Progress::new(progress);
```
"#;

pub(crate) const DOC_MENUS: &str = r#"
## Menus

This page validates two common overlay menu primitives:

- `DropdownMenu` (triggered by a button)
- `ContextMenu` (triggered by right click)
"#;

pub(crate) const USAGE_MENUS: &str = r#"
```rust
let open = app.models_mut().insert(false);
let menu = shadcn::DropdownMenu::new(open).into_element(cx, trigger, |_cx| entries);
```
"#;

pub(crate) const DOC_COMMAND: &str = r#"
## Command Palette

`CommandDialog` (cmdk) renders a searchable list of host commands.

In this gallery we register a small command surface (`File`, `View`) so cmdk has something to show.
"#;

pub(crate) const USAGE_COMMAND: &str = r#"
```rust
let open = app.models_mut().insert(false);
let query = app.models_mut().insert(String::new());
let cmdk = shadcn::CommandDialog::new_with_host_commands(cx, open, query);
```
"#;

pub(crate) const DOC_TOAST: &str = r#"
## Toast (Sonner)

Toasts are queued via `Sonner::global(app)` and rendered by a `Toaster` element (overlay layer).
"#;

pub(crate) const USAGE_TOAST: &str = r#"
```rust
let sonner = shadcn::Sonner::global(app);
sonner.toast_success_message(&mut host, window, "Done!", shadcn::ToastMessageOptions::new());
```
"#;

pub(crate) const DOC_OVERLAY: &str = r#"
## Overlay / Portal

Tooltip/HoverCard/Popover/Dialog/Sheet are rendered through overlay/portal mechanisms, outside the normal layout flow.

Goals:

- open/close state model binding
- basic policies (ESC, overlay click, focus behavior)
"#;

pub(crate) const USAGE_OVERLAY: &str = r#"
```rust
let open = app.models_mut().insert(false);

let dialog = shadcn::Dialog::new(open.clone()).into_element(
    cx,
    |cx| shadcn::Button::new("Open").toggle_model(open.clone()).into_element(cx),
    |cx| shadcn::DialogContent::new(vec![cx.text("Hello")]).into_element(cx),
);
```
"#;

pub(crate) const DOC_CARD: &str = r#"
## Card

`Card` is a composition primitive used throughout the gallery:

- header/title/description
- content body
- footer actions
"#;

pub(crate) const USAGE_CARD: &str = r#"
```rust
let card = shadcn::Card::new(vec![
    shadcn::CardHeader::new(vec![
        shadcn::CardTitle::new("Title").into_element(cx),
        shadcn::CardDescription::new("Description").into_element(cx),
    ])
    .into_element(cx),
    shadcn::CardContent::new(vec![cx.text("Body")]).into_element(cx),
])
.into_element(cx);
```
"#;

pub(crate) const DOC_BADGE: &str = r#"
## Badge

Small label component used for status, filters, and categories.
"#;

pub(crate) const USAGE_BADGE: &str = r#"
```rust
let badge = shadcn::Badge::new("Beta")
    .variant(shadcn::BadgeVariant::Secondary)
    .into_element(cx);
```
"#;

pub(crate) const DOC_AVATAR: &str = r#"
## Avatar

Avatar is a clipped, rounded container intended to host:

- `AvatarImage` (optional)
- `AvatarFallback` (initials / placeholder)
"#;

pub(crate) const USAGE_AVATAR: &str = r#"
```rust
let avatar = shadcn::Avatar::new(vec![
    shadcn::AvatarFallback::new("FR").into_element(cx),
])
.into_element(cx);
```
"#;

pub(crate) const DOC_TOOLTIP: &str = r#"
## Tooltip

Tooltip is an overlay-driven component with hover/open-delay policies.
"#;

pub(crate) const USAGE_TOOLTIP: &str = r#"
```rust
let trigger = shadcn::Button::new("Hover").into_element(cx);
let content = shadcn::TooltipContent::new(vec![
    shadcn::TooltipContent::text(cx, "Hello"),
])
.into_element(cx);

let tooltip = shadcn::Tooltip::new(trigger, content).into_element(cx);
```
"#;

pub(crate) const DOC_SLIDER: &str = r#"
## Slider

Slider is a pointer-driven control with support for:

- single value
- multi-thumb range

This page uses `Slider::new_controllable` to keep demo state local to the subtree.
"#;

pub(crate) const USAGE_SLIDER: &str = r#"
```rust
let slider = shadcn::Slider::new_controllable(cx, None, || vec![50.0])
    .range(0.0, 100.0)
    .into_element(cx);
```
"#;

pub(crate) const DOC_SKELETON: &str = r#"
## Skeleton

Skeleton validates animation scheduling and theme-driven chrome defaults.
"#;

pub(crate) const USAGE_SKELETON: &str = r#"
```rust
let skeleton = shadcn::Skeleton::new().into_element(cx);
```
"#;

pub(crate) const DOC_SCROLL_AREA: &str = r#"
## Scroll Area

Scrollable region with custom scrollbars and nested content.
"#;

pub(crate) const USAGE_SCROLL_AREA: &str = r#"
```rust
let body = stack::vstack(cx, stack::VStackProps::default(), |_cx| items);
let scroll = shadcn::ScrollArea::new(vec![body]).into_element(cx);
```
"#;
