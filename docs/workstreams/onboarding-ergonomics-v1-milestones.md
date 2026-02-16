# Onboarding Ergonomics + Ecosystem Interop — Milestones (v1)

Status: Draft

This plan is intentionally staged: docs + templates first, then runnable interop evidence, then
optional tooling/macros.

Design note:

- `docs/workstreams/onboarding-ergonomics-v1.md`

TODO tracker:

- `docs/workstreams/onboarding-ergonomics-v1-todo.md`

---

## Milestone 0 (M0): Docs-first onboarding ladder

Outcome:

- A new user can find a single doc that tells them “what to do in the first hour”.

Deliverables:

- “First hour” doc exists and is linked from a discoverable place (decision: `README.md` or `docs/README.md`).
- `docs/examples/todo-app-golden-path.md` clearly distinguishes:
  - “simple-todo baseline” (Model + MVU),
  - “best practice baseline” (selector + query).

Acceptance:

- A reviewer can follow the doc end-to-end without needing to open runner/kernel code.

---

## Milestone 1 (M1): Tiered templates

Outcome:

- Templates match the onboarding ladder (progressive disclosure).

Deliverables:

- `fretboard new simple-todo` scaffold exists.
- Existing `todo` scaffold remains the “best practice baseline”.

Acceptance:

- Both templates compile and run (native).
- `simple-todo` has no selector/query deps.
- Both templates teach keyed list rendering.

Evidence (current):

- `apps/fretboard/src/scaffold/templates.rs`
- `docs/first-hour.md` (template matrix + progressive disclosure ladder)
- `docs/examples/todo-app-golden-path.md` (baseline intent)
- `apps/fret-examples/src/todo_demo.rs` (baseline reference implementation)

---

## Milestone 2 (M2): Authoring density (composition macros)

Outcome:

- The `simple-todo` path feels “GPUI/iced-like”: composition is uniform and call-site adapter noise is reduced.

Deliverables:

- An iced-like `children![...]` macro exists for heterogeneous child lists.
- `simple-todo` template can be authored without repetitive `.into_element(cx)` for common components.
  - Either by `children![...]`, and/or by ecosystem constructors accepting `UiIntoElement` children.

Evidence (current):

- `ecosystem/fret-ui-kit/src/lib.rs` (`children!`)
- `ecosystem/fret-ui-kit/src/ui.rs` (`ui::*` layout constructors accept `UiIntoElement` children)
- `apps/fretboard/src/scaffold/templates.rs` (`simple-todo` template)
- `apps/fret-examples/src/assets_demo.rs` (example: `ui::children!` replaces `vec![...]` in panel builder)
- `apps/fret-examples/src/cjk_conformance_demo.rs` (example: `ui::children!` replaces `vec![...]` root return)

Acceptance:

- A reviewer can scan `simple-todo` and not see “adapter noise” dominate the code (goal: composition reads first).
- Suggested quant target (bikesheddable):
  - `< 10` explicit `.into_element(cx)` calls in template UI code (conversion moves into ecosystem boundaries).

---

## Milestone 3 (M3): Interop “Tier A” demo + cookbook

Outcome:

- Interop is not just a concept; it is a runnable, debuggable path.

Deliverables:

- Minimal embedded-viewport demo exists (offscreen render target + input forwarding).
- Interop cookbook doc exists (Tier A only; explicit pitfalls section).

Acceptance:

- Demo shows visible feedback from forwarded input (e.g. click counter + last event text).
- At least one scripted repro exists (screenshot / trace) so regressions are reviewable.

---

## Milestone 4 (M4): Third-party integration helpers (stable-ish, ecosystem-only)

Outcome:

- Third-party ecosystem crates can opt into `.ui()` with minimal ceremony, without copying internal macros.

Deliverables:

- `macro_rules!` helpers exist in `fret-ui-kit` to implement `UiPatchTarget`/`UiSupports*`/`UiIntoElement` in one line.
- The “IntoElement-like adapter” posture is decided for third-party components:
  - blanket impl (preferred) or derive (fallback) is documented with the coherence constraint.

Evidence (current):

- `ecosystem/fret-ui-kit/src/lib.rs` (`ui_component_*` helpers)
- `docs/component-authoring-contracts.md` (recommended usage + patch-only)
- `ecosystem/fret-ui-ai/src/elements/message.rs` (external-style sample)
- `ecosystem/fret-ui-ai/src/elements/workflow/panel.rs` (patch-only sample)

Acceptance:

- A minimal third-party-style component can integrate with `.ui()` without bespoke glue code.
- A third-party component type can opt into `.ui()` in one line (macro_rules helper), and avoid per-type `UiIntoElement` glue
  via blanket impl or derive (documented with coherence constraints).

---

## Milestone 5 (M5): Commands macros (optional, but high leverage)

Outcome:

- Third-party crates can declare `CommandId` + `CommandMeta` + default keybindings with low boilerplate (GPUI-like).

Deliverables:

- A command macro plan exists (`commands!{...}` / `#[derive(Action)]`-style), with a small prototype on 1–2 crates.

Acceptance:

- The generated output remains “thin plumbing” (no hidden policy), and app integration stays explicit (`install(app)`).

---

## Milestone 6 (M6): MVU clarity + future escape hatch definition

Outcome:

- Users understand MVU’s “simple default” posture and how to evolve beyond it.

Deliverables:

- MVU docs explain the “tick refresh” behavior and when to drop down to manual driver wiring.
- A scoped proposal exists for an opt-in “more precise invalidation” mode (if we choose to pursue it).

Acceptance:

- MVU posture is explicit in docs; perf-sensitive users have clear guidance.

Evidence (current):

- `docs/examples/todo-app-golden-path.md` (tick refresh notes)
- `ecosystem/fret/src/mvu.rs` (internal `tick` model)

---

## (Optional) Milestone 7 (M7): Demo shell consolidation

Outcome:

- The “what do I run?” story is unambiguous for both native and wasm.

Deliverables:

- `apps/fret-demo` and `apps/fret-demo-web` are the documented canonical shells.
- Core onboarding demos (at least `simple-todo`) run on both native and wasm through the same selection mechanism.
- `ui_gallery` is selectable and runnable in the native shell (mirroring wasm).

Acceptance:

- `fretboard dev web --demo simple_todo` (name TBD) works.
- Native has an equivalent runnable path (either `--bin simple_todo` or `fret-demo -- <id>`).

Evidence (current):

- `apps/fret-demo/src/bin/ui_gallery.rs` (native bin, feature-gated)
- `apps/fret-demo-web/src/wasm.rs` (wasm selection already supports `ui_gallery`)
