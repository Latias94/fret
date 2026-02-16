pub(crate) const DOC_MATERIAL3_BUTTON: &str = r#"
## Material 3 Button (MVP)

This page validates the first Material 3 outcome-aligned component surface:

- state layer (hover / pressed / focus) driven by Material tokens
- bounded ripple (pointer-origin) driven by motion tokens
- ADR 0220 style overrides via `ButtonStyle` (partial per-state overrides)

This is intentionally *not* a full `@material/web` parity port: it focuses on the interaction + visual outcomes within Fret's retained scene model.
"#;

pub(crate) const DOC_MATERIAL3_GALLERY: &str = r#"
## Material 3 Gallery

This page is a compact, outcomes-first surface for visually scanning Material 3 components.

Goals:
- Provide a single place to spot styling drift quickly (colors, shapes, typography).
- Make it easy to flip Standard vs Expressive outcomes while keeping the rest of the gallery stable.

Notes:
- This is not a pixel-perfect golden snapshot tool (yet). It is intended to guide refactors.
"#;

pub(crate) const USAGE_MATERIAL3_GALLERY: &str = r#"
Use the “Expressive” toggle at the top to switch the variant for this page.
"#;

pub(crate) const DOC_MATERIAL3_STATE_MATRIX: &str = r#"
## Material 3 State Matrix

This page is a **manual regression harness** for cross-component outcome consistency.

Goals:
- Validate state outcomes (hover / focus / pressed / disabled / selected) across multiple M3 components.
- Catch structural instability (flicker) and token mismatch regressions early.

Notes:
- This page is not a "golden" visual diff tool; it is a fast, interactive smoke test.
"#;

pub(crate) const USAGE_MATERIAL3_STATE_MATRIX: &str = r#"
Use the controls below to exercise:

- Hover / press / focus-visible behavior
- Disabled and selected/checked states
- Menu open/close (Esc and outside press)
"#;

pub(crate) const DOC_MATERIAL3_TOUCH_TARGETS: &str = r#"
## Material 3 Touch Targets

This page validates minimum interactive sizing outcomes (touch targets):

- pressable bounds enforce a minimum size (default: 48x48)
- visual chrome remains token-sized (usually 40x40) and is centered

Notes:
- This mirrors Compose Material3 `minimumInteractiveComponentSize()` outcomes.
- Set `md.sys.layout.minimum-touch-target.size` to `0` to disable enforcement (dense desktop mode).
- Some previews may omit the “token chrome” outline when the component does not have a distinct
  chrome size smaller than its pressable bounds.
"#;

pub(crate) const USAGE_MATERIAL3_TOUCH_TARGETS: &str = r#"
Token: `md.sys.layout.minimum-touch-target.size` (default: 48).
"#;

pub(crate) const USAGE_MATERIAL3_BUTTON: &str = r#"
```rust
use fret_ui_material3 as m3;

let filled = m3::Button::new("Filled")
    .variant(m3::ButtonVariant::Filled)
    .into_element(cx);

let text = m3::Button::new("Text")
    .variant(m3::ButtonVariant::Text)
    .into_element(cx);
```
"#;

pub(crate) const DOC_MATERIAL3_ICON_BUTTON: &str = r#"
## Material 3 Icon Button (MVP)

This page validates a second Material 3 component:

- token-driven icon color + container color (variants)
- state layer (hover / pressed / focus)
- bounded ripple (pointer-origin)
- ADR 0220 style overrides via `IconButtonStyle` (partial per-state overrides)
"#;

pub(crate) const USAGE_MATERIAL3_ICON_BUTTON: &str = r#"
```rust
use fret_icons::ids;
use fret_ui_material3 as m3;

let close = m3::IconButton::new(ids::ui::CLOSE)
    .variant(m3::IconButtonVariant::Standard)
    .a11y_label("Close")
    .into_element(cx);
```
"#;

pub(crate) const DOC_MATERIAL3_CHECKBOX: &str = r#"
## Material 3 Checkbox (MVP)

This page validates a third Material 3 component:

- token-driven sizing + colors
- state layer (hover / pressed / focus)
- bounded ripple (pointer-origin)
- ADR 0220 style overrides via `CheckboxStyle` (partial per-state overrides)

Notes:
- This is the control-only MVP (40px target, 18px box). Label-click behavior is a follow-up recipe.
"#;

pub(crate) const USAGE_MATERIAL3_CHECKBOX: &str = r#"
```rust
use fret_ui_material3 as m3;

let checked = app.models_mut().insert(false);
let cb = m3::Checkbox::new(checked)
    .a11y_label("Accept terms")
    .into_element(cx);
```
"#;

pub(crate) const DOC_MATERIAL3_SWITCH: &str = r#"
## Material 3 Switch (MVP)

This page validates a Material 3 switch surface:

- token-driven sizing + colors
- state layer (hover / pressed / focus) centered on the thumb
- bounded ripple (pointer-origin)
- ADR 0220 style overrides via `SwitchStyle` (partial per-state overrides)
"#;

pub(crate) const USAGE_MATERIAL3_SWITCH: &str = r#"
```rust
use fret_ui_material3 as m3;

let selected = app.models_mut().insert(false);
let sw = m3::Switch::new(selected)
    .a11y_label("Enable feature")
    .into_element(cx);
```
"#;

pub(crate) const DOC_MATERIAL3_RADIO: &str = r#"
## Material 3 Radio (MVP)

This page validates a Material 3 radio button surface:

- token-driven sizing + colors
- state layer (hover / pressed / focus)
- bounded ripple (pointer-origin)
- ADR 0220 style overrides via `RadioStyle` (partial per-state overrides)

This page uses the group-value binding API (`Model<Option<Arc<str>>>`) so multiple items behave like a real radio group.

This page also includes `RadioStyle` override previews for both `RadioGroup` (forwarded to items) and standalone `Radio`.
"#;

pub(crate) const USAGE_MATERIAL3_RADIO: &str = r#"
```rust
use fret_ui_material3 as m3;
use std::sync::Arc;

let value = app.models_mut().insert(None::<Arc<str>>);
let a = m3::Radio::new_value("A", value.clone()).a11y_label("A");
```
"#;

pub(crate) const DOC_MATERIAL3_BADGE: &str = r#"
## Material 3 Badge (MVP)

This page validates a Material 3 badge surface:

- token-driven sizing + colors via `md.comp.badge.*`
- dot and large (value) variants
- navigation icon placement (Material Web labs placement parity)
"#;

pub(crate) const USAGE_MATERIAL3_BADGE: &str = r#"
```rust
use fret_ui_material3 as m3;
use fret_core::Px;
use fret_ui::element::{ContainerProps, Length};

let mut anchor = ContainerProps::default();
anchor.layout.size.width = Length::Px(Px(24.0));
anchor.layout.size.height = Length::Px(Px(24.0));

let badged = m3::Badge::text("9")
    .navigation_anchor_size(Px(24.0))
    .into_element(cx, |cx| [cx.container(anchor, |_cx| [])]);
```
"#;

pub(crate) const DOC_MATERIAL3_TOP_APP_BAR: &str = r#"
## Material 3 Top App Bar (Primitives)

This page validates top app bar primitives driven by Material Web v30 tokens:

- variants: small / small-centered / medium / large
- token-driven sizing + colors via `md.comp.top-app-bar.*`
- minimal "scrolled" surface (switches to `on-scroll` container tokens)

Note: Fret currently models top app bar semantics as `Group` (core does not yet expose a dedicated
toolbar semantics role). This is tracked in the Material 3 next wave workstream.
"#;

pub(crate) const USAGE_MATERIAL3_TOP_APP_BAR: &str = r#"
```rust
use fret_ui_material3 as m3;
use fret_icons::ids;

let bar = m3::TopAppBar::new("Title")
    .variant(m3::TopAppBarVariant::Small)
    .navigation_icon(m3::TopAppBarAction::new(ids::ui::CHEVRON_RIGHT).a11y_label("Navigate"))
    .actions(vec![
        m3::TopAppBarAction::new(ids::ui::SEARCH).a11y_label("Search"),
        m3::TopAppBarAction::new(ids::ui::MORE_HORIZONTAL).a11y_label("More"),
    ])
    .scrolled(false)
    .into_element(cx);
```
"#;

pub(crate) const DOC_MATERIAL3_BOTTOM_SHEET: &str = r#"
## Material 3 Bottom Sheet (Primitives)

This page validates bottom sheet primitives driven by Material Web v30 tokens:

- token-driven container outcomes via `md.comp.sheet.bottom.*`
- drag handle sizing + opacity
- modal variant: `OverlayRequest::modal` with a scrim and focus trap/restore
- dismissal: outside press on the scrim (Escape handling is tracked separately)

Non-goals (for this MVP):

- Compose-style `SheetState`, dragging, nested scrolling, and partial expansion.
"#;

pub(crate) const USAGE_MATERIAL3_BOTTOM_SHEET: &str = r#"
```rust
use fret_ui_material3 as m3;
use fret_runtime::Model;

let open: Model<bool> = app.models_mut().insert(false);

let sheet = m3::ModalBottomSheet::new(open.clone()).into_element(
    cx,
    |cx| m3::Button::new("Open").into_element(cx),
    |cx| [m3::Button::new("Close").into_element(cx)],
);
```
"#;

pub(crate) const DOC_MATERIAL3_DATE_PICKER: &str = r#"
## Material 3 Date Picker (Primitives)

This page validates date picker primitives driven by Material Web v30 tokens:

- token-driven container + day cell outcomes via `md.comp.date-picker.{docked,modal}.*`
- docked variant: a non-overlay surface suitable for scaffold-like layouts
- modal variant: `OverlayRequest::modal` with a scrim and focus trap/restore
- selection is staged while open and applied on confirm

Non-goals (for this MVP):

- range selection, year selection, and input mode.
"#;

pub(crate) const USAGE_MATERIAL3_DATE_PICKER: &str = r#"
```rust
use fret_ui_material3 as m3;
use fret_ui_headless::calendar::CalendarMonth;
use time::{Date, Month};

let open = app.models_mut().insert(false);
let month = app.models_mut().insert(CalendarMonth::new(2026, Month::January));
let selected = app.models_mut().insert(None::<Date>);

let dialog = m3::DatePickerDialog::new(open, month.clone(), selected.clone())
    .into_element(cx, |cx| m3::Button::new("Open").into_element(cx));

let docked = m3::DockedDatePicker::new(month, selected).into_element(cx);
```
"#;

pub(crate) const DOC_MATERIAL3_TIME_PICKER: &str = r#"
## Material 3 Time Picker (Primitives)

This page validates time picker primitives driven by Material Web v30 tokens:

- token-driven outcomes via `md.comp.time-picker.*`
- docked variant: a non-overlay surface suitable for scaffold-like layouts
- modal variant: `OverlayRequest::modal` with a scrim and focus trap/restore
- selection is staged while open and applied on confirm

Non-goals (for this MVP):

- drag/gesture dial selection and input mode toggle.
"#;

pub(crate) const USAGE_MATERIAL3_TIME_PICKER: &str = r#"
```rust
use fret_ui_material3 as m3;
use time::Time;

let open = app.models_mut().insert(false);
let selected = app.models_mut().insert(Time::from_hms(9, 41, 0).unwrap());

let dialog = m3::TimePickerDialog::new(open, selected.clone())
    .into_element(cx, |cx| m3::Button::new("Open").into_element(cx));

let docked = m3::DockedTimePicker::new(selected).into_element(cx);
```
"#;

pub(crate) const DOC_MATERIAL3_SEGMENTED_BUTTON: &str = r#"
## Material 3 Segmented Button (MVP)

This page validates an outlined segmented button surface:

- token-driven sizing + colors via `md.comp.outlined-segmented-button.*`
- single-select and multi-select groups
- roving focus (Arrow keys + Home/End; skip disabled)
- state layer (hover / pressed / focus) and bounded ripple
"#;

pub(crate) const USAGE_MATERIAL3_SEGMENTED_BUTTON: &str = r#"
```rust
use fret_ui_material3 as m3;
use std::collections::BTreeSet;
use std::sync::Arc;

let single = app.models_mut().insert(Arc::<str>::from("alpha"));
let multi: BTreeSet<Arc<str>> = [Arc::<str>::from("alpha")].into_iter().collect();
let multi = app.models_mut().insert(multi);

let set = m3::SegmentedButtonSet::single(single)
    .items(vec![
        m3::SegmentedButtonItem::new("alpha", "Alpha"),
        m3::SegmentedButtonItem::new("beta", "Beta"),
    ])
    .a11y_label("Options")
    .into_element(cx);
```
"#;

pub(crate) const DOC_MATERIAL3_SELECT: &str = r#"
## Material 3 Select (MVP)

This page validates a Material 3 select surface:

- token-driven trigger outcomes via `md.comp.{outlined,filled}-select.*`
- listbox overlay anchored to the trigger (Escape / outside press dismissal)
- ADR 0220 style overrides via `SelectStyle` (partial per-state overrides)
"#;

pub(crate) const USAGE_MATERIAL3_SELECT: &str = r#"
```rust
use fret_ui_material3 as m3;
use std::sync::Arc;

let model = app.models_mut().insert(None::<Arc<str>>);
let items = [
    m3::SelectItem::new("a", "Option A"),
    m3::SelectItem::new("b", "Option B"),
];

let select = m3::Select::new(model)
    .placeholder("Pick one")
    .items(items)
    .into_element(cx);
```
"#;

pub(crate) const DOC_MATERIAL3_AUTOCOMPLETE: &str = r#"
## Material 3 Autocomplete (MVP)

This page validates a Material 3 autocomplete surface:

- token-driven input + menu outcomes via `md.comp.{outlined,filled}-autocomplete.*`
- combobox semantics (ADR 0073): `active_descendant` + `controls` ↔ `labelled_by`
- non-modal popover menu that stays interactive while typing (click-through)
- composition surface: `ExposedDropdown` (searchable select policy over `Autocomplete`)
"#;

pub(crate) const USAGE_MATERIAL3_AUTOCOMPLETE: &str = r#"
```rust
use fret_ui_material3 as m3;
use std::sync::Arc;

let query = app.models_mut().insert(String::new());
let selected_value = app.models_mut().insert(None::<Arc<str>>);
let items = [
    m3::AutocompleteItem::new("alpha", "Alpha"),
    m3::AutocompleteItem::new("beta", "Beta"),
];

let ac = m3::Autocomplete::new(query)
    .selected_value(selected_value)
    .label("Search")
    .placeholder("Type to filter")
    .items(items)
    .into_element(cx);

// Composition: searchable select.
let committed = app
    .models_mut()
    .insert(Some(Arc::<str>::from("beta")) as Option<Arc<str>>);
let exposed_query = app.models_mut().insert(String::new());
let exposed = m3::ExposedDropdown::new(committed)
    .query(exposed_query)
    .label("Searchable select")
    .items(items)
    .into_element(cx);
```
"#;

pub(crate) const DOC_MATERIAL3_TEXT_FIELD: &str = r#"
## Material 3 Text Field (MVP)

This page validates Material 3 text field variants:

- outlined: token-driven outline colors + widths (hover/focus/error/disabled)
- filled: token-driven filled container + active indicator + hover state layer
- label + placeholder outcomes (best-effort)
- outlined: animated label float + an outline "notch" patch (best-effort)
- ADR 0220 style overrides via `TextFieldStyle` (partial per-state overrides)

This is built on top of `fret-ui`'s `TextInput` mechanism widget (caret/selection/IME).
"#;

pub(crate) const USAGE_MATERIAL3_TEXT_FIELD: &str = r#"
```rust
use fret_ui_material3 as m3;

let model = app.models_mut().insert(String::new());
let tf = m3::TextField::new(model)
    .variant(m3::TextFieldVariant::Filled)
    .label("Name")
    .placeholder("Type here")
    .into_element(cx);
```
"#;

pub(crate) const DOC_MATERIAL3_TABS: &str = r#"
## Material 3 Tabs (MVP)

This page validates a Material 3 primary navigation tabs surface:

- roving focus + automatic activation (selection follows focus)
- state layer (hover / pressed / focus)
- bounded ripple (pointer-origin)
- ADR 0220 style overrides via `TabsStyle` (partial per-state overrides)
"#;

pub(crate) const USAGE_MATERIAL3_TABS: &str = r#"
```rust
use fret_ui_material3 as m3;
use std::sync::Arc;

let value = app.models_mut().insert(Arc::<str>::from("overview"));
let tabs = m3::Tabs::new(value)
    .a11y_label("Demo tabs")
    .items(vec![
        m3::TabItem::new("overview", "Overview"),
        m3::TabItem::new("settings", "Settings"),
    ])
    .into_element(cx);
```
"#;

pub(crate) const DOC_MATERIAL3_NAVIGATION_BAR: &str = r#"
## Material 3 Navigation Bar (MVP)

This page validates a Material 3 bottom navigation bar surface:

- roving focus + automatic activation (selection follows focus)
- state layer (hover / pressed / focus)
- bounded ripple (pointer-origin)
- active indicator that tracks the selected icon slot
"#;

pub(crate) const USAGE_MATERIAL3_NAVIGATION_BAR: &str = r#"
```rust
use fret_icons::ids;
use fret_ui_material3 as m3;
use std::sync::Arc;

let value = app.models_mut().insert(Arc::<str>::from("search"));
let bar = m3::NavigationBar::new(value)
    .a11y_label("Demo navigation bar")
    .items(vec![
        m3::NavigationBarItem::new("search", "Search", ids::ui::SEARCH),
        m3::NavigationBarItem::new("settings", "Settings", ids::ui::SETTINGS),
    ])
    .into_element(cx);
```
"#;

pub(crate) const DOC_MATERIAL3_NAVIGATION_RAIL: &str = r#"
## Material 3 Navigation Rail (MVP)

This page validates a Material 3 navigation rail surface:

- roving focus + automatic activation (selection follows focus)
- state layer (hover / pressed / focus) bounded to the indicator pill
- bounded ripple (pointer-origin) bounded to the indicator pill
- active indicator that tracks the selected icon slot
"#;

pub(crate) const USAGE_MATERIAL3_NAVIGATION_RAIL: &str = r#"
```rust
use fret_icons::ids;
use fret_ui_material3 as m3;
use std::sync::Arc;

let value = app.models_mut().insert(Arc::<str>::from("search"));
let rail = m3::NavigationRail::new(value)
    .a11y_label("Demo navigation rail")
    .items(vec![
        m3::NavigationRailItem::new("search", "Search", ids::ui::SEARCH),
        m3::NavigationRailItem::new("settings", "Settings", ids::ui::SETTINGS),
        m3::NavigationRailItem::new("play", "Play", ids::ui::PLAY),
    ])
    .into_element(cx);
```
"#;

pub(crate) const DOC_MATERIAL3_NAVIGATION_DRAWER: &str = r#"
## Material 3 Navigation Drawer (MVP)

This page validates a Material 3 navigation drawer surface:

- roving focus + automatic activation (selection follows focus)
- state layer (hover / pressed / focus)
- bounded ripple (pointer-origin)
- selected item pill uses `active-indicator.color` (Compose parity)
"#;

pub(crate) const USAGE_MATERIAL3_NAVIGATION_DRAWER: &str = r#"
```rust
use fret_icons::ids;
use fret_ui_material3 as m3;
use std::sync::Arc;

let value = app.models_mut().insert(Arc::<str>::from("search"));
let drawer = m3::NavigationDrawer::new(value)
    .a11y_label("Demo navigation drawer")
    .items(vec![
        m3::NavigationDrawerItem::new("search", "Search", ids::ui::SEARCH),
        m3::NavigationDrawerItem::new("settings", "Settings", ids::ui::SETTINGS),
        m3::NavigationDrawerItem::new("play", "Play", ids::ui::PLAY),
    ])
    .into_element(cx);
```
"#;

pub(crate) const DOC_MATERIAL3_MODAL_NAVIGATION_DRAWER: &str = r#"
## Material 3 Modal Navigation Drawer (MVP)

This page validates a Material 3 **modal** navigation drawer surface:

- modal barrier (no click-through)
- scrim: Neutral-Variant10 @ 50% (token-driven override)
- slide-in motion driven by theme easing tokens
- focus trap while open + focus restore on close
"#;

pub(crate) const USAGE_MATERIAL3_MODAL_NAVIGATION_DRAWER: &str = r#"
```rust
use fret_ui_material3 as m3;
use std::sync::Arc;

let open = app.models_mut().insert(false);
let value = app.models_mut().insert(Arc::<str>::from("search"));
let root = m3::ModalNavigationDrawer::new(open).into_element(
    cx,
    |cx| m3::NavigationDrawer::new(value).variant(m3::NavigationDrawerVariant::Modal).into_element(cx),
    |cx| cx.text("Content"),
);
```
"#;

pub(crate) const DOC_MATERIAL3_DIALOG: &str = r#"
## Material 3 Dialog (MVP)

This page validates a Material 3 dialog surface:

- modal barrier (no click-through)
- scrim opacity (default policy) + deterministic motion timeline
- focus trap while open + focus restore on close
- dialog actions use `md.comp.dialog.action.*` tokens for label/state-layer outcomes
"#;

pub(crate) const USAGE_MATERIAL3_DIALOG: &str = r#"
```rust
use fret_ui_material3 as m3;

let open = app.models_mut().insert(false);
let dialog = m3::Dialog::new(open)
    .headline("Title")
    .supporting_text("Supporting text")
    .actions(vec![m3::DialogAction::new("OK")])
    .into_element(cx, |cx| cx.text("Underlay"), |_cx| vec![]);
```
"#;

pub(crate) const DOC_MATERIAL3_MENU: &str = r#"
## Material 3 Menu (MVP)

This page validates a Material 3 menu surface:

- token-driven menu container + list item sizing
- roving focus (Up/Down/Home/End) + prefix typeahead
- state layer (hover / pressed / focus)
- bounded ripple (pointer-origin)
- dismissible overlay outcomes (Escape / outside press, anchored to trigger)

Notes:
- This is a dropdown overlay MVP built on top of the in-place `Menu` list surface.
"#;

pub(crate) const USAGE_MATERIAL3_MENU: &str = r#"
```rust
use fret_ui_material3 as m3;

let menu = m3::Menu::new().entries(vec![
    m3::MenuEntry::Item(m3::MenuItem::new("Cut")),
    m3::MenuEntry::Separator,
    m3::MenuEntry::Item(m3::MenuItem::new("Paste").disabled(true)),
]);
```
"#;

pub(crate) const DOC_MATERIAL3_LIST: &str = r#"
## Material 3 List (MVP)

This page validates the Material 3 list surface:

- token-driven list item sizing (`md.comp.list.list-item.*`)
- selection follows focus (roving focus → model update)
- state layer + bounded ripple aligned to item bounds
"#;

pub(crate) const USAGE_MATERIAL3_LIST: &str = r#"
```rust
use fret_ui_material3 as m3;
use std::sync::Arc;

let value = app.models_mut().insert(Arc::<str>::from("alpha"));
let list = m3::List::new(value).items(vec![
    m3::ListItem::new("alpha", "Alpha"),
    m3::ListItem::new("beta", "Beta").disabled(true),
]);
```
"#;

pub(crate) const DOC_MATERIAL3_SNACKBAR: &str = r#"
## Material 3 Snackbar (MVP)

This page validates a Material 3 snackbar surface:

- posted via a dedicated toast store (so it does not conflict with the gallery's shadcn toaster)
- rendered by `fret-ui-kit` toast layer using a Material token-driven skin (`md.comp.snackbar.*`)
- action + dismiss icon use snackbar state-layer tokens
"#;

pub(crate) const USAGE_MATERIAL3_SNACKBAR: &str = r#"
```rust
use fret_ui_material3 as m3;
use fret_ui_kit::ToastStore;

let store = cx.app.models_mut().insert(ToastStore::default());
let _snackbar_host = m3::SnackbarHost::new(store.clone()).into_element(cx);

// In an action handler:
let controller = m3::SnackbarController::new(store);
let _id = controller.show(host, acx.window, m3::Snackbar::new("Saved").action("Undo", cmd));
```
"#;

pub(crate) const DOC_MATERIAL3_TOOLTIP: &str = r#"
## Material 3 Tooltip (MVP)

This page validates Material 3 tooltip surfaces (plain + rich):

- Radix-aligned delay group + hover intent + safe-hover corridor (via `fret-ui-kit`)
- deterministic open/close motion driven by `md.sys.motion.spring.*` (fast spatial/effects springs)
- token-driven container/text styling via `md.comp.{plain,rich}-tooltip.*`

Notes:

- In Fret, `OverlayKind::Tooltip` is click-through, so rich tooltip actions are currently out of
  scope.
"#;

pub(crate) const USAGE_MATERIAL3_TOOLTIP: &str = r#"
```rust
use fret_ui_material3 as m3;

m3::TooltipProvider::new().with_elements(cx, |cx| {
    let trigger = m3::Button::new("Hover me").into_element(cx);

    let plain = m3::PlainTooltip::new(trigger, "Plain tooltip text").into_element(cx);
    let rich = m3::RichTooltip::new(
        m3::Button::new("Hover me (rich)").into_element(cx),
        "Supporting text",
    )
    .title("Title")
    .into_element(cx);

    [plain, rich]
})
```
"#;
