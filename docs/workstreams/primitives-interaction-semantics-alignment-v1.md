# Primitives Interaction Semantics Alignment v1 (Radix + Base UI)

Status: Active (workstream note; not a contract)

This workstream audits and aligns **interaction semantics** for “primitives-level” controls by
porting **behavior outcomes** (state machines + invariants), not upstream event wiring.

Primary initial targets:

- `Select` (baseline: Radix)
- `Combobox` (baseline: Base UI, per shadcn/ui v4)

Non-goals:

- API compatibility with Radix/Base UI React components.
- Moving interaction policy into `crates/fret-ui` (mechanism layer).

Related docs:

- Radix mapping overview: `docs/radix-primitives-alignment.md`
- Runtime contract boundary: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Action hooks policy split: `docs/adr/0074-component-owned-interaction-policy-and-runtime-action-hooks.md`
- Progress tracker: `docs/workstreams/primitives-interaction-semantics-alignment-v1-matrix.md`
- Audit sheets:
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-select.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-combobox.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-dropdown-menu.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-context-menu.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-menubar.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-navigation-menu.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-tooltip.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-hover-card.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-popover.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-dialog.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-alert-dialog.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-sheet.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-drawer.md`
  - `docs/workstreams/primitives-interaction-semantics-alignment-v1-toast.md`

---

## Audit dimensions (what we track beyond “policy split”)

State-machine modeling and the Trigger/Listbox/Commit split are the *center of gravity*, but a full
primitive audit also needs a stable set of outcome dimensions that match Fret’s architecture:

- **Focus**: trap/restore/tab order outcomes; reason-aware close restoration.
- **Dismiss**: escape/outside press/focus out/scroll dismissal semantics.
- **Pointer**: misclick guards, modal barriers vs click-through, capture/hover intent where relevant.
- **Keys**: keyboard nav + typeahead; “open keys” vs “commit keys”.
- **A11y**: semantics roles and active-descendant vs roving mapping (AccessKit).
- **Placement/size**: anchored placement, collision, max-height/width clamping.
- **Time**: delays are `Duration` and represent a named semantic (hover intent, guard windows, etc.).

The matrix tracks these dimensions explicitly so we do not accidentally “fix a recipe” while
leaving the underlying primitive semantics unstable.

---

## Why this exists

We currently have a mix of:

- `ecosystem/fret-ui-kit` primitives that are explicitly Radix-shaped (e.g. `primitives::select`)
- shadcn “recipes” that embed meaningful state machines directly (e.g. `fret-ui-shadcn::combobox`)

That’s workable short-term, but it creates long-term churn:

- policies drift across components,
- “anti-misclick” fixes land ad-hoc in recipes,
- tests end up gating *skins* rather than *semantics*.

This workstream standardizes a repeatable approach:

1. Define the **outcome/state machine** and invariants.
2. Split the state machine into a few **composable policies**.
3. Implement those policies in `ecosystem/fret-ui-kit::primitives`.
4. Keep `ecosystem/fret-ui-shadcn` as composition + styling only.

---

## Layering: where things belong

- `crates/fret-ui` (mechanism): overlays substrate, focus scopes, outside-press observation, key
  routing, semantics tree.
- `ecosystem/fret-ui-kit` (policy/headless): state machines, policy knobs, reusable handlers, action
  hook wiring, overlay request presets.
- `ecosystem/fret-ui-shadcn` (recipes): shadcn naming + default chrome/layout + composition.

If a behavior is “dismiss rules, focus restore, hover intent, misclick guards, keyboard nav,
typeahead, scroll-into-view”, it belongs in `fret-ui-kit` unless it truly requires new mechanism.

---

## Sources of truth (local pinned snapshots)

shadcn/ui v4 (New York v4 registry):

- Select recipe: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/select.tsx`
- Combobox recipe: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/combobox.tsx` (imports `@base-ui/react`)

Radix primitives (for interaction semantics):

- `repo-ref/primitives/packages/react/select/src/*`
- `repo-ref/primitives/packages/react/dismissable-layer/src/*`
- `repo-ref/primitives/packages/react/focus-scope/src/*`

Base UI (for combobox semantics and misclick heuristics):

- `repo-ref/base-ui/packages/*` (exact module depends on the pinned Base UI revision)

---

## Modeling rule: explicit outcomes, not event transcription

When auditing a primitive, model it as:

- **State**: `open`, `query`, `value`, `active/highlight`, “pending commit”, “guard armed”, etc.
- **Events**: press, pointer down/up, key down, focus in/out, outside press, timer fired.
- **Reasons**: `OpenChangeReason`, `CloseReason`, “cancel open”, “selection committed”, etc.
- **Invariants** (gated by tests): “value-change fires only on real changes”, “do not commit on
  the same click that opened”, “typeahead does not change selection while closed unless configured”…

The goal is a portable state machine that fits Fret’s non-DOM model (overlay layers, semantics tree,
runner-owned input dispatch).

---

## Policy split (template)

For complex overlay listbox controls (Select/Combobox), split into 3 policies:

1. **TriggerPolicy**
   - What inputs open/close? (mouse, touch, keyboard, focus, typing)
   - Produces `OpenChangeReason` (e.g. `TriggerPress`, `EscapeKey`).

2. **ListboxPolicy**
   - Active/highlight, roving vs active-descendant, typeahead, scroll-into-view.
   - Owns “how to navigate” but not “when to commit”.

3. **SelectionCommitPolicy**
   - When does highlight become value? (enter, click, blur, “on close”, etc.)
   - Anti-misclick guards (pointer-up suppression, mouseup gate).
   - Defines “commit once vs multiple” and “close after commit”.

This structure lets us **absorb Radix + Base UI outcomes** without branching into two
implementations.

---

## Select (baseline: Radix; optional Base UI anti-misclick)

Upstream baseline:

- shadcn/ui v4 Select is Radix (`repo-ref/ui/.../select.tsx`).

Current Fret status:

- Mechanism wiring + a lot of policy already lives in `ecosystem/fret-ui-kit/src/primitives/select.rs`.
- shadcn recipe composes it: `ecosystem/fret-ui-shadcn/src/select.rs`.

What we keep as the default outcome:

- Radix pointer-up suppression when opened by mouse `pointerdown` (avoid committing on the
  “same click”).

What we make explicit as policy knobs (instead of ad-hoc recipe wiring):

- `pointer_up_guard` (Radix): when open was triggered by mouse `pointerdown`, suppress the matching
  `pointerup` if it is within a slop threshold.
- `mouse_up_selection_gate` (Base UI style): only allow “mouseup commit” paths when the item also
  saw a pointer-down (avoid a stray mouseup committing).
- `cancel_open_on_mouseup_outside` (Base UI style; optional): if trigger `pointerdown` opened,
  and the matching `pointerup` is outside trigger + popup bounds (±offset), close with
  `reason=CancelOpen`.

Implementation direction:

- Keep Radix semantics as default behavior.
- Add explicit configuration types in `fret-ui-kit::primitives::select` to host these knobs.
- Implement `cancel_open_on_mouseup_outside` using a one-shot pointer region in the overlay layer
  (no “document listeners”; use Fret overlay routing).

---

## Combobox (baseline: Base UI)

Upstream baseline:

- shadcn/ui v4 Combobox imports `@base-ui/react` (`repo-ref/ui/.../combobox.tsx`).

Current Fret status:

- `ecosystem/fret-ui-shadcn/src/combobox.rs` contains meaningful state machine logic:
  open reasons, “open change complete” events, and value-change gating.

Goal:

- Move the core state machine down to `ecosystem/fret-ui-kit/src/primitives/combobox.rs` so the
  recipe becomes “Input + Popover + Command layout/styling”.

State machine outline to standardize (Base UI shaped):

- `open` changes driven by: trigger press, input focus, typing, `ArrowDown`, etc.
- `query` (input text) drives filtering but is not `value`.
- `activeIndex`/highlight driven by keyboard + pointer hover.
- `Enter` commits highlighted item; pointer click commits clicked item.
- `onValueChange` fires only when the value actually changes.
- Close handling is reason-aware: `OutsidePress`, `EscapeKey`, `FocusOut`, `ItemPress`, …
- Focus restore is driven by `CloseReason → FocusRestorePolicy` (configurable).

---

## Timing rule: delays must be semantic

For “defensive” interaction gates (misclick, hover intent, grace corridors):

- Use `Duration` for public policy configuration.
- Avoid APIs that look like `*_ms` but are implemented as ticks/frames.

---

## Testing and regression protection

Per the `fret-shadcn-source-alignment` workflow, protect semantics with:

1. **Targeted Rust unit tests** in `ecosystem/fret-ui-kit/src/primitives/{select,combobox}.rs`
   for state-machine invariants (fast and stable).
2. **shadcn-web goldens** when the mismatch is a style/layout outcome.
3. **`fretboard diag` scripts** for multi-step interactive timelines (state machines).
   - Ensure stable `test_id` targets in shadcn recipes.

---

## Deliverables (v1)

- Policy knobs for Select (explicit config surface, Radix default).
- A new Combobox primitive module in `fret-ui-kit` (state machine + policy surface).
- Recipe refactor: `fret-ui-shadcn::combobox` calls the primitive module rather than embedding the
  semantics.
- A small set of unit tests + at least one scripted interaction repro per primitive.
