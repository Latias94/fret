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

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/select.mdx
- Radix docs: https://www.radix-ui.com/docs/primitives/components/select

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::Select`
- `fret_ui_shadcn::SelectItem`, `SelectGroup`, `SelectLabel`, `SelectSeparator`
- `fret_ui_shadcn::select::SelectPosition` (item-aligned vs popper)

Upstream API reference:

- https://www.radix-ui.com/docs/primitives/components/select#api-reference
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

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/combobox.mdx
- Base UI docs: https://base-ui.com/react/components/combobox

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::Combobox`, `ComboboxItem`

Upstream API reference:

- https://base-ui.com/react/components/combobox#api-reference
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

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/date-picker.mdx

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::DatePicker`
- Uses `fret_ui_headless::calendar` models (`CalendarMonth`)
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

Upstream shadcn/ui docs examples:

- Demo (nested groups)
- Vertical
- Handle (`with_handle(true)` in Fret; approximated chrome)
- RTL (via a direction provider)

This page validates:

- fraction model (`Model<Vec<f32>>`) persistence
- nested groups (horizontal + vertical)
"#;

pub(crate) const USAGE_RESIZABLE: &str = r#"
```rust
let h = app.models_mut().insert(vec![0.5, 0.5]);
let v = app.models_mut().insert(vec![0.25, 0.75]);

let nested = shadcn::ResizablePanelGroup::new(v)
    .axis(fret_core::Axis::Vertical)
    .entries([
        shadcn::ResizablePanel::new([/* ... */]).into(),
        shadcn::ResizableHandle::new().with_handle(true).into(),
        shadcn::ResizablePanel::new([/* ... */]).into(),
    ]);

let group = shadcn::ResizablePanelGroup::new(h)
    .axis(fret_core::Axis::Horizontal)
    .entries([
        shadcn::ResizablePanel::new([/* ... */]).into(),
        shadcn::ResizableHandle::new().with_handle(true).into(),
        shadcn::ResizablePanel::new([nested]).into(),
    ]);
```
"#;

pub(crate) const DOC_DATA_TABLE: &str = r#"
## DataTable

`DataTable` integrates the TanStack-aligned headless engine (ADR 0100):

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

pub(crate) const DOC_SHADCN_EXTRAS: &str = r#"
## Shadcn Extras (`fret-ui-shadcn::extras`)

This page showcases shadcn-styled **blocks / recipes** that are intentionally out of scope for
shadcn/ui v4 taxonomy parity.

Design rules:

- Exposed under a module surface (`shadcn::extras::*`), not glob re-exported from the crate root.
- Should not expand the `fret-ui` runtime contract surface (ADR 0066).
"#;

pub(crate) const USAGE_SHADCN_EXTRAS: &str = r#"
```rust
use fret_ui_shadcn as shadcn;

let tags = shadcn::extras::Tags::new(["Alpha", "Beta"]).into_element(cx);
let marquee = shadcn::extras::Marquee::new(["One", "Two"]).into_element(cx);
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

pub(crate) const DOC_IMAGE_OBJECT_FIT: &str = r#"
## Image / Object Fit

This page demonstrates:

- the core `SceneOp::Image { fit }` contract (Stretch / Contain / Cover),
- the shadcn policy recipe `MediaImage`,
- a virtualized thumbnails list surface (VirtualList),
- and a small streaming-update harness (partial `ImageUpdateRgba8` writes).
"#;

pub(crate) const USAGE_IMAGE_OBJECT_FIT: &str = r#"
```rust
// Fixed-size thumbnail (default cover):
let thumb = shadcn::MediaImage::maybe(Some(image_id))
    .fit(fret_core::ViewportFit::Cover)
    .refine_layout(LayoutRefinement::default().w_px(Px(48.0)).h_px(Px(48.0)))
    .into_element(cx);

// Optional intrinsic aspect ratio wrapper (policy-owned metadata store; opt-in):
let card = shadcn::MediaImage::maybe(Some(image_id))
    .intrinsic_aspect_ratio_from_metadata(true)
    .fit(fret_core::ViewportFit::Contain)
    .refine_layout(LayoutRefinement::default().w_px(Px(240.0)))
    .into_element(cx);
```
"#;

pub(crate) const DOC_TOOLTIP: &str = r#"
## Tooltip

Tooltip is an overlay-driven component with hover/open-delay policies.

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/tooltip.mdx
- Radix docs: https://www.radix-ui.com/docs/primitives/components/tooltip

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::TooltipProvider`
- `fret_ui_shadcn::Tooltip`, `TooltipContent`

Upstream API reference:

- https://www.radix-ui.com/docs/primitives/components/tooltip#api-reference
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
- range / multiple thumbs (`step` + `min_steps_between_thumbs`)
- `orientation` (horizontal / vertical)
- direction-aware mapping (`dir` + `inverted`, Radix-aligned)
- `on_value_commit` (Radix `onValueCommit`)

Upstream shadcn/ui docs examples:

- Range
- Multiple Thumbs
- Vertical
- Controlled
- Disabled
- RTL

This page demonstrates both uncontrolled (`Slider::new_controllable`) and controlled (`Slider::new(model)`) usage.
"#;

pub(crate) const USAGE_SLIDER: &str = r#"
```rust
// Uncontrolled (state in element subtree):
let slider = shadcn::Slider::new_controllable(cx, None, || vec![33.0])
    .range(0.0, 100.0)
    .step(1.0)
    .on_value_commit(|_host, _cx, _values| {
        // Called on pointer up and keyboard commits.
    })
    .into_element(cx);

// Controlled (state in the model store):
let values = app.models_mut().insert(vec![0.3, 0.7]);
let slider = shadcn::Slider::new(values)
    .range(0.0, 1.0)
    .step(0.1)
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
let scroll = shadcn::ScrollArea::new([body]).into_element(cx);
```
"#;

pub(crate) const DOC_ICONS: &str = r#"
## Icons

Fret uses renderer-agnostic `IconId`s to decouple UI components from specific icon packs:

- UI code references semantic IDs (`ui.close`, `ui.search`, ...)
- Icon packs (e.g. Lucide) register SVG sources into the global registry
- Rendering can preload SVGs into `SvgId`s for performance
"#;

pub(crate) const USAGE_ICONS: &str = r#"
```rust
use fret_icons::ids;

let icon = icon::icon(cx, ids::ui::SEARCH);
let spinner = shadcn::Spinner::new().into_element(cx);
```
"#;

pub(crate) const DOC_FIELD: &str = r#"
## Field

`Field` is a composition helper for consistent form layout:

- label + description + error slots
- content wrapper for any control (input/select/checkbox groups)
- optional separators and grouping (`FieldSet`)

Upstream reference:

- https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/components/radix/field.mdx

## API Reference

Fret surface (shadcn ecosystem):

- `fret_ui_shadcn::Field`, `FieldSet`, `FieldGroup`
- `fret_ui_shadcn::FieldLabel`, `FieldDescription`, `FieldError`, `FieldContent`
"#;

pub(crate) const USAGE_FIELD: &str = r#"
```rust
let email = shadcn::Input::new(email_model)
    .a11y_label("Email")
    .placeholder("name@example.com")
    .into_element(cx);

let field = shadcn::Field::new(vec![
    shadcn::FieldLabel::new("Email").into_element(cx),
    shadcn::FieldDescription::new("We'll never share your email.").into_element(cx),
    shadcn::FieldContent::new(vec![email]).into_element(cx),
])
.into_element(cx);
```
"#;
