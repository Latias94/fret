# Model-Centered Widget Contract Audit

Status: updated after narrow text bridge landing plus checkbox/switch/toggle source alignment
Last updated: 2026-03-08

Related:

- V2 golden path: `docs/workstreams/action-first-authoring-fearless-refactor-v1/V2_GOLDEN_PATH.md`
- Teaching-surface inventory: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TEACHING_SURFACE_LOCAL_STATE_INVENTORY.md`
- TODO: `docs/workstreams/action-first-authoring-fearless-refactor-v1/TODO.md`
- Milestones: `docs/workstreams/action-first-authoring-fearless-refactor-v1/MILESTONES.md`

---

## Scope

This note audits the remaining `Model<T>`-heavy widget contracts that still show up in
`apps/fret-ui-gallery` after the post-v1 default-path convergence work.

The goal is to separate three cases:

1. **true contract pressure**: the widget public surface is still genuinely model-centered,
2. **authoring choice**: the widget already has an uncontrolled or internally-scoped path,
3. **runtime/API exposure**: the model is intentionally the outward synchronization surface.

---


## Audit matrix

| Surface / widget class | Upstream public surface | Current Fret public surface | Audit status | Remaining gap | Next audit move |
| --- | --- | --- | --- | --- | --- |
| Text value widgets (`Input`, `Textarea`) | Prop-driven text value + callback | Narrow bridge via `IntoTextValueModel` (`Input::new(&local_text)`) while keeping model-backed internals | Narrowed; no longer a default-path blocker | Internal storage is still model-backed by design | Keep as-is; only revisit if another text-specific regression appears |
| Disclosure / overlay roots (`Collapsible`, `Popover`, `Dialog`, `AlertDialog`) | Controlled + uncontrolled (`open` / `defaultOpen`) | Controlled model path plus existing uncontrolled constructors | Mostly authoring-choice pressure, not a missing runtime primitive | Snippets can still overuse controlled models out of habit | Continue snippet cleanup toward uncontrolled paths where external sync is unnecessary |
| `ButtonGroup` composition hosts | Composition wrapper around child contracts | Same | Not a root blocker | Nested child widgets still determine most `Model<T>` pressure | Do not prioritize `ButtonGroup` itself unless nested contracts are already cleaned up |
| `Checkbox` | Prop-driven discrete state (`checked` / `defaultChecked` + callback) | Model-backed paths plus `Checkbox::from_checked(...)` / `from_checked_state(...)` + `action(...)` / `action_payload(...)` | Narrowed; shared label/control parity landed | No remaining default-path parity gap in the discrete checkbox contract | Keep stable; only revisit if a more specific checkbox regression appears |
| `Switch` | Prop-driven discrete state (`checked` / `defaultChecked` + callback) | Model-backed paths plus `Switch::from_checked(...)` + `action(...)` / `action_payload(...)` | Narrowed; shared label/control parity landed | No remaining default-path parity gap in the discrete switch contract | Keep stable; only revisit if a more specific switch regression appears |
| `Toggle` | Prop-driven discrete state (`pressed` / `defaultPressed` + callback) | Model-backed paths plus `Toggle::from_pressed(...)` + `action(...)` / `action_payload(...)` while keeping uncontrolled `default_pressed` | Narrowed; shared label/control parity landed | No remaining default-path parity gap in the discrete toggle contract | Keep stable; only revisit if a more specific toggle regression appears |
| Intentional outward-sync widgets (shared filters, app-wide settings, complex selection) | Shared/runtime-owned state | Explicit `Model<T>` surface | Intentionally model-backed | None, as long as the shared boundary is explicit | Keep as-is |


---

## Findings

### 1) Text value widgets were the last real authoring-pressure hotspot

Representative evidence:

- `ecosystem/fret-ui-shadcn/src/input.rs`
- `ecosystem/fret-ui-shadcn/src/textarea.rs`
- `ecosystem/fret-ui-shadcn/src/text_value_model.rs`
- `ecosystem/fret/src/view.rs`

Current shape:

- internal storage still remains `Model<String>`
- public constructors now accept a narrow `IntoTextValueModel` bridge
- `fret::view::LocalState<String>` now implements that bridge, so post-v1 views can call
  `Input::new(&local_text)` / `Textarea::new(&local_text)` directly

Assessment:

- This was a **real** contract boundary, not just a demo habit.
- The narrow bridge removes the biggest remaining `Model<String>` authoring cliff without adding a
  broad wave of per-widget helpers.
- Text widgets still stay model-backed internally, so IME/focus semantics and existing widget
  internals remain unchanged.

Examples affected:

- `apps/fret-ui-gallery/src/ui/snippets/card/demo.rs`
- `apps/fret-ui-gallery/src/ui/snippets/button_group/input.rs`
- `apps/fret-ui-gallery/src/ui/snippets/button_group/popover.rs`

### 2) Disclosure / overlay roots are not all hard model blockers

Representative evidence:

- `ecosystem/fret-ui-shadcn/src/collapsible_primitives.rs`
- `ecosystem/fret-ui-shadcn/src/popover.rs`
- `ecosystem/fret-ui-shadcn/src/dialog.rs`
- `ecosystem/fret-ui-shadcn/src/alert_dialog.rs`

Current shape:

- controlled path: `open(Model<bool>)` or `new(open: Model<bool>)`
- uncontrolled path already exists via `default_open(...)` or `new_controllable(cx, None, default)`

Assessment:

- Many of these widgets are **not** blocked on a new runtime primitive.
- When the snippet does not need external open-state synchronization, the gallery should prefer the
  uncontrolled path instead of allocating a local model just to satisfy a controlled constructor.
- This means some current `Model<bool>` usage in gallery snippets is an **authoring choice**, not a
  missing v2 capability.

Concrete cleanup landed:

- `apps/fret-ui-gallery/src/ui/snippets/collapsible/demo.rs` now uses
  `Collapsible::new().default_open(false)` instead of `use_controllable_model(...).model()`.

### 3) `ButtonGroup` is mostly a composition host, not the root cause

Representative evidence:

- `ecosystem/fret-ui-shadcn/src/button_group.rs`
- `apps/fret-ui-gallery/src/ui/snippets/button_group/input.rs`
- `apps/fret-ui-gallery/src/ui/snippets/button_group/popover.rs`

Assessment:

- `ButtonGroup` itself is not the main model-centered contract in these snippets.
- The `Model<T>` pressure usually comes from nested children such as `Input`, `Textarea`,
  `Popover`, or controlled selection widgets.
- This matters for prioritization: rewriting `ButtonGroup` would not materially reduce authoring
  noise if the nested child contracts stay model-centered.

### 4) Checkbox pressure narrowed after source alignment

Representative evidence:

- `F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/registry/new-york-v4/ui/checkbox.tsx`
- `F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/registry/new-york-v4/examples/checkbox-demo.tsx`
- `ecosystem/fret-ui-shadcn/src/checkbox.rs`
- `apps/fret-cookbook/examples/simple_todo_v2_target.rs`

Current shape:

- upstream shadcn checkbox is prop-driven (`checked` / `defaultChecked` + event callback), not model-driven
- Fret checkbox still keeps model-backed constructors for the existing controlled/uncontrolled paths
- a narrow source-aligned snapshot path now exists via `Checkbox::from_checked(...)` / `from_checked_state(...)`
- the checkbox recipe now also exposes `action(...)` / `action_payload(...)`, so action-first rows can render a checkbox from plain data without allocating a per-row `Model<bool>`

Assessment:

- This was a **real** contract pressure point for todo-like local collections, because the old model-only surface pushed authors back toward nested row models or surrogate buttons.
- The new snapshot path removes that specific cliff without broadening the framework into a generic `IntoModel<T>` story for every widget.
- The shared label/control parity follow-up has now landed: action-only `control_id` label activation mirrors the model-backed path, including typed action dispatch and payload forwarding alongside state toggles when applicable.

### 5) Switch pressure also narrowed after source alignment

Representative evidence:

- `F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/registry/new-york-v4/ui/switch.tsx`
- `F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/registry/new-york-v4/examples/switch-demo.tsx`
- `ecosystem/fret-ui-shadcn/src/switch.rs`
- `apps/fret-cookbook/examples/commands_keymap_basics.rs`

Current shape:

- upstream shadcn switch is prop-driven (`checked` / `defaultChecked` + event callback), not model-driven
- Fret switch still keeps model-backed constructors for the existing controlled/uncontrolled paths
- a narrow source-aligned snapshot path now exists via `Switch::from_checked(...)`
- the switch recipe now also exposes `action(...)` / `action_payload(...)`, so action-first views can render a switch from plain data without allocating extra local models

Assessment:

- This was a smaller pressure point than checkbox, but it showed the same underlying contract drift: plain local state still had to cross a `Model<bool>` boundary even when the view already owned the data.
- `apps/fret-cookbook/examples/commands_keymap_basics.rs` now demonstrates the narrower path on both an interactive local toggle (`allow_command`) and a derived read-only indicator (`panel_open`).
- The shared label/control parity follow-up has now landed here too: `Switch::from_checked(...).action(...)` can participate in `control_id` / `Label::for_control(...)` without falling back to model-backed registration.

### 6) Toggle pressure also narrowed after source alignment

Representative evidence:

- `F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/registry/new-york-v4/ui/toggle.tsx`
- `F:/SourceCodes/Rust/fret/repo-ref/ui/apps/v4/registry/new-york-v4/examples/toggle-demo.tsx`
- `ecosystem/fret-ui-shadcn/src/toggle.rs`
- `apps/fret-cookbook/examples/toggle_basics.rs`

Current shape:

- upstream shadcn toggle is prop-driven (`pressed` / `defaultPressed` + event callback), not model-driven
- Fret toggle still keeps model-backed constructors plus the uncontrolled `default_pressed` path
- a narrow source-aligned snapshot path now exists via `Toggle::from_pressed(...)`
- the toggle recipe now also exposes `action(...)` / `action_payload(...)`, so action-first views can render a toggle from plain data without allocating a `Model<bool>`

Assessment:

- This was the third discrete-widget contract class with the same underlying drift: view-owned boolean state still had to cross a `Model<bool>` boundary even when the view already owned the data.
- `apps/fret-cookbook/examples/toggle_basics.rs` now demonstrates the narrower path on a simple view-local action-first surface.
- The shared label/control parity follow-up has now landed for toggle as well: snapshot/action toggles register command-backed control entries, so label activation mirrors the same press path as the model-backed contract.

### 7) `use_controllable_model(...)` remains a bridge, not the default story

Representative evidence:

- `ecosystem/fret-ui-kit/src/declarative/controllable_state.rs`

Assessment:

- This helper is still useful for stable controlled/uncontrolled composition inside declarative
  widgets.
- It should not become the recommended first-contact authoring surface for application views.
- Prefer `use_local*` for normal view-local state, and reserve `use_controllable_model(...)` for
  widget internals or cases where a component truly needs a `Model<T>` boundary.

---

## Recommended next steps

### Keep as-is for now

- outward-facing API/event models (carousel handles, avatar image models, similar sync surfaces)
- controlled widgets whose public contract is intentionally broader than a single text value

### Follow-up after the narrow text bridge

- use uncontrolled overlay/disclosure constructors when external synchronization is unnecessary
- avoid storing `Model<bool>` in gallery snippet state just because a controlled constructor exists

### What this changes in prioritization

The narrow text bridge is now landed, so the next improvements should stay focused on:

- continued gallery/snippet cleanup where uncontrolled overlay paths already exist,
- explicit documentation that text widgets are no longer a default-path blocker,
- treating checkbox/switch/toggle parity as closed for the default path and resisting helper growth unless a new discrete-widget regression appears,
- resisting new per-widget helper growth unless another contract class proves equally common.

---

## Landable follow-up options

1. Continue gallery cleanup for snippets that only use `Model<bool>` as a convenience wrapper around
   already-uncontrolled roots.
2. Keep documenting the difference between ?default teaching surface? and ?reference/controlled?
   contract surface so migration pressure stays honest.
3. Revisit wider contract work only if another widget class shows the same pressure density that
   text-value widgets previously had; for the current discrete widgets, the next job is shared
   action-only label/control parity rather than broad helper/macro expansion.
