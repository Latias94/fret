# Adaptive Layout Contract Closure v1 — Milestones

Status: Closed closeout lane (adaptive taxonomy + proof closure shipped; follow-on only)
Last updated: 2026-04-10

Closeout note on 2026-04-10:

- `CLOSEOUT_AUDIT_2026-04-10.md` closes this lane on the shipped adaptive taxonomy / proof
  surface / editor-rail owner-split goal.
- Future work should start as a narrower follow-on rather than reopening this broad lane.

## M0 — Baseline and inventory freeze

Exit criteria:

- The lane clearly states why it exists separately from the older query implementation trackers.
- The current adaptive feature inventory is explicit.
- The first drift list is grouped by axis and owning layer instead of by random component bugs.

Primary evidence:

- `docs/workstreams/adaptive-layout-contract-closure-v1/DESIGN.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/TODO.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/BASELINE_AUDIT_2026-04-10.md`
- `docs/workstreams/container-queries-v1/container-queries-v1.md`
- `docs/workstreams/environment-queries-v1/environment-queries-v1.md`
- `docs/known-issues.md`

Current status:

- Opened on 2026-04-10.
- The first gallery narrow-surface proof already exists via the popup/menu sweep.
- M0 baseline and inventory freeze closed on 2026-04-10.
- The next active work is M1 adaptive taxonomy freeze.

## M1 — Adaptive taxonomy freeze

Exit criteria:

- The v1 adaptive feature set is explicit.
- Query-axis ownership is frozen for the public story.
- The `fret::env` app-facing import lane is either confirmed or narrowed with a bounded follow-up.

Primary evidence:

- `docs/workstreams/adaptive-layout-contract-closure-v1/DESIGN.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/TARGET_INTERFACE_STATE.md`
- `docs/workstreams/adaptive-layout-contract-closure-v1/M1_CONTRACT_FREEZE_2026-04-10.md`
- `docs/crate-usage-guide.md`
- `ecosystem/fret/src/lib.rs`
- `ecosystem/fret-ui-kit/src/declarative/container_queries.rs`
- `ecosystem/fret-ui-kit/src/declarative/viewport_queries.rs`
- `docs/adr/0325-adaptive-authoring-surface-and-query-axis-taxonomy-v1.md`

Current status:

- Closed on 2026-04-10 via ADR 0325 plus the target-interface freeze note.
- `fret::env` remains the explicit low-level lane; any future higher-level adaptive lane belongs
  above it in ecosystem policy/facade space rather than in `crates/fret-ui`.
- A minimal shared adaptive policy lane now exists in `ecosystem/fret-ui-kit/src/adaptive.rs` and
  is exposed to app code on the explicit `fret::adaptive::{...}` lane.
- Adaptive participation alone is no longer considered evidence for widening generic
  `children(...)` APIs.
- The next active work is M2 proof-surface promotion plus bounded recipe/API cleanup slices.

## M2 — Proof surfaces and gates

Exit criteria:

- One gallery narrow-window proof surface is stable and reviewable.
- One panel-resize proof surface is promoted into the active gate set.
- At least one adaptive strategy/example surface is anchored in evidence.

Primary gates:

- `cargo nextest run -p fret-ui-gallery --test popup_menu_narrow_surface`
- `cargo nextest run -p fret-ui-gallery --test navigation_menu_docs_surface --no-fail-fast`
- `cargo nextest run -p fret-ui-shadcn --test navigation_menu_query_mode_reopen --no-fail-fast`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/navigation/ui-gallery-navigation-menu-md-breakpoint-query-source-toggle.json --dir target/fret-diag/adaptive-navigation-menu-query-axis --session-auto --pack --include-screenshots --launch target/release/fret-ui-gallery`
- `cargo run -p fretboard -- diag run tools/diag-scripts/ui-gallery/overlay/ui-gallery-popup-menu-narrow-sweep.json --dir target/fret-diag-popup-menu-narrow-sweep --session-auto --pack --include-screenshots --launch target/release/fret-ui-gallery`
- `cargo run -p fretboard -- diag run tools/diag-scripts/container-queries-docking-panel-resize.json --dir target/fret-diag/container-queries-docking --session-auto --pack --include-screenshots --launch target/release/container_queries_docking_demo`

Current status:

- The UI Gallery narrow-surface gate now covers both popup/menu surfaces and the Dialog demo
  trigger/content lane under a 420px-wide window.
- `ui-gallery-overlay-narrow-header-sweep.json` also passed after the Dialog width-lane fix, so
  sampled Popover / Sheet / Drawer / Alert Dialog overlay surfaces do not currently show the same
  narrow-window regression class.
- The panel-resize gate was promoted on 2026-04-10 via the `container_queries_docking_demo`
  release build plus a passing diag run under
  `target/fret-diag/adaptive-panel-resize-promote/sessions/1775822919781-88694`.
- The promoted script now leaves before/after layout sidecars in addition to screenshots and
  bounded bundles, so container-width ownership is reviewable without reopening the older
  container-query implementation lane.
- The query-axis teaching proof was promoted on 2026-04-10 via the Navigation Menu docs page,
  focused component/gallery tests, and a passing diag run under
  `target/fret-diag/adaptive-navigation-menu-query-axis/sessions/1775826527322-55840`.
- Closed on 2026-04-10 via
  `docs/workstreams/adaptive-layout-contract-closure-v1/M2_GALLERY_QUERY_AXIS_PROOF_2026-04-10.md`.
- The follow-on M3 rail-composition slice that this milestone pointed to is now complete as well;
  see `docs/workstreams/adaptive-layout-contract-closure-v1/M3_EDITOR_RAIL_COMPOSITION_2026-04-10.md`.

## M3 — First fearless-refactor slices

Exit criteria:

- At least one mixed-debt surface is refactored without breaking the mechanism/policy boundary.
- The refactor is proven on both source-aligned docs/gallery surfaces and a regression gate.
- Follow-up work is narrowed instead of left as a vague "responsive cleanup" backlog.

Primary gates:

- Targeted `cargo nextest run` for the affected recipe/page crates
- Relevant `fretboard diag run ...` proof surface
- `python3 tools/check_layering.py` when the slice touches crate boundaries

Current status:

- Started on 2026-04-10 with a bounded dialog width-hygiene slice.
- The landed slice kept width ownership in the Gallery caller shell instead of pushing new
  responsive policy into `fret-ui` or shadcn recipe defaults.
- A second bounded rename slice now moved ambiguous recipe APIs onto explicit axes:
  `Combobox::device_shell_responsive(...)` / `device_shell_md_breakpoint(...)` for the
  viewport/device-shell branch and `FieldOrientation::ContainerAdaptive` for the container-query
  field layout lane.
- A third bounded docs-surface slice pins `SidebarProvider::is_mobile(...)` /
  `is_mobile_breakpoint(...)` as app-shell/device-shell vocabulary and keeps editor/panel rails as
  a separate future surface instead of forcing a premature rename.
- A fourth bounded audit slice now confirms that reusable editor side panels should stay on the
  existing editor/workspace owner path (`fret-ui-editor` composites plus workspace shell), while
  `fret-docking` keeps topology/registry ownership and shadcn `Sidebar` remains app-shell only.
- A fifth bounded audit slice now resolves the next seam decision: use the existing
  `fret-workspace::WorkspaceFrame.left/right` shell slots as the outer rail seam, keep
  `fret-ui-editor` as the reusable inner-panel owner, and keep the concrete rail recipe app-local
  until a second consumer proves extraction.
- A sixth bounded slice now implements that seam decision in running code:
  `workspace_shell_demo` mounts an editor rail through `WorkspaceFrame.right(...)` while the inner
  surface stays on `InspectorPanel + PropertyGroup + PropertyGrid`.
- Closed on 2026-04-10 via
  `docs/workstreams/adaptive-layout-contract-closure-v1/M3_EDITOR_RAIL_COMPOSITION_2026-04-10.md`.
- This milestone handed the lane back to M4 closeout / follow-on split decisions, which are now
  resolved.

## M4 — Closeout or split

Exit criteria:

- The remaining backlog is clearly small enough for maintenance, or
- the lane splits a narrower follow-on with explicit ownership.

Primary evidence:

- `docs/workstreams/adaptive-layout-contract-closure-v1/EVIDENCE_AND_GATES.md`
- updated roadmap / todo / known-issues entrypoints
- explicit closeout note or new follow-on lane

Current status:

- Closed on 2026-04-10 via
  `docs/workstreams/adaptive-layout-contract-closure-v1/CLOSEOUT_AUDIT_2026-04-10.md`.
- Repo-level entrypoints now point readers to:
  - `TARGET_INTERFACE_STATE.md`
  - `EVIDENCE_AND_GATES.md`
  - `CLOSEOUT_AUDIT_2026-04-10.md`
- `container-queries-v1` and `environment-queries-v1` remain mechanism/reference lanes rather than
  reopened authoring trackers.
- Future adaptive work should start as a narrower follow-on only if fresh evidence appears.
