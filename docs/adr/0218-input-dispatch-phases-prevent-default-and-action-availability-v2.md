# ADR 0218: Input Dispatch Phases, `prevent_default`, and Action Availability (v2)

Status: Proposed

## Implementation Status

This ADR is partially implemented.

Implemented:

- Dispatch phase enum: `crates/fret-runtime/src/input.rs` (`InputDispatchPhase::{Preview,Capture,Bubble}`).
- `prevent_default` plumbing: `crates/fret-runtime/src/input.rs` (`DefaultAction`, `DefaultActionSet`) and `crates/fret-ui/src/widget.rs` (`EventCx::{prevent_default,default_prevented}`).
- Observer pass contract: outside-press uses `Widget::event_observer` with `ObserverCx` (Preview phase) so routing state (focus/capture/propagation/default actions) cannot be mutated: `crates/fret-ui/src/widget.rs`, `crates/fret-ui/src/tree/dispatch.rs`.
- Default actions step (v1): `DefaultAction::FocusOnPointerDown` is applied by default during event dispatch and can be suppressed via `prevent_default`: `crates/fret-ui/src/tree/dispatch.rs`.
- Capture-phase dispatch (root → target): key down/up and pointer interactions (down/up/wheel/pinch/cancel, and move when buttons are pressed or capture is active): `crates/fret-ui/src/tree/dispatch.rs`.
- Per-layer pointer-move observation: when overlay policies opt in (e.g. Radix menu safe-hover corridors), pointer-move observers still run even when hit-tested pointer dispatch is suppressed by pointer occlusion: `crates/fret-ui/src/tree/dispatch.rs`, `crates/fret-ui/src/declarative/host_widget/event/dismissible.rs`, `ecosystem/fret-ui-kit/src/window_overlays/render.rs`.
- Tests: `crates/fret-ui/src/tree/tests/prevent_default.rs`.
- Tests: `crates/fret-ui/src/tree/tests/dispatch_phase.rs`.
- Dispatch-path action availability query (retained `UiTree`):
  - `Widget::command_availability` returns `CommandAvailability::{NotHandled,Available,Blocked}`.
  - `UiTree::{command_availability,is_command_available}` provides a best-effort query suitable for gating UI surfaces.
  - Data-only runner bridge: `WindowCommandActionAvailabilityService` publishes per-window availability snapshots.

Not implemented yet / known gaps:

- Additional default actions beyond `FocusOnPointerDown` (intentionally deferred to keep v1 low risk).
- Capture-phase coverage is still incremental for non-pointer/key event families and may need to be expanded as the contract hardens.

## Context

Fret is a non-DOM UI runtime targeting editor-grade interaction workloads (multi-root overlays,
focus/capture, docking, viewports). The runtime substrate already provides:

- deterministic hit-test based input routing across layer roots (ADR 0011),
- pointer capture and focus routing (ADR 0020),
- an outside-press *observer pass* for click-through non-modal overlays (ADR 0069),
- component-owned interaction policy via action hooks (ADR 0074).

However, several ergonomics gaps remain when compared to GPUI-style patterns:

1) There is no **general** notion of `prevent_default` that can suppress a runtime default behavior
   while still allowing event propagation (DOM/GPUI-style "prevent default, keep bubbling").
2) Dispatch phases are not modeled as a first-class contract (GPUI `DispatchPhase::{Capture,Bubble}`),
   making it harder to express global pre-processing and deterministic cleanups.
3) There is no unified, queryable **action availability** API (GPUI `is_action_available`) to support:
   OS menus, command palette gating, shortcut help UI, and editor-grade "disabled but visible" UX.

These problems tend to cause large rewrites if decided late because they cut across:

- event routing and focus/capture primitives (`crates/fret-ui`),
- keymap/shortcut arbitration and command metadata (`crates/fret-runtime`, ADR 0021/0022/0023),
- overlay policy composition (`ecosystem/fret-ui-kit`),
- docking and viewport tooling arbitration (`ecosystem/fret-docking`, ADR 0072).

## Goals

- Provide a mechanism-only contract for:
  - explicit dispatch phases,
  - default behavior suppression (`prevent_default`),
  - action availability queries along a dispatch path.
- Keep all policy decisions (Radix/APG outcomes, overlay dismissal rules, menu navigation, etc.) in
  `ecosystem/*` crates (ADR 0066 / ADR 0074).
- Make the contract compatible with both:
  - the current retained `UiTree` substrate, and
  - a future "per-frame rebuilt declarative tree + cross-frame state in models" style (ADR 0005 /
    ADR 0028 direction), without forcing that migration.

## Non-Goals

- Implement a DOM-compatible event model (event targets, composed paths, etc.).
- Adopt Flutter-style gesture arenas as the primary abstraction.
- Encode design-system policies into `crates/fret-ui`.

## Current State (Summary)

- Input dispatch is primarily "hit-test then bubble to root", with pointer capture overriding hit-test.
- A dedicated observer path exists for outside-press click-through overlay dismissal:
  - `InputDispatchPhase::Preview` is used for observer dispatch, routed through `Widget::event_observer`.
  - invariants: observer dispatch must not mutate focus/capture/propagation/default actions (ADR 0069).
- Some default-behavior suppression exists but is *component-specific*:
  - `PressablePointerDownResult::{Continue,SkipDefault,...}` in `crates/fret-ui/src/action.rs`.
- Command availability exists only as a minimal runner-facing snapshot (undo/redo):
  - `WindowCommandAvailabilityService` in `crates/fret-runtime/src/window_command_availability.rs`.
- There is also a command-level enable/disable override seam for OS menu gating:
  - `WindowCommandEnabledService` (ADR 0173).

## Problems

### 1) Default behavior suppression is not general

Component authors can currently suppress defaults only through bespoke, per-component hooks (e.g.
pressable pointer-down), which:

- does not scale as new interaction primitives are introduced,
- forces policy-heavy crates to learn runtime implementation details,
- makes "prevent default but keep bubbling" hard to express consistently.

### 2) Phase is not an explicit contract

Some behaviors are easiest to implement deterministically with a capture-style phase:

- pre-processing that must run before target/bubble handlers (pressed state cleanup, modality/hover
  resets, etc.),
- consistent ordering for global glue (runner hooks, diagnostics, action gating).

Without an explicit phase contract, these behaviors tend to become ad-hoc runtime special cases.

### 3) There is no action availability query API

Editor-grade UX requires the system to answer questions like:

- "Should this menu item be enabled right now?"
- "Is this action available along the focused dispatch path?"
- "Does this keystroke have any available binding in the current focus context?"

Without a query API, consumers often resort to brittle heuristics or trial-dispatch with side
effects, both of which scale poorly.

## Decision

### 1) Introduce explicit dispatch phases (v2)

Define a stable phase contract for UI event dispatch:

```text
Preview (Observer)  ->  Capture  ->  Bubble  ->  DefaultActions
```

- **Preview** (a.k.a. Observer):
  - Used for click-through policies (outside press, hover transitions, diagnostics).
  - Must not mutate routing state (focus/capture/propagation). It may request redraw/invalidations.
  - Supersedes `InputDispatchPhase::Observer` (same invariant, clearer naming).
- **Capture**:
  - Runs from the dispatch root toward the target (root → leaf).
  - Intended for deterministic cleanup/pre-processing and for policies that must "see" events early.
  - Delivered via a dedicated widget hook (`Widget::event_capture`) so the default widget `event()`
    implementation continues to represent the Bubble phase only.
- **Bubble**:
  - Runs from target toward the dispatch root (leaf → root).
  - The primary phase for component interaction logic.
- **DefaultActions**:
  - A runtime-owned step that executes *mechanism-level* default behaviors unless prevented.

Phase must be observable in `EventCx` and in any hook host contexts used by ecosystem policy code.

### 2) Add `prevent_default` as a mechanism-level contract

Introduce `prevent_default` that is orthogonal to `stop_propagation`:

- `stop_propagation` controls whether subsequent handlers in the current phase/path run.
- `prevent_default` suppresses a specific runtime default behavior while allowing propagation.

`prevent_default` is keyed by a **default action** identifier (not a boolean). This avoids
accidentally preventing unrelated defaults as the system evolves.

#### Default actions (v1 scope)

Start with a conservative set of defaults that are hard-to-change and widely needed:

- `DefaultAction::FocusOnPointerDown`

Additional defaults (pressed/capture/activate/text selection) may be introduced later, but are
explicitly out of scope for v1 to keep the migration low risk.

Status note (implementation alignment): we intentionally keep the catalog minimal until a concrete,
cross-surface motivation exists. See `docs/workstreams/input-dispatch-v2-todo.md` (IDV2-def-006/007)
for the current boundary decision and the "defer by default" rule for new defaults.

### 3) Standardize an action availability query API

Add a stable mechanism to answer:

- "Is this `CommandId` available in the current focus context?"
- "Is this `CommandId` available for a specific focus handle / node?"

Key points:

- Availability is a **pure query** (no side effects).
- Query walks a dispatch path in the **most recent rendered dispatch tree snapshot** (similar in
  spirit to GPUI's `rendered_frame.dispatch_tree`).
- The mechanism must support runner integrations (OS menu validation) without requiring policy code
  to depend on `fret-ui` internals. This may require a data-only snapshot service, similar to
  `WindowInputContextService`.

#### Relationship to existing command gating

Fret already has multiple layers of command gating:

- **Keymap binding gating** (`when` expressions): determines whether a keystroke maps to a `CommandId`
  in the current window/input context (ADR 0022 / ADR 0021).
- **Per-command enabled overrides** (`WindowCommandEnabledService`): a data-only, window-scoped
  override intended for OS menu validation and shortcut consistency (ADR 0173).

This ADR introduces **dispatch-path availability**:

- **Action availability** answers "would this command be handled along the current dispatch path?"
  (focus chain / target chain), as a pure query.

Recommended composition for UI surfaces:

- A command is *enabled* if:
  1) it is available along the current dispatch path, AND
  2) it is not disabled by `WindowCommandEnabledService`, AND
  3) the surface-specific gating (e.g. `when` for keystrokes) allows it.

#### Suggested mechanism surface (retained `UiTree`)

For the retained runtime, action availability is expressed as a pure query method on widgets:

- `Widget::command_availability(&self, cx, command) -> CommandAvailability`
  - `NotHandled`: this node does not participate in availability for this command.
  - `Available`: this node (or its subtree policy) enables the command.
  - `Blocked`: this node intentionally prevents the command from "reaching" ancestors (used for
    modal barriers, focus traps, and other dispatch-scoping constructs).

The runtime provides a helper query:

- `UiTree::is_command_available(&mut self, app, command) -> bool`
  - Walks the focused bubble path (or modal barrier root when applicable),
  - Stops at the first `Available` / `Blocked`,
  - Returns `false` when no node claims availability.

#### Suggested runner integration (data-only snapshot)

Native runners need a synchronous, data-only view of command gating for OS menu validation.
Availability queries should therefore be *published* by the UI/app layer as a snapshot service:

- `WindowCommandActionAvailabilityService` (`HashMap<AppWindowId, HashMap<CommandId, bool>>`)

The snapshot is typically recomputed:

- for all commands when the command palette is open, and/or
- for the current menu bar command set during normal operation.

#### Command identity strategy

`CommandId` remains the canonical cross-surface identity (keymap, menus, command palette).
Type-safe wrappers may exist in ecosystem crates, but must lower to `CommandId` at the framework
boundary.

### 4) Ecosystem integration (how policy crates plug in)

This ADR does not move policy into the runtime. Instead, it clarifies how ecosystem crates should
compose the new mechanisms:

- `ecosystem/fret-ui-kit`:
  - overlay policies use Preview (Observer) for outside press without blocking hit-tested dispatch,
    and may use `prevent_default(FocusOnPointerDown)` to avoid focus stealing where appropriate.
  - shadcn/Radix recipes express their outcomes via action hooks (ADR 0074) + prevent-default where
    required by upstream behavior.
- `ecosystem/fret-docking`:
  - docking/viewport arbitration can depend on phase ordering guarantees (e.g. capture for cleanup,
    bubble for tool dispatch), instead of introducing bespoke runtime flags.
- runner/menus/command palette:
  - UI surfaces gate enable/disable via `is_action_available` (or a published snapshot), instead of
    inferring from app-owned state ad hoc.

## Migration Plan (Phased)

1) **Introduce phase + prevent-default types behind an unstable feature**
   - Keep current `InputDispatchPhase` working; add a v2 path in parallel.
2) **Implement `DefaultAction::FocusOnPointerDown`**
   - Move "focus-on-mousedown" from component-specific defaults into the runtime default action
     step, and allow suppression via `prevent_default`.
3) **Add capture + bubble dispatch for pointer interactions + key down/up**
   - Start with the smallest set of events that benefit from phase separation.
4) **Add availability query plumbing**
   - Build and store a dispatch-tree snapshot that supports `is_action_available(CommandId)`.
5) **Migrate ecosystem policy code**
   - Replace any bespoke "skip default focus" logic with `prevent_default`.
   - Adopt availability query for menu/command palette gating.
6) **Deprecate legacy phase naming**
   - Replace `InputDispatchPhase::{Normal,Observer}` with the v2 phase model.

## Alternatives Considered

### A) Keep current model + add more per-component "skip default" enums

- Pros: incremental, minimal refactor.
- Cons: grows runtime surface area indirectly, increases policy/runtime coupling, and does not solve
  availability queries.

### B) Adopt Flutter gesture arenas as the primary input abstraction

- Pros: powerful for complex gesture disambiguation.
- Cons: adds substantial conceptual and implementation complexity, and does not directly address
  command/action availability or OS menu gating.

### C) Iced-style event `Status::{Ignored,Captured}` only

- Pros: simple.
- Cons: too coarse for editor-grade composition; lacks `prevent_default` and explicit phase tools.

## Risks

- Phase expansion can increase dispatch cost if implemented naïvely; must avoid doubling all work in
  hot paths (pointer move, wheel). Phase should be applied selectively and optimized.
- Availability query must remain deterministic and side-effect-free; any accidental mutation during
  queries will become a debugging hazard.
- Over-eager default-action expansion can become a de facto policy layer; keep v1 minimal.

## Open Questions

- Should availability query be exposed via a data-only snapshot service for runners, or is it
  acceptable for runners to call into a `UiTree`-backed API directly on the UI thread?
- What is the minimal "command payload" story? (Prefer `CommandId` + structured data for cross-
  boundary uses; reserve `Any` payloads for internal-only actions.)
- How should command namespaces be standardized for third-party extensions? (Recommend reverse-domain
  prefixes and aliasing for renames.)

## References

- Runtime contract surface and policy boundary: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Component-owned policy via action hooks: `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md`
- Outside press observer: `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
- Focus + command routing: `docs/adr/0020-focus-and-command-routing.md`
- Keymap and command metadata: `docs/adr/0021-keymap-file-format.md`, `docs/adr/0022-when-expressions.md`, `docs/adr/0023-command-metadata-menus-and-palette.md`
- OS menu command gating seam: `docs/adr/0173-window-command-enabled-service.md`
- Plugin/panel boundary (namespacing motivation): `docs/adr/0016-plugin-and-panel-boundaries.md`
- GPUI reference (dispatch phase / prevent default / availability): `repo-ref/zed/crates/gpui/src/window.rs`
