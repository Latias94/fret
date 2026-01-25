# ADR 0066: `fret-ui` Runtime Contract Surface (Tailwind primitives + shadcn components)

Status: Accepted

## Context

Fret’s component ecosystem targets “Tailwind-like primitives + shadcn-like recipes”, but Fret is not
a DOM/CSS runtime. To avoid long-term drift and repeated rewrites, we must **fix the runtime contract
surface early**, before scaling `fret-components-*`.

In practice, the biggest architectural risk is allowing shadcn/Radix “policy” (dismissal rules,
focus trap/restore, hover intent delays, per-component default paddings/row heights, etc.) to leak
into `crates/fret-ui`. Once that happens, every new component expands the runtime surface area and
the runtime becomes a “component junk drawer”.

This ADR defines:

- what `crates/fret-ui` is responsible for (stable contracts),
- what is explicitly *out of scope* for the runtime (must live in `fret-components-*`),
- which upstream sources are authoritative for each contract,
- how we compare Fret’s contract surface to GPUI patterns.

## Authoritative references (by contract)

When adopting a behavior/contract, we **port outcomes**, not implementations. Every runtime contract
must name at least one authoritative upstream reference.

- **Keyboard/focus interaction**: WAI-ARIA Authoring Practices (APG) (see `docs/reference-stack-ui-behavior.md`)
- **Overlay policy outcomes** (dismissal, focus restore/trap, nested overlays): Radix UI Primitives (via shadcn/ui)
  - Pinned local reference: Radix UI Primitives (upstream: <https://github.com/radix-ui/primitives>; see `docs/repo-ref.md`), `repo-ref/ui`
- **Placement algorithms** (flip/shift/size/offset/arrow): Floating UI
  - Pinned local reference: `repo-ref/floating-ui`
- **Virtualization contract vocabulary** (items/range/measurement/scroll-to): virtualizer (Rust)
  - Pinned local reference: `repo-ref/virtualizer`
- **Declarative layout solver**: Taffy (implementation engine)
  - Semantics target: CSS Flexbox/Grid + Tailwind layout primitives vocabulary
  - Pinned local reference for Tailwind semantics: `repo-ref/tailwindcss`
- **Ergonomic patterns / glue reference**: GPUI + gpui-component
  - Pinned local reference: `repo-ref/gpui-component`

## Decision

### 1) `crates/fret-ui` is a runtime substrate, not a component library

`crates/fret-ui` owns **hard-to-change runtime semantics** that components need to compose
deterministically:

- event routing (pointer/keyboard), hit testing, focus + capture, focus-visible,
- render transforms that keep paint + hit testing + event coordinates consistent (ADR 0083),
- deterministic multi-root layering (overlays/barriers) and scene operations,
- declarative element tree hosting (ADR 0028 / ADR 0039) and layout vocabulary (`LayoutStyle`),
- scroll + virtualization contracts (handles + algorithms),
- overlay placement solver (ADR 0064),
- editing engines that need platform hooks (IME/caret/selection geometry).

`crates/fret-ui` does **not** own shadcn/Radix policy surfaces:

- Popover/Dialog/Tooltip/HoverCard/Menu/Toast/Command behaviors,
- dismissal rules, focus trap/restore policies,
- per-component default values (row heights, padding, delays) beyond minimal deterministic defaults.

### 2) Stability tiers (how to “lock” contracts)

To keep the runtime usable but clean, we classify public runtime APIs as:

- **Stable**: relied on by `fret-components-*`; changes require ADR update + migration plan.
- **Experimental**: allowed to churn; must be behind a feature flag or clearly marked.
- **Compatibility**: legacy retained widgets (temporary); must be feature-gated and delete-planned.

### 3) Stable runtime contracts (P0) — contract table

This is the minimum contract set `fret-ui-kit` can depend on long-term.

| Contract | `fret-ui` provides (mechanism) | Invariants (must hold) | Reference (authoritative) | Related ADRs / docs |
| --- | --- | --- | --- | --- |
| Input routing + hit testing | deterministic event routing, hit testing, pointer transparency | same inputs → same hit results and routing, including across overlay roots | APG (outcomes), internal determinism requirement | ADR 0005, ADR 0020, `docs/reference-stack-ui-behavior.md` |
| Hover tracking + geometry queries | `HoverRegion` + `elements::bounds_for_element` | hover signals are deterministic and driven by hover-capable pointers (mouse/pen); touch pointers must not perturb hover state; last-frame bounds are stable enough for anchored overlay policies | APG (pointer outcomes), Floating UI (anchored placement vocabulary) | ADR 0066, `docs/reference-stack-ui-behavior.md` |
| Focus + capture + focus-visible + traversal | focus/capture primitives and focus-visible detection + `focus.next`/`focus.previous` | background cannot steal focus under modal barrier; focus/capture/traversal routing deterministic | APG (outcomes), Radix (policy target at component layer) | ADR 0020, ADR 0061, ADR 0068 |
| Multi-root layers substrate | overlay roots, barrier installation, deterministic root ordering | hit testing + rendering order match root stack; barrier enforces inert background | Radix (policy target), Flutter/WPF-style overlay barrier model | ADR 0011 |
| Non-modal outside press observer | opt-in outside-press dispatch for topmost non-modal overlay | click-through dismissal without modal barrier; does not break normal hit-tested routing | Radix DismissableLayer (outcomes) | ADR 0069 |
| Placement solver | pure placement algorithm API | deterministic placement for same inputs; no component policy in runtime | Floating UI | ADR 0064 |
| Declarative authoring | element tree + keyed state + model observation | stable IDs, predictable reuse, testable state reuse | GPUI-style authoring model | ADR 0028, ADR 0039, ADR 0051 |
| Frame scheduling | one-shot frame requests + RAF requests + refcounted continuous leases | event-driven by default; continuous frames are explicit and scoped; coalesced per window per tick | GPUI/Zed `Window::refresh()` mental model | ADR 0034 |
| Layout vocabulary | `LayoutStyle` + Flex/Grid semantics (Taffy-backed) | CSS/Tailwind-like defaults; no per-component hidden defaults in runtime | CSS + Tailwind semantics, Taffy engine | ADR 0057, ADR 0062, ADR 0035 |
| Scroll contract | scroll handles + strategies | scroll-to behavior is deterministic; components can build scrollbars/policies | GPUI handle patterns | ADR 0042 |
| Virtualization contract | variable-size metrics + visible range computation + scroll-to-item | supports measured heights, stable keys, overscan; deterministic | virtualizer (primary), GPUI (engineering ref) | ADR 0070 |
| Text input/IME engine contract | IME plumbing hooks, editing engine state/commands, caret geometry | caret/selection geometry query stable; IME preedit/commit deterministic | platform IME + APG text expectations | ADR 0012, ADR 0044, ADR 0045, ADR 0046 |
| Semantics tree | semantics tree mechanism + overlay root semantics boundaries | modal barrier hides/inerts background semantics; deterministic snapshot | WAI-ARIA + platform bridges | ADR 0033 |

The table above intentionally **does not include any shadcn/Radix overlay policies**. Those belong
in `fret-components-*` and must be locked there with behavior tests.

#### 3.1 Overlay substrate contract (Stable)

The runtime provides **overlay mechanisms**, not overlay policies.

**Runtime responsibilities (mechanism):**

- per-window **root stack**: 1 base root + 0..N overlay roots with a deterministic order,
- a **barrier root** mechanism (“modal barrier”) that can make background roots inert,
- deterministic hit testing and event routing across roots,
- an **outside press observer pass** for the topmost opt-in non-modal overlay (click-through),
- optional pointer transparency for overlay content (e.g. tooltip-like overlays),
- semantics snapshot boundaries across roots (a11y-friendly multi-root).

**Component responsibilities (policy):**

- dismissal rules (Escape, click outside, focus loss),
- focus trap/restore policies,
- hover intent/delay policies (Tooltip/HoverCard),
- nested overlay stack rules (menu → submenu, dialog → popover, etc.).

**Modal barrier recommendation (Stable semantics):**

- Pointer: clicks outside modal content hit the barrier (not the background).
- Focus: while a barrier is installed, background nodes cannot become focus targets.
- Keyboard: background does not receive key events while inert.
- Semantics: background is hidden or inert in the semantics tree while barrier is active.
- Nesting: the topmost barrier defines the active interaction region; deeper layers remain inert.

This defines the *mechanism*; whether a click-outside closes is a component policy decision (Radix
target).

#### 3.2 Placement solver contract (Stable; arrow is P1)

The placement solver is a pure algorithm surface (ADR 0064). It must support, at minimum:

- flip + shift + size constraints + offset (P0),
- arrow positioning (P1; do not lock until renderer/shape semantics are stable).

Reference: Floating UI (`repo-ref/floating-ui`).

#### 3.3 Virtualization contract (Stable; virtualizer alignment)

We treat `virtualizer` as the primary **contract vocabulary** reference (not as a UI
implementation target).

**Concept mapping (TanStack → Fret):**

- `VirtualItem { key, index, start, end, size, lane }` → runtime-provided item geometry output
- `count` → list length
- `estimateSize(index)` → estimated row/column size
- `measureElement` → a runtime measurement write-back API (components call after layout/paint)
- `overscan` → a policy knob (default in components; supported by runtime computation)
- `getItemKey` → stable item keys (component supplies; runtime uses for caches)
- `rangeExtractor(range)` → policy hook (component supplies; runtime supports visible range output)
- `scrollToIndex/scrollToOffset` + `align` → runtime scroll handle strategies
- `scrollMargin` → runtime-supported offset origin shift (critical for headers/sticky rows)
- `gap` → spacing between items (runtime metric support)
- `lanes` → P1 (masonry/grid virtualization)

Primary reference:

- virtualizer README: `repo-ref/virtualizer/README.md`

Engineering reference:

- gpui-component virtual list patterns: `repo-ref/gpui-component/crates/ui/src/virtual_list.rs`

#### 3.4 Frame scheduling contract (Stable; GPUI-aligned)

The runtime must support both "draw only when dirty" and "continuous frames while active" without
introducing component-specific toggles.

Mechanism (runtime/host provided):

- `request_frame(window)`: schedule exactly one draw for the window (coalesced per tick).
- `request_animation_frame(window)`: request a draw at the runner `frame_interval` cadence.
- `begin_continuous_frames(window) -> ContinuousFrames`: returns a refcounted RAII lease; while any
  lease is alive, the window continues to request animation frames; dropping the last lease returns
  the window to event-driven scheduling.

Reference: GPUI/Zed `Window::refresh()` and its "dirty window invalidator" model (`repo-ref/zed`).

#### 3.5 Layout contract (Stable; CSS/Tailwind semantics)

Runtime layout semantics must match the Tailwind/CSS mental model:

- default cross-axis alignment behaves like CSS (`align-items: stretch`) unless explicitly set,
- layout-affecting primitives are only those in ADR 0057/0062 (no widget-local hidden layout rules),
- Taffy is the implementation engine; semantics are validated by tests against our ADRs and
  Tailwind vocabulary usage in `fret-components-*`.

### 4) Declarative-only authoring (no retained widget primitives)

The component ecosystem is declarative-first (ADR 0028 / ADR 0039). We treat retained widgets
(`UiTree` + `Widget`) as an internal hosting mechanism, not a supported authoring model.

Rules:

- `fret-components-*` APIs should prefer `ElementContext`/`AnyElement` authoring.
- New reusable UI "primitives" should be expressed as element kinds, handles, or pure algorithms
  (not as exported retained widgets).

## Comparison: GPUI-style contract surface

GPUI provides a useful “contract vs component” separation reference:

- **GPUI runtime (`gpui`)** provides substrate contracts: `ScrollHandle`, `ScrollStrategy`,
  `FocusHandle`, anchored/deferred primitives, event routing/hit testing.
- **gpui-component `repo-ref/gpui-component/crates/ui`** builds policy-heavy components on top: `Popover`, `Dialog`,
  `VirtualList`, scrollbars, Tailwind-like `StyledExt`, etc.

Fret’s target mapping:

- `gpui` (runtime substrate) ≈ `crates/fret-ui`
- gpui-component `repo-ref/gpui-component/crates/ui` (policy + recipes) ≈ `ecosystem/fret-ui-kit` (infra) +
  `ecosystem/fret-ui-shadcn` (taxonomy + recipes)

Key principle:

- port the *contracts and outcomes* (APG/Radix/Floating UI), not the React/GPUI implementations.

## Consequences

- `fret-components-*` can grow aggressively without forcing runtime churn.
- shadcn parity can be locked with tests at the component layer, while runtime stays minimal.
- migration work becomes a deliberate, staged process (not a perpetual refactor).

## Follow-ups (gates before scaling component surface)

- Add a “contract checklist” to new component proposals:
  - does it require a new runtime contract, or can it be built from existing contracts?
- Ensure every Stable contract has:
  - a small unit test in `fret-ui` (mechanism), and
  - a behavior contract test in `fret-components-*` (policy/outcome).

## Decision gates (what is fixed now)

This section records the concrete decisions we are locking *now* so that subsequent work can
proceed without re-litigating fundamentals.

### Gate A — Stability and versioning policy

- Any new public API in `crates/fret-ui` is **Experimental by default**.
- Promoting an API to **Stable** requires:
  - updating this ADR (including an authoritative reference),
  - adding/adjusting runtime mechanism tests in `fret-ui`,
  - adding/adjusting behavior tests in `fret-components-*` (where policy lives),
  - documenting a migration plan if any API is changed/removed.
- Experimental APIs must be behind a feature flag or clearly marked as such in docs.

### Gate B — Overlay substrate API shape

- `fret-ui` exposes only the minimal **mechanism** needed to install/uninstall overlay roots and an
  optional modal barrier (ADR 0011).
- Overlay *policies* (dismissal, focus trap/restore, hover intent, nested overlay state machines)
  must live in `fret-components-*`.

### Gate C — Modal barrier semantics (Stable)

When a modal barrier is installed for a window:

- Pointer: clicks outside modal content hit the barrier, not the background.
- Focus: background nodes cannot become focus targets while the barrier is active.
- Keyboard: background does not receive key events while inert.
- Semantics: background is hidden or inert in the semantics tree while the barrier is active.
- Nesting: the topmost barrier defines the active interaction region; deeper layers remain inert.
- Focus handoff: if focus is currently in the background when the barrier is installed, the runtime
  clears focus; the component policy must explicitly request focus inside the modal content.

Whether click-outside closes is strictly a component policy decision (Radix/shadcn target).

### Gate D — Virtualization P0/P1 boundary (virtualizer alignment)

- P0 (Stable): `scrollMargin`, `rangeExtractor`, `getItemKey`, `overscan` support, and `VirtualItem`
  geometry output compatible with `virtualizer`’s vocabulary.
- P1 (defer): masonry/lanes and any multi-lane virtualization contract.
- P1 (defer): `initialMeasurementsCache`-style seeding of measurements (allowed later, not locked now).

### Gate E — Scroll contract scope

- P0 (Stable): offset/content/viewport + deterministic `scrollToOffset` / `scrollToIndex` strategies.
- P1 (defer): “is scrolling” state and scroll-end detection as a formal runtime contract (these are
  primarily used for UI polish like scrollbar fade/show modes).

### Gate F — Layout default values and Tailwind semantics

- Runtime layout defaults must be CSS/Tailwind-like.
- The runtime must not introduce per-component hidden defaults (row heights, paddings, delays).
  Recipes and defaults belong in `fret-components-*`.

### Gate G — Text engine contracts (P0)

- P0 (Stable): caret/selection geometry queries and a minimal, deterministic text editing command set.
- Styling/chrome and visual defaults belong in `fret-components-*`.

### Gate H — Semantics tree overlay behavior (P0)

- P0 (Stable): modal barriers must hide/inert background semantics so assistive technologies do not
  “see through” modal overlays.
