# Control Chrome Normalization Audit v1

Status: Active (workstream note; not a contract)

This workstream audits and migrates pressable-based “control chrome” composition patterns to a
single, stable normalization rule-set so we do not regress into “hit/visual separation”.

Specifically, we want to prevent cases where:

- an outer `Pressable` participates in layout (e.g. `flex-1`, `w-full`, explicit `w/h`, grid
  stretch), so its resolved bounds grow, but
- the inner node responsible for background/corner radii/border/shadow (“chrome”) remains
  shrink-wrapped (`Auto`), causing visible misalignment (background not filling hit box).

Primary target surface:

- `ecosystem/fret-ui-kit/src/declarative/chrome.rs` (`control_chrome_pressable_with_id_props`)

Related docs:

- Runtime contract boundary: `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- Tailwind semantics alignment: `docs/tailwind-semantics-alignment.md`

Trackers:

- TODO + audit sheet: `docs/workstreams/control-chrome-normalization-audit-v1/control-chrome-normalization-audit-v1-todo.md`
- Milestones: `docs/workstreams/control-chrome-normalization-audit-v1/control-chrome-normalization-audit-v1-milestones.md`

---

## Goals

1. Define the **mechanism-level invariant** for “control chrome” sizing:
   - If the outer pressable obtains a definite box from layout, the inner chrome must fill it
     (both axes).
2. Ensure min/max + padding/border behave like **`box-sizing: border-box`**:
   - The pressable drives the outer box size.
   - The chrome node enforces min/max constraints on its *content box*, after padding/border.
3. Make the rule reusable and hard to bypass accidentally:
   - Prefer a single helper (`control_chrome_*`) rather than per-component ad-hoc “set Fill”.
4. Lock the invariant with **unit tests** (contract-level) and a small set of **component
   regression gates** (diagnostic/visual).

---

## Non-goals

- Changing `crates/fret-ui` mechanism-layer behavior (layout engine, focus ring painting, etc.).
- Enforcing a single visual style (tokens, radius, shadow) across ecosystems.
- Achieving API compatibility with web libraries; we only care about behavior outcomes.

---

## Layering and ownership

- `crates/fret-ui` (mechanism): pressable input, focus ring plumbing, layout engine.
- `ecosystem/fret-ui-kit` (policy/headless + composition helpers): normalization rules and the
  canonical “Pressable → Chrome → Content” structure.
- `ecosystem/*` (recipes/components): should use `control_chrome_*` or fully replicate its
  normalization logic when intentionally deviating.

The core invariant belongs in `fret-ui-kit` so components do not need to rediscover it.

---

## Proposed contract (v1)

For the helper:

`control_chrome_pressable_with_id_props(cx, f) -> AnyElement`

We treat the following as normative behavior:

- Outer pressable:
  - `layout.overflow = Visible` (focus rings can paint outside).
- Inner chrome container:
  - `layout.overflow = Clip` (rounded corners/borders mask content).
- Sizing normalization:
  - If the pressable has an explicit size in an axis (`Px` or `Fill`), chrome must be `Fill` in
    that axis.
  - If the pressable participates as a “fill remaining space” item (e.g. `flex.grow > 0`),
    chrome must be `Fill` in both axes.
  - If the pressable provides min/max constraints, chrome min/max must be reduced by the chrome
    padding + border so the overall border-box dimensions match the pressable’s outer box.

The TODO tracker defines the test matrix used to lock these rules.

