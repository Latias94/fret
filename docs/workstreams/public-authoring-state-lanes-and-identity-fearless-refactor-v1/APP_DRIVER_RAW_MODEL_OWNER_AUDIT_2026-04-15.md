# App/Driver Raw Model Owner Audit — 2026-04-15

Status: landed follow-on audit
Last updated: 2026-04-15

Related:

- `TODO.md`
- `MILESTONES.md`
- `apps/fret-cookbook/examples/embedded_viewport_basics.rs`
- `apps/fret-examples/src/embedded_viewport_demo.rs`
- `apps/fret-examples/src/external_texture_imports_demo.rs`
- `apps/fret-examples/src/external_texture_imports_web_demo.rs`
- `apps/fret-examples/src/external_video_imports_avf_demo.rs`
- `apps/fret-examples/src/external_video_imports_mf_demo.rs`
- `apps/fret-examples/src/plot_stress_demo.rs`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `apps/fret-examples/src/launcher_utility_window_demo.rs`
- `apps/fret-examples/src/components_gallery.rs`

## Why this note exists

After the render-root selector, stress-root, GenUI message-lane, and IMUI bridge-owner slices
landed, the remaining raw `app.models()` grep tail in first-party examples was no longer one
backlog.

The clean split is now:

- pure app/driver loops that intentionally own `&mut App` plus engine/runtime orchestration,
- retained/component owners that still need their own separate audit,
- and source-policy test markers that only record older drift for regression coverage.

This note freezes the first of those three classes so future cleanup does not try to migrate it
onto render-lane helpers by reflex.

## Findings

### 1. Embedded viewport recorders are app/driver owners

`embedded_viewport_demo` and cookbook `embedded_viewport_basics` both read tracked data inside
`record_embedded_viewport(...)`, not inside `AppUi` or `ElementContext` render code.

Those functions own:

- `&mut KernelApp`,
- GPU encoder/texture state,
- and per-frame recorder orchestration.

That makes raw `app.models().read(...)` the correct surface for click/UV diagnostics there.

### 2. External texture/video import frame recorders are app/driver owners

`external_texture_imports_demo`, `external_texture_imports_web_demo`,
`external_video_imports_avf_demo`, and `external_video_imports_mf_demo` already moved their
render-root `show` reads onto `cx.data().selector_model_layout(...)`.

The remaining raw `show` reads live in `record_engine_frame(...)`, where the function owns:

- target registration/unregistration,
- native/web importer lifecycle,
- GPU texture allocation,
- and frame submission bookkeeping.

Those are driver responsibilities, so the raw store read should stay there.

### 3. Command/effect loops are app/driver owners too

`workspace_shell_demo` and `launcher_utility_window_demo` still read tracked state from raw
`app.models()` inside command handlers.

That is correct because the owner there is not render code; it is command dispatch and effect
orchestration:

- workspace dirty-close resolution,
- utility-window `AlwaysOnTop` style toggling.

Moving those reads onto render helpers would blur the render-vs-command boundary rather than
improve authoring clarity.

### 4. Plot stress driver helpers are raw by design

`plot_stress_demo` reads `state.animate` from raw `app.models()` inside the driver-side animation
and reporting helpers.

Those helpers run in the frame/driver loop, outside declarative render ownership, so the raw
store surface remains the right one.

### 5. Retained/component owners are out of scope here

`components_gallery` still contains raw `app.models()` reads, but that file mixes retained table
hosting, theme sync, and component-gallery-specific owner seams.

That remaining class should be audited separately instead of being folded into this pure
app/driver note.

## Landed result

This audit lands:

- a dedicated examples source-policy gate that locks raw `app.models()` ownership inside the
  driver/app-loop slices above,
- a dedicated cookbook source-policy gate for `embedded_viewport_basics`,
- and an explicit statement that `components_gallery` is the next separate owner class rather than
  evidence that all raw `app.models()` usage is still unresolved.

## Decision from this audit

Treat pure app/driver loops as an explicit raw `ModelStore` owner class.

Do not migrate these functions onto:

- `cx.data().selector_*`,
- `LocalState::*_value_in(...)`,
- or other render-lane grouped helpers,

unless the surrounding owner boundary itself moves into render code.
