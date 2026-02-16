# A11y + AccessKit xplat bridge v1 (audit + design)

Status: Draft (workstream notes only; ADRs remain the source of truth)

Tracking files:

- `docs/workstreams/a11y-accesskit-xplat-bridge-v1-todo.md`
- `docs/workstreams/a11y-accesskit-xplat-bridge-v1-milestones.md`

## 0) Why this workstream exists

Fret already has a largely-complete **portable semantics contract** (ADR 0033) and a solid AccessKit
mapping layer (`crates/fret-a11y-accesskit`). However, the native desktop runner is currently on
`winit 0.31.0-beta.2`, and AccessKit’s official `accesskit_winit` adapter lags behind the beta.

As a result, the final “last mile” glue in `crates/fret-runner-winit` is currently a stub (no-op),
which means OS-level accessibility is effectively disabled even though the semantics tree and the
AccessKit `TreeUpdate` generation are implemented.

This workstream locks an approach to:

1) audit our overall A11y implementation (excluding the runner glue), and
2) re-enable OS accessibility by implementing an **internal, winit-version-independent**
   AccessKit adapter (“xplat”), inspired by `repo-ref/blitz/packages/accesskit_xplat`.

## 1) Scope and non-goals

### In scope

- Audit the portable semantics contract and its current implementation:
  - schema (`crates/fret-core/src/semantics.rs`)
  - snapshot production + overlay gating (`crates/fret-ui/src/tree/ui_tree_semantics.rs`)
  - AccessKit mapping + action decoding (`crates/fret-a11y-accesskit/src/lib.rs`)
  - ecosystem policy tests that rely on semantics (`docs/a11y-acceptance-checklist.md` + shadcn tests)
- Design the internal AccessKit adapter glue that works with `winit 0.31.0-beta.2`.
- Define deliverables, gates, and acceptance checks.

### Explicit non-goals (v1)

- A full “web accessibility bridge” equivalent to AccessKit (see ADR 0180 for web IME scope).
- Platform-perfect virtualization patterns (UIA/AX-specific) beyond the existing portable contract
  (ADR 0084).
- Expanding semantics schema without an ADR + tests.

## 2) Current state audit (excluding winit glue)

This section is intentionally “runner-independent”: it focuses on contracts and conversion logic.

### 2.1 Portable semantics schema is present and validated (partially)

- Schema types live in `crates/fret-core/src/semantics.rs`:
  - roles: `SemanticsRole::*`
  - states: `SemanticsFlags { focused, captured, disabled, selected, expanded, checked }`
  - actions: `SemanticsActions { focus, invoke, set_value, set_text_selection }`
  - relations: `labelled_by`, `described_by`, `controls`
  - composite widgets: `active_descendant`
  - collections/virtualization: `pos_in_set`, `set_size`
- Validation exists but is currently limited to UTF-8 text range correctness
  (`SemanticsNode::validate` / `SemanticsSnapshot::validate`).

### 2.2 Snapshot production models overlays + modal reachability

`crates/fret-ui/src/tree/ui_tree_semantics.rs` builds a `SemanticsSnapshot` that:

- enumerates visible roots in paint order,
- computes `barrier_root` (input-modal) and `focus_barrier_root` (focus-modal),
- traverses the UI tree in a stable-ish order,
- stamps per-node semantics gathered from `Widget::semantics` / declarative host hooks,
- derives certain relation edges (e.g. controller `controls` from `labelled_by` for a subset of role pairs),
- runs optional validation (`FRET_VALIDATE_SEMANTICS_*`).

### 2.3 AccessKit mapping is implemented with focused regression tests

`crates/fret-a11y-accesskit/src/lib.rs` converts `SemanticsSnapshot -> accesskit::TreeUpdate` and:

- gates reachability by `barrier_root` (only roots at-or-above the barrier are exposed),
- maps roles and state (disabled/selected/expanded/checked),
- maps relations (`labelled_by`, `described_by`, `controls`) for reachable nodes,
- maps collection metadata (`pos_in_set`, `set_size`) to AccessKit,
- implements `active_descendant` emission with modal reachability suppression tests,
- implements action decoding for:
  - focus (`Action::Focus`)
  - invoke/click (`Action::Click`)
  - set value (`Action::SetValue`)
  - text selection (`Action::SetTextSelection`)
  - replace selected text (`Action::ReplaceSelectedText`) with composition blocking rules
- includes a synthetic `TextRun` node for `TextField` so selection works consistently.

### 2.4 Ecosystem conformance is already partially locked by tests

- Manual acceptance checklist: `docs/a11y-acceptance-checklist.md`
- Examples of automated checks in shadcn surfaces:
  - collection position metadata tests (command/select/context-menu)
  - active-descendant reachability tests (where present)

## 3) Gaps, risks, and follow-ups (excluding winit glue)

### 3.1 Schema coverage gaps (expected, but should be explicit)

Current schema intentionally does not model:

- rich “text document” semantics for code editors (large text, line/offset virtualization, etc.)
- table/grid semantics (e.g. data table / grid widgets)
- platform-native incremental/scroll actions (beyond focus/invoke/value/selection)

These are acceptable for v1 as long as we keep ADR-driven expansion and leave a clear audit trail.

### 3.2 Validation gaps

Current validation checks text ranges only. Missing checks that would catch drift earlier:

- `pos_in_set` / `set_size` invariants (`1 <= pos_in_set <= set_size`)
- relation edges referencing non-existent nodes (or nodes outside reachable roots)
- `active_descendant` referencing non-reachable nodes (runtime-side, not only mapping-side)
- duplicate nodes / parent cycles (the snapshot builder detects cycles only via a debug guard)

### 3.3 Text editing semantics: “value” policy for large buffers

ADR 0033 acknowledges that large text widgets may need excerpt policies, but we do not yet have:

- a portable contract for excerpt windows with exact selection/composition ranges,
- a consistent way to avoid copying huge strings into snapshots.

This is primarily a “code editor ecosystem” work item; track separately and only extend the
portable semantics surface via ADR + tests.

## 4) Design: internal AccessKit xplat adapter (winit-version-independent)

### 4.1 Requirements

- Must work with `winit 0.31.0-beta.2` without waiting for `accesskit_winit` releases.
- Must support multi-window (one adapter per OS window).
- Must be thread-safe: platform adapters may invoke handlers on arbitrary threads.
- Must integrate with Fret’s existing runner lifecycle:
  - create the adapter while the window is invisible (already done in `fret-launch`)
  - request redraw on initial tree request
  - update the tree once per frame when active
  - drain action requests and translate them into existing driver hooks

### 4.2 Proposed placement

Prefer implementing the adapter inside the runner glue layer (so the “backend deps” remain out of
portable crates):

Option A (recommended): keep it inside `crates/fret-runner-winit`

- Add a new module (e.g. `accessibility_accesskit_xplat.rs`).
- Keep `WinitAccessibility` as the runner-facing facade used by `fret-launch`.

Option B: introduce a dedicated crate (e.g. `crates/fret-a11y-accesskit-xplat`)

- Pros: reusable by non-winit runners in the future.
- Cons: more surface area; easier to accidentally broaden dependencies.

We should start with Option A and only extract if a second runner appears.

### 4.3 Adapter API surface (runner-facing)

The runner needs a minimal API that mirrors today’s stub:

- `new(event_loop, window)` (or `new(window)` if the xplat adapter doesn’t require the event loop)
- `process_event(window, WindowEvent)`:
  - `Focused(bool)` → `adapter.set_focus(bool)`
  - `Moved` / `SurfaceResized` (and any other geometry-affecting events) → `adapter.set_window_bounds(...)`
- `update_if_active(|| TreeUpdate)`
- `take_activation_request() -> bool`
- `is_active() -> bool`
- `drain_actions(&mut Vec<ActionRequest>)`

This matches the call sites in `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`.

### 4.4 Implementation approach (inspired by Blitz’s accesskit_xplat)

Use `RawWindowHandle` (via `winit::raw_window_handle::HasWindowHandle`) to initialize per-platform
AccessKit adapters without depending on winit’s adapter crate.

Reference-only (do not copy/paste blindly):

- `repo-ref/blitz/packages/accesskit_xplat`

We should implement our own glue with:

- a combined event handler (`InitialTreeRequested`, `AccessibilityDeactivated`, `ActionRequested`)
- an internal thread-safe queue for `ActionRequest`
- an atomic “activation requested” flag
- per-platform adapter implementation behind `cfg(target_os = ...)`

### 4.5 Dependency/version plan

Upgrade AccessKit to match the `repo-ref/accesskit` snapshot (currently `accesskit 0.24.0`) so the
platform crates line up cleanly.

Workspace changes (planned):

- `Cargo.toml`:
  - bump `accesskit = "0.24.0"` (from `0.22`)
- runner glue:
  - add per-platform deps (`accesskit_windows`, `accesskit_macos`, `accesskit_unix`, `accesskit_android`)
  - add `raw-window-handle` if needed explicitly (winit already uses it, but the adapter should not)

### 4.6 Regression/acceptance gates

Minimum gates for landing the xplat adapter:

- `cargo fmt`
- `cargo nextest run -p fret-a11y-accesskit`
- `cargo nextest run -p fret-runner-winit`
- `python3 tools/check_layering.py`
- Manual checks: `docs/a11y-acceptance-checklist.md` on at least Windows + one of macOS/Linux

## 5) Open questions (to resolve in v1)

- Which window events should be treated as “bounds changed” for AccessKit on each platform?
  (Blitz uses `Moved` and `SurfaceResized`; we likely want to include scale-factor changes too.)
- Should we keep `accessibility_accesskit_winit.rs` as a fallback for stable winit later, or delete
  it once xplat is in place?
- Do we want a feature flag that can disable the bridge at runtime (env var) for debugging?

## References and evidence anchors

- ADRs:
  - `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
  - `docs/adr/0073-active-descendant-and-composite-widget-semantics.md`
  - `docs/adr/0084-virtualized-accessibility-and-collection-semantics.md`
- Current implementation anchors:
  - schema: `crates/fret-core/src/semantics.rs`
  - snapshot: `crates/fret-ui/src/tree/ui_tree_semantics.rs`
  - mapping + decoding: `crates/fret-a11y-accesskit/src/lib.rs`
  - runner call sites: `crates/fret-launch/src/runner/desktop/runner/app_handler.rs`
  - manual checklist: `docs/a11y-acceptance-checklist.md`
  - reference inspiration: `repo-ref/blitz/packages/accesskit_xplat`

