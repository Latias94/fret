# AppUi Overlay Root Capability Surface Audit — 2026-04-17

Status: Landed slice + targeted evidence pass

Related:

- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/TODO.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/MILESTONES.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/APP_UI_DEREF_COMPILE_AUDIT_2026-04-17.md`
- `docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/API_WORKBENCH_FRAMEWORK_PRIORITY_AUDIT_2026-04-15.md`
- `docs/adr/0319-public-authoring-state-lanes-and-identity-contract-v1.md`
- `ecosystem/fret-ui-shadcn/src/{dialog.rs,alert_dialog.rs,sheet.rs,drawer.rs}`
- `ecosystem/fret-ui-shadcn/src/ui_builder_ext/overlay_roots.rs`
- `ecosystem/fret-workspace/src/frame.rs`
- `apps/fret-examples/src/{api_workbench_lite_demo.rs,editor_notes_device_shell_demo.rs,emoji_conformance_demo.rs}`

## Scope

Decide how the remaining app-lane pressure around late-landed overlay roots should be resolved
without reopening blanket `AppUi` forwarding:

- direct shadcn overlay roots:
  - `Dialog`
  - `AlertDialog`
  - `Sheet`
  - `Drawer`
- root-level `UiBuilder` overlay adapters in
  `ecosystem/fret-ui-shadcn/src/ui_builder_ext/overlay_roots.rs`
- the product-shell frame surfaced by the same real app proof:
  - `WorkspaceFrame`

This note is intentionally about authoring capability shape, not overlay interaction policy.

## Method

1. Inspect the current no-`Deref` tail and real proof surfaces:
   - `apps/fret-examples/src/api_workbench_lite_demo.rs`
   - `apps/fret-examples/src/editor_notes_device_shell_demo.rs`
   - `apps/fret-examples/src/emoji_conformance_demo.rs`
2. Land the smallest owner-correct framework slice:
   - add `build_in(...)` / `into_element_in(...)` on the direct shadcn overlay roots that still
     hard-require `&mut ElementContext<'_, H>`,
   - add matching `into_element_in(...)` overloads on the `UiBuilder` overlay-root adapters,
   - add `WorkspaceFrame::into_element_in(...)` for the same capability-first late-landing shape.
3. Migrate real app-facing callsites and gates:
   - `api_workbench_lite_demo`
   - `editor_notes_device_shell_demo`
   - `emoji_conformance_demo`
   - `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`
   - `ecosystem/fret-workspace/src/lib.rs`
   - `apps/fret-examples/src/lib.rs`
   - `apps/fret-examples/tests/editor_notes_device_shell_surface.rs`
4. Re-run the temporary local no-`Deref` compile audit:
   - remove `impl Deref` / `impl DerefMut` for `AppUi` in `ecosystem/fret/src/view.rs`
   - `cargo check -p fret-examples --all-targets --message-format short`
   - `cargo check -p fret-cookbook --all-targets --message-format short`

The temporary patch was reverted immediately after collecting the new failure map.

## Findings

### 1) Direct overlay-root late builders are app-facing authoring surfaces

The pressure here is not “we need all of `ElementContext` back on `AppUi`”.

The real gap was narrower: direct root-level overlay recipes still required a raw
`&mut ElementContext<'_, H>` even though they only needed to late-land authored trigger/content
children. That is the same capability shape already handled by `ElementContextAccess<'a, H>` on
other app-lane builder surfaces.

The correct fix is therefore explicit capability overloads on the direct roots, not blanket
forwarding from `AppUi`.

### 2) Existing generic `IntoUiElementInExt` already covers the wrapper layer

This slice does **not** need a second API family for every wrapper that already implements
`IntoUiElement<H>`.

`DialogChildren`, `DrawerChildren`, and similar wrapper/composition surfaces already participate in
the generic `into_element_in(...)` extension lane once the callsite imports
`fret_ui_kit::IntoUiElementInExt as _`.

The missing explicit owner decision was therefore concentrated on:

- direct root types that expose `into_element(cx, trigger, content)`,
- and direct root-level `UiBuilder` adapters that lowered into those same raw entry points.

### 3) The editor shell proof exposed one more owner-correct late-builder seam

While migrating `editor_notes_device_shell_demo`, the same capability-shape pressure surfaced on
`WorkspaceFrame`.

`WorkspaceFrame` is not a raw mechanism seam; it is an ecosystem shell builder that late-lands
typed slot children. Keeping it on raw `ElementContext` would have forced the product proof back
through `AppUi` `Deref` for the wrong reason.

Adding `WorkspaceFrame::into_element_in(...)` keeps the editor-shell authoring story aligned with
the rest of this lane.

### 4) The real proofs now use explicit capability lanes instead of stale bridge syntax

This slice leaves concrete evidence on three real first-party surfaces:

- `api_workbench_lite_demo`
  - uses `cx.app().global::<HistoryDbGlobal>()`
  - uses `Dialog::into_element_in(...)`
- `editor_notes_device_shell_demo`
  - imports `IntoUiElementInExt`
  - lands `WorkspaceFrame` and the mobile drawer root through explicit `into_element_in(...)`
- `emoji_conformance_demo`
  - uses `cx.app().global::<FontCatalogCache>()`

That is the right kind of evidence for this lane: real app authoring friction removed without
re-expanding the façade into a raw `ElementContext` clone.

### 5) The temporary no-`Deref` audit gets materially smaller again

With the temporary local patch that removes both `impl Deref` and `impl DerefMut` from `AppUi`:

- `fret-examples` now emits `126` error lines,
- `fret-cookbook` stays at `22`,
- `api_workbench_lite_demo` is no longer part of the failure tail,
- `editor_notes_device_shell_demo` is no longer part of the failure tail,
- and the remaining app-lane pressure is now concentrated more clearly on
  `emoji_conformance_demo` plus the broader unresolved raw builder/text surface family.

## Evidence

- `ecosystem/fret-ui-shadcn/src/{dialog.rs,alert_dialog.rs,sheet.rs,drawer.rs}`
- `ecosystem/fret-ui-shadcn/src/ui_builder_ext/overlay_roots.rs`
- `ecosystem/fret-ui-shadcn/src/surface_policy_tests.rs`
- `ecosystem/fret-workspace/src/{frame.rs,lib.rs}`
- `apps/fret-examples/src/{api_workbench_lite_demo.rs,editor_notes_device_shell_demo.rs,emoji_conformance_demo.rs}`
- `apps/fret-examples/src/lib.rs`
- `apps/fret-examples/tests/editor_notes_device_shell_surface.rs`
- `cargo test -p fret-ui-shadcn --lib overlay_root_late_builders_offer_explicit_context_access_overloads`
- `cargo test -p fret-ui-shadcn --lib provider_late_builders_offer_explicit_context_access_overloads`
- `cargo test -p fret-examples --lib api_workbench_lite_demo_uses_query_for_sqlite_reads_and_mutation_for_explicit_submit`
- `cargo test -p fret-examples --lib selected_app_ui_roots_prefer_explicit_render_context_accessors_over_deref`
- `cargo test -p fret-examples --test editor_notes_device_shell_surface editor_notes_device_shell_demo_keeps_shell_switch_explicit_and_reuses_inner_editor_content`
- temporary local no-`Deref` audit:
  - `cargo check -p fret-examples --all-targets --message-format short`
  - `cargo check -p fret-cookbook --all-targets --message-format short`

## Outcome

The repo now has an explicit capability-first answer for direct overlay-root late builders:

1. direct shadcn overlay roots expose app-lane `*_in(...)` entry points instead of depending on
   `AppUi` `Deref`,
2. the root-level `UiBuilder` overlay adapters teach the same capability shape,
3. `WorkspaceFrame` follows the same late-landing owner rule for editor-shell chrome,
4. real app proofs now use explicit `app()` / `into_element_in(...)` accessors rather than stale
   field syntax or hidden bridge inheritance,
5. the temporary no-`Deref` audit drops `fret-examples` from `140` errors to `126` and removes the
   current `api_workbench_lite_demo` / `editor_notes_device_shell_demo` pressure from the failure
   map.
