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

pub(crate) const DOC_VIEW_CACHE: &str = r#"
## View Cache (experimental)

This page is a **stress + acceptance** harness for GPUI-style cached subtree execution.

When view-cache mode is enabled, a `ViewCache` wrapper can become a cache boundary:

- model/global invalidations stop at the nearest cache root,
- paint-cache is only allowed for cache roots (so they can range-replay),
- on cache-hit, the runtime may skip executing the child render closure and reuse the previous retained subtree.

The goal is "editor-grade smoothness" with fewer full-tree re-renders, while keeping correctness (state + invalidation).
"#;

pub(crate) const USAGE_VIEW_CACHE: &str = r#"
```rust
let subtree = cx.view_cache(ViewCacheProps::default(), |cx| {
    // expensive subtree build here
    vec![cx.text("...")]
});
```
"#;

pub(crate) const DOC_VIRTUAL_LIST_TORTURE: &str = r#"
## Virtual List (torture harness)

This page exists to validate "editor-grade scrolling feel" under realistic composition:

- 10k+ virtualized rows
- row focus + selection-like interactions
- scroll-to-item correctness (anchor preservation + measured heights)
- a small inline text input inside the list

The goal is not to ship a component; it is to provide a deterministic surface for performance
instrumentation and regression scripts (GPUI parity workstream).
"#;

pub(crate) const USAGE_VIRTUAL_LIST_TORTURE: &str = r#"
```rust
let handle = VirtualListScrollHandle::new();

let list = cx.virtual_list_keyed(
    len,
    VirtualListOptions::new(Px(28.0), 8),
    &handle,
    |i| i as ItemKey,
    |cx, i| row(cx, i),
);
```
"#;

pub(crate) const DOC_CODE_VIEW_TORTURE: &str = r#"
## Code View (torture harness)

This page is a stress surface for **large scrollable text/code documents**.

It is intended to back the GPUI parity workstream:

- validate scroll stability (no “stale paint” / “UI looks not refreshed” regressions)
- identify when code/text surfaces should become **prepaint-windowed** (ADR 0190)
- provide a deterministic bundle capture target for perf investigations
"#;

pub(crate) const USAGE_CODE_VIEW_TORTURE: &str = r#"
```rust
use fret_code_view::CodeBlock;

let code = Arc::<str>::from("...");
let block = CodeBlock::new(code).language("rust").show_line_numbers(true);
```
"#;

pub(crate) const DOC_CODE_EDITOR_MVP: &str = r#"
## Code Editor (MVP)

This page hosts a v1 MVP for a **paint-driven, windowed code editor surface**:

- Owns its buffer + selection state (ecosystem crate, not `fret-ui`).
- Uses a `TextInputRegion` seam to receive `TextInput` / `Ime` events while it draws its own text.
- Focuses on validating the input/IME contract and scroll stability before performance work.
"#;

pub(crate) const USAGE_CODE_EDITOR_MVP: &str = r#"
```rust
use fret_code_editor::{CodeEditor, CodeEditorHandle};

let handle = CodeEditorHandle::new("fn main() {}\n");
let editor = CodeEditor::new(handle).into_element(cx);
```
"#;

pub(crate) const DOC_CODE_EDITOR_TORTURE: &str = r#"
## Code Editor (torture harness)

This page is a stress surface for the **windowed, paint-driven code editor**.

Goals:

- validate scroll stability (no “stale paint” / “looks not refreshed” regressions),
- validate text blob caching stays bounded to the visible window,
- provide a deterministic target for perf investigations.
"#;

pub(crate) const USAGE_CODE_EDITOR_TORTURE: &str = r#"
```rust
use fret_code_editor::{CodeEditor, CodeEditorHandle, CodeEditorTorture};
use fret_core::Px;

let handle = CodeEditorHandle::new("...\n");
let editor = CodeEditor::new(handle)
    .overscan(128)
    .torture(CodeEditorTorture::auto_scroll_bounce(Px(8.0)))
    .into_element(cx);
```
"#;

pub(crate) const DOC_CHART_TORTURE: &str = r#"
## Chart (torture harness)

This page is a stress surface for **canvas-driven charts** with pan/zoom interactions.

It exists to support the GPUI parity workstream:

- validate “no stale scene” behavior under view-cache reuse,
- identify where charts/plots should adopt prepaint-windowed sampling (ADR 0190),
- provide a deterministic bundle capture target for perf investigations.
"#;

pub(crate) const USAGE_CHART_TORTURE: &str = r#"
```rust
use fret_chart::{ChartCanvasPanelProps, chart_canvas_panel};

let props = ChartCanvasPanelProps::new(spec);
let el = chart_canvas_panel(cx, props);
```
"#;

pub(crate) const DOC_CANVAS_CULL_TORTURE: &str = r#"
## Canvas Cull (torture harness)

This page is a stress surface for **pan/zoom canvas scenes** with viewport-driven culling.

It exists to support the GPUI parity workstream:

- validate “no stale scene” behavior under view-cache reuse,
- identify when large canvas/node-graph surfaces should become **prepaint-windowed** (ADR 0190),
- provide a deterministic bundle capture target for perf investigations.
"#;

pub(crate) const USAGE_CANVAS_CULL_TORTURE: &str = r#"
```rust
use fret_canvas::ui::{PanZoomCanvasSurfacePanelProps, pan_zoom_canvas_surface_panel};

let props = PanZoomCanvasSurfacePanelProps::default();
let el = pan_zoom_canvas_surface_panel(cx, props, |_painter, _cx| {});
```
"#;

pub(crate) const DOC_CHROME_TORTURE: &str = r#"
## Chrome (torture harness)

This page is a stress surface for interaction-driven “chrome”:

- hover/pressed/focus rings,
- caret/selection visuals,
- overlay open/close loops.

It exists to support the GPUI parity workstream:

- validate that “hover-only” and “focus-only” ticks can be paint-only under view-cache reuse,
- catch stale-paint regressions where semantics/hit-testing updates but the scene does not.
"#;

pub(crate) const USAGE_CHROME_TORTURE: &str = r#"
This page is intentionally policy-heavy and should be driven via diagnostics scripts.
"#;

pub(crate) const DOC_WINDOWED_ROWS_SURFACE_TORTURE: &str = r#"
## Windowed Rows Surface (torture harness)

This page is a baseline for **scroll-driven windowing without per-row declarative subtrees**.

It uses a structurally stable element tree (a `Scroll` + a leaf `Canvas`) and paints only the
visible row window inside the canvas paint handler.

This is intended to validate:

- scroll stability under view-cache reuse (no stale paint),
- near-minimal CPU work for scroll-only deltas,
- a reusable pattern for huge “list-like” surfaces that do not need per-row semantics/focus.
"#;

pub(crate) const USAGE_WINDOWED_ROWS_SURFACE_TORTURE: &str = r#"
```rust
use fret_ui_kit::declarative::windowed_rows_surface::{
    WindowedRowsSurfaceProps, windowed_rows_surface,
};

let el = windowed_rows_surface(cx, WindowedRowsSurfaceProps::default(), |_p, _i, _rect| {});
```
"#;

pub(crate) const DOC_WINDOWED_ROWS_SURFACE_INTERACTIVE_TORTURE: &str = r#"
## Windowed Rows Surface (interactive harness)

This page demonstrates a “windowed surface” pattern (ADR 0190) with **paint-only hover chrome**
(ADR 0181) using a stable element tree:

- `Scroll` (retained scroll state + transform)
- `PointerRegion` (row hit-testing in event hooks)
- `Canvas` (paint only the visible row window)

The goal is to show that pointer-driven chrome (hover highlight) can update via paint invalidation
without forcing rerender or relayout.
"#;

pub(crate) const USAGE_WINDOWED_ROWS_SURFACE_INTERACTIVE_TORTURE: &str = r#"
```rust
use fret_ui::element::PointerRegionProps;
use fret_ui_kit::declarative::windowed_rows_surface::{
    WindowedRowsSurfacePointerHandlers, WindowedRowsSurfaceProps,
    windowed_rows_surface_with_pointer_region,
};

let props = WindowedRowsSurfaceProps::default();
let pointer = PointerRegionProps::default();
let handlers = WindowedRowsSurfacePointerHandlers::default();

let el = windowed_rows_surface_with_pointer_region(
    cx,
    props,
    pointer,
    handlers,
    None,
    |_p, _i, _rect| {},
);
```
"#;

pub(crate) const DOC_DATA_TABLE_TORTURE: &str = r#"
## DataTable (torture harness)

This page is a baseline for **virtualized business tables** built with:

- `fret-ui-headless` TanStack-aligned table engine,
- `fret-ui-kit` virtualization helpers,
- `fret-ui-shadcn` `DataTable` integration surface.

This harness exists to make performance regressions measurable and reproducible when refactoring
VirtualList windowing, row measurement modes, and cache-root placement.
"#;

pub(crate) const USAGE_DATA_TABLE_TORTURE: &str = r#"
```rust
use fret_ui_shadcn::DataTable;

let table = DataTable::new();
```
"#;

pub(crate) const DOC_TREE_TORTURE: &str = r#"
## Tree (torture harness)

This page is a baseline for **virtualized trees** built with `fret-ui-kit::declarative::tree_view`.

It exists to validate:

- scroll stability under view-cache reuse (no stale paint),
- row-window correctness (expand/collapse does not detach state on cache hits),
- future migrations toward prepaint-driven windowing (ADR 0190).
"#;

pub(crate) const USAGE_TREE_TORTURE: &str = r#"
```rust
use fret_ui_kit::declarative::tree::tree_view;
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

pub(crate) const DOC_MATERIAL3_BUTTON: &str = r#"
## Material 3 Button (MVP)

This page validates the first Material 3 outcome-aligned component surface:

- state layer (hover / pressed / focus) driven by Material tokens
- bounded ripple (pointer-origin) driven by motion tokens
- ADR 1159 style overrides via `ButtonStyle` (partial per-state overrides)

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
- ADR 1159 style overrides via `IconButtonStyle` (partial per-state overrides)
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
- ADR 1159 style overrides via `CheckboxStyle` (partial per-state overrides)

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
- ADR 1159 style overrides via `SwitchStyle` (partial per-state overrides)
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
- ADR 1159 style overrides via `RadioStyle` (partial per-state overrides)

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

pub(crate) const DOC_MATERIAL3_SELECT: &str = r#"
## Material 3 Select (MVP)

This page validates a Material 3 select surface:

- token-driven trigger outcomes via `md.comp.{outlined,filled}-select.*`
- listbox overlay anchored to the trigger (Escape / outside press dismissal)
- ADR 1159 style overrides via `SelectStyle` (partial per-state overrides)
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

pub(crate) const DOC_MATERIAL3_TEXT_FIELD: &str = r#"
## Material 3 Text Field (MVP)

This page validates Material 3 text field variants:

- outlined: token-driven outline colors + widths (hover/focus/error/disabled)
- filled: token-driven filled container + active indicator + hover state layer
- label + placeholder outcomes (best-effort)
- outlined: animated label float + an outline "notch" patch (best-effort)
- ADR 1159 style overrides via `TextFieldStyle` (partial per-state overrides)

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
- ADR 1159 style overrides via `TabsStyle` (partial per-state overrides)
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

This page validates a Material 3 plain tooltip surface:

- Radix-aligned delay group + hover intent + safe-hover corridor (via `fret-ui-kit`)
- deterministic open/close motion driven by `md.sys.motion.*` (duration + cubic-bezier)
- token-driven container/text styling via `md.comp.plain-tooltip.*`
"#;

pub(crate) const USAGE_MATERIAL3_TOOLTIP: &str = r#"
```rust
use fret_ui_material3 as m3;

m3::TooltipProvider::new().with_elements(cx, |cx| {
    let trigger = m3::Button::new("Hover me").into_element(cx);
    [m3::PlainTooltip::new(trigger, "Tooltip text").into_element(cx)]
})
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
