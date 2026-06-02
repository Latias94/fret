# `fret-ui-material3`

Material Design 3 (and Expressive) component surface for Fret.

This crate is a **design-system surface** intended to mirror Material 3 visual and interaction
outcomes while keeping `crates/fret-ui` focused on mechanisms rather than Material-specific policy.

## Status

Experimental learning project (not production-ready).

## When to use

- You want a Material 3 / Material Expressive component surface on top of Fret.
- You want theme-token-driven components rather than ad-hoc styling.
- You want app-facing widgets to follow the same action-first authoring story as the rest of the
  action-first/view-runtime workstream.

## Public authoring model

Material3 recipes are app-facing components, but they still keep the framework boundary explicit:

- Controlled and copyable state: controlled roots keep explicit `new(model)` constructors. Copyable
  teaching paths use `new_controllable(cx, ...)`, `uncontrolled(cx)`, and `*_model()` accessors when
  the component can own local state safely.
- Action-first pressables: normal app-facing pressables expose `action(...)` for stable unit-action
  dispatch.
- Style overrides: components with public `*Style` types expose `.style(...)`. These style surfaces
  own intrinsic Material chrome, state-layer colors, shapes, text styles, density, and slot spacing.
  Page width, flex, grid placement, and surrounding layout remain caller-owned.
- Automation IDs: rendered recipes expose `.test_id(...)` and derive stable part IDs for triggers,
  listboxes, options, panels, indicators, chrome, and other automation-critical slots.
- Icons: recipes consume semantic `IconId` values such as `ui.*`; apps or reusable bundles install
  the actual icon provider.

Example:

```rust
use fret_ui_material3 as material3;

let title = material3::TextField::uncontrolled(cx)
    .label("Title")
    .test_id("settings.title");

let section = material3::Select::uncontrolled(cx)
    .label("Section")
    .test_id("settings.section");

let save = material3::Button::new("Save")
    .style(material3::ButtonStyle::default())
    .action("settings.save")
    .test_id("settings.save");
```

## Component families

- Actions and surfaces: `Button`, `Fab`, `IconButton`, `IconToggleButton`, `Card`, and
  `CarouselItem`.
- Selection controls: `Checkbox`, `Radio`, `RadioGroup`, `Switch`, `Slider`, `RangeSlider`,
  `SegmentedButtonSet`, `AssistChip`, `FilterChip`, `InputChip`, `SuggestionChip`, and `ChipSet`.
- Fields and search: `TextField`, `Select`, `ExposedDropdown`, `Autocomplete`, `SearchBar`,
  `SearchView`, `DatePickerDialog`, `DockedDatePicker`, `TimePickerDialog`, and `DockedTimePicker`.
- Navigation: `Tabs`, `NavigationBar`, `NavigationRail`, `NavigationDrawer`,
  `ModalNavigationDrawer`, `TopAppBar`, and `List`.
- Overlays and feedback: `Menu`, `DropdownMenu`, `Dialog`, `ModalBottomSheet`,
  `DockedBottomSheet`, `Snackbar`, `SnackbarHost`, `PlainTooltip`, and `RichTooltip`.
- Display feedback: `Badge`, `Divider`, `LinearProgressIndicator`, and
  `CircularProgressIndicator`.
- Foundation APIs: `tokens`, `motion`, and `context` expose the Material theme, motion, and
  tree-local override surfaces used by recipes.

## Proof surface

Public recipes are tracked by `material3_recipe_proof_manifest_v1.json`, headless golden suites,
focused behavior tests, gallery teaching-surface gates, and an API documentation manifest that keeps
rustdoc, README guidance, public re-exports, style builders, copyable state helpers, and stable
`test_id` APIs aligned.

## Features

- `state-selector`: opt into derived-state helper integration
- `state-query`: opt into async/query helper integration
- `state`: enables both selector + query integration
- `diagnostics`: enables `fret-ui` live diagnostic helpers used by Material 3 test and
  automation surfaces

## Authoring note

- Prefer action-first public spellings on normal app-facing surfaces.
- Keep command-shaped or lower-level spellings only where the surface is intentionally exposing a
  deeper compatibility/interop boundary.
- For example, snackbar-style actions should prefer the explicit action-first naming path in
  default-facing examples/docs.
- Default-facing clickable families such as `Button`, `Fab`, `IconButton`, `Checkbox`, `Switch`,
  `Radio`, `AssistChip`, `SuggestionChip`, `FilterChip`, and `InputChip` now expose
  `action(...)` directly; prefer that over wiring `.on_activate(cx.actions().dispatch::<A>())`
  when you only need a stable unit action on the app-facing lane.

## Icons

- Material 3 widgets consume semantic `IconId` / `ui.*` ids; they do not choose a vendor pack as
  part of the component contract.
- This crate does not install a default icon provider for you.
- App/bootstrap code should install a pack explicitly (`fret_icons_lucide::app::install`,
  `fret_icons_radix::app::install`, or your own bundle surface).
- If a reusable ecosystem bundle depends on Material 3 plus a specific icon pack, keep that
  composition on one installer/bundle surface so the app composes one named dependency bundle.

Example:

```rust
use fret_icons::ids;

fret_icons_lucide::app::install(app);

let _button = fret_ui_material3::Button::new("Search")
    .leading_icon(ids::ui::SEARCH);
```

## Upstream references (non-normative)

Primary references:

- Material Design 3: https://m3.material.io/
- Material Web: https://github.com/material-components/material-web
- Jetpack Compose Material 3: https://developer.android.com/jetpack/compose/designsystems/material3
- MUI Material UI: https://github.com/mui/material-ui

See also:

- [`docs/reference-stack-ui-behavior.md`](../../docs/reference-stack-ui-behavior.md)
- [`docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md`](../../docs/workstreams/action-first-authoring-fearless-refactor-v1/COMMAND_FIRST_RETAINED_SEAMS_DECISION_DRAFT.md)
