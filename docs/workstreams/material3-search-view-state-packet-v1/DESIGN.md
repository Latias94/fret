# Material 3 SearchView State Packet v1

Status: Closed
Last updated: 2026-05-28

## Why This Lane Exists

The closed Material3 component sweep classified `SearchView` as selector-ready but still a docked
MVP. The remaining Material gap is not token polish; it is presentation/state choreography:

- Compose Material3 models `SearchBarState` separately from text query state.
- It offers both docked and full-screen expanded search presentations.
- Back handling collapses the expanded search surface.
- Full-screen presentation is a modal surface with focused search input and results content.

Fret currently has a controlled `open: Model<bool>` and `query: Model<String>` plus a docked
dismissible popover. This lane evolves that shape without leaking Material policy into `crates/*`.

## Target State

- `SearchView` keeps the existing docked behavior as the default.
- A Material-owned full-screen presentation mode is available for compact/mobile-like surfaces.
- Open/close state remains explicit and model-driven.
- Escape/back-like dismissal routes through existing overlay policy and returns `open` to `false`.
- The expanded full-screen input receives initial focus and remains inside a modal focus scope.
- Stable `test_id` surfaces cover root, chrome, icons, overlay/panel, and the full-screen header
  surface without duplicate root ids.

## Source Precedence

- Compose Material3 `SearchBar.kt`: primary truth for state split, docked vs full-screen
  presentations, focus, and back handling.
- Material Design 3: primary UX truth for search roles and expanded search surfaces.
- Existing Fret overlay controller/focus-scope primitives: implementation substrate.
- Current Fret SearchBar/SearchView recipe: compatibility baseline for controlled/uncontrolled API
  and selector naming.

Local source anchor:

- `repo-ref/compose-multiplatform-core/compose/material3/material3/src/commonMain/kotlin/androidx/compose/material3/SearchBar.kt`

## Layer Ownership

- `ecosystem/fret-ui-material3/src/search_view.rs`: Material presentation enum, full-screen overlay
  composition, stable ids, and recipe-level focus handoff.
- `ecosystem/fret-ui-material3/src/search_bar.rs`: only shared header rendering hooks when needed;
  do not turn SearchBar into a SearchView state machine.
- `ecosystem/fret-ui-kit`: existing overlay modal/dismiss/focus primitives only, unless this lane
  proves a design-system-agnostic back-handler gap.
- `crates/*`: out of scope unless Fret needs a platform back/navigation event contract.

## In Scope

- Source packet for Compose SearchBar/SearchBarState outcomes.
- Add a full-screen SearchView presentation mode behind an explicit API.
- Focus and dismissal tests for the new mode.
- Update automation/golden/diag surfaces only as needed to prove the behavior.

## Out Of Scope

- Predictive back gesture progress and shape interpolation.
- Top app bar scroll behavior.
- Mobile IME inset choreography.
- New cross-platform `Back` key/event mechanism in `crates/*`.
- Replacing existing docked SearchView defaults.

## Closeout Condition

This lane can close when:

- full-screen SearchView is source-aligned at the state/presentation level,
- docked SearchView behavior remains green,
- Escape/back-equivalent dismissal and focus routing are gated,
- any true platform-back mechanism gap is explicitly split,
- and workstream JSON/catalog plus focused Material3 gates pass.

Closeout result on 2026-05-28: closed. The lane added explicit docked/full-screen presentation
selection, modal full-screen SearchView rendering, overlay-local header focus, Escape collapse, and
SearchView golden coverage. Predictive back gesture progress remains a future mechanism/policy
follow-on only if a product surface proves the need.
