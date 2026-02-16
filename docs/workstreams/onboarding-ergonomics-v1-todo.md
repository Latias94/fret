# Onboarding Ergonomics + Ecosystem Interop — TODOs (v1)

Status: Active

This tracker focuses on improvements that make Fret easier to learn and easier to adopt, while
preserving the “mechanism vs policy” split.

Design note:

- `docs/workstreams/onboarding-ergonomics-v1.md`

## Decisions (2026-02-15)

- Template naming: use a **separate** `simple-todo` template name.
- Demo shells: `apps/fret-demo` (native) + `apps/fret-demo-web` (wasm) are the canonical entry points.
- Native demo shell must be able to select and run `ui_gallery`.
- Authoring surface posture: keep the ecosystem `UiPatch` / `UiBuilder` design (ADR 0160); do not remove it.
- Macro posture (v1): prioritize “composition macros” that reduce call-site adapters (GPUI/iced style); avoid DSL macros.

## Tracking Format

- ID: `ONB-{area}-{nnn}`
- Status: `[ ]` open, `[~]` in progress, `[x]` done, `[!]` blocked

---

## A. Onboarding Ladder (docs-first)

- [x] ONB-docs-001 Add a “First hour” onboarding doc (native-first).
  - Must include: dependencies, minimal MVU, keyed lists, invalidation cheat sheet.
  - Evidence anchors:
    - `README.md` (quick start)
    - `docs/first-hour.md`
    - `docs/examples/todo-app-golden-path.md`
    - `docs/ui-ergonomics-and-interop.md` (interop tiering)

- [x] ONB-docs-002 Update `docs/examples/todo-app-golden-path.md` to explicitly position:
  - “simple-todo” as the Step 1 baseline (Model + MVU),
  - “todo” as Step 2/3 (selector + query).

- [x] ONB-docs-003 Add a short “Invalidation rules of thumb” section.
  - Table:
    - value affects only visuals → `Paint`
    - affects sizing/flow/scroll extents → `Layout`
    - affects hit regions only → `HitTest`
  - Evidence:
    - `docs/first-hour.md`
    - `ecosystem/fret-ui-kit/src/declarative/model_watch.rs`
    - `crates/fret-ui/src/elements/cx.rs` (`observe_model`)

- [x] ONB-docs-004 Add a short “Identity rules of thumb” section.
  - Must teach `cx.keyed(...)` for dynamic lists and why.
  - Evidence:
    - `docs/first-hour.md`
    - `crates/fret-ui/src/elements/cx.rs` (`scope`, `keyed`)

---

## B. Templates (fretboard scaffolds)

- [x] ONB-tpl-010 Add `simple-todo` template.
  - Scope:
    - `Model<T>` + MVU typed messages
    - shadcn components
    - keyed list rendering
    - no selector/query dependencies
  - Evidence:
    - `apps/fretboard/src/scaffold/templates.rs`

- [x] ONB-tpl-011 Keep `todo` template as the “best practice baseline”.
  - Ensure it remains aligned with:
    - `apps/fret-examples/src/todo_demo.rs`
    - `docs/examples/todo-app-golden-path.md`
  - Evidence:
    - `apps/fretboard/src/scaffold/templates.rs` (`todo` template)

- [x] ONB-tpl-012 Add a “template matrix” doc section (what each template teaches).
  - Where:
    - `docs/first-hour.md`

---

## C. Reduce Authoring Noise (teach + small deltas)

- [x] ONB-auth-020 Ensure onboarding docs teach `ui::*` constructors early.
  - Evidence:
    - `ecosystem/fret-ui-kit/src/ui.rs`
    - `ecosystem/fret-ui-shadcn/src/lib.rs` (prelude exports)
    - `docs/first-hour.md`

- [x] ONB-auth-021 Promote iterator helpers for child collection.
  - Teach `AnyElementIterExt::elements()` / `elements_owned()` in onboarding docs.
  - Evidence:
    - `crates/fret-ui/src/element.rs` (`AnyElementIterExt`)
    - `ecosystem/fret-ui-kit/src/ui.rs` (`*_build` iterator helpers)
    - `docs/first-hour.md`

- [x] ONB-auth-022 Audit top-level examples for avoidable `vec![...]` / `.collect()` boilerplate.
  - Target files (initial):
    - `apps/fret-examples/src/*_demo.rs`
    - `apps/fret-ui-gallery/src/ui/*`
  - Partial evidence (started):
    - `apps/fret-ui-gallery/src/ui/pages/alert.rs` (migrated to `ui::children!`)
    - `apps/fret-examples/src/todo_demo.rs` (uses `ui::children!` for heterogeneous children)
    - `apps/fret-examples/src/assets_demo.rs` (replaced `vec![...]` with `ui::children!` in panel builder)
    - `apps/fret-examples/src/cjk_conformance_demo.rs` (replaced `vec![page]` with `ui::children!`)
    - `apps/fret-examples/src/emoji_conformance_demo.rs` (replaced `vec![page]` with `ui::children!`)
    - `apps/fret-examples/src/genui_demo.rs` (replaced `vec![...]` child lists with arrays/`ui::children!`)
    - `apps/fret-examples/src/hello_counter_demo.rs` (reduced explicit `.into_element(cx)` in layout composition by relying on `UiIntoElement` children)

- [x] ONB-auth-023 Reduce call-site `.into_element(cx)` noise by teaching constructors to accept `UiIntoElement` children.
  - Goal: move adapter verbosity to ecosystem boundaries (GPUI-like composition).
  - Proposed targets (ecosystem-only):
    - `ecosystem/fret-ui-kit/src/ui.rs` (`ui::h_flex`, `ui::v_flex`, `ui::container_build`, `ui::scroll_area_build`, ...)
  - Prerequisites:
    - confirm a single component trait posture for third-party code (`UiIntoElement` and/or `RenderOnce`).
  - Acceptance:
    - `simple-todo` template can be authored without `.into_element(cx)` for common components.
  - Evidence:
    - `ecosystem/fret-ui-kit/src/ui.rs` (layout constructors accept `UiIntoElement` children)
    - `docs/first-hour.md`

- [x] ONB-auth-025 Migrate `simple-todo` template to `ui::children!` + `.ui()` (reduce adapter noise).
  - Evidence:
    - `apps/fretboard/src/scaffold/templates.rs`

- [x] ONB-auth-024 Add an iced-like `children![...]` macro for heterogeneous child lists.
  - Goal: remove repetitive `.into_element(cx)` calls without redesigning all constructors.
  - Reference:
    - iced `row!/column!`: `repo-ref/iced/widget/src/helpers.rs`
  - Proposed shape (bikesheddable):
    - `ui::children![cx; a, b, c] -> Vec<AnyElement>`
    - usable inside `ui::h_flex(cx, |cx| ui::children![cx; ...])`
  - Scope:
    - `ecosystem/fret-ui-kit` (re-export from `fret_ui_shadcn::prelude`)

---

## D. Interop (Tier A) — runnable demo + cookbook

- [x] ONB-interop-030 Add a minimal runnable “embedded viewport” demo.
  - Must show:
    - offscreen target allocation + resize
    - rendering into target
    - embedding as a panel
    - input forwarding + visible feedback
  - Evidence:
    - `apps/fret-examples/src/embedded_viewport_demo.rs`
    - `apps/fret-demo/src/bin/embedded_viewport_demo.rs`
    - `ecosystem/fret/src/interop/embedded_viewport.rs`

- [x] ONB-interop-031 Add a short interop cookbook doc.
  - Must include:
    - “Do this” (Tier A)
    - “Don’t do this” (same-tree mixing pitfalls)
    - pointers to diagnostics tooling
  - Evidence:
    - `docs/ui-ergonomics-and-interop.md`
    - `docs/interop-tier-a-embedded-viewport.md`

- [ ] ONB-interop-032 Add at least one scripted repro gate for the interop demo.
  - Where:
    - `tools/diag-scripts/` (exact filename TBD)
  - Output:
    - screenshot bundle or deterministic input trace

---

## E. MVU ergonomics (clarity + future escape hatch)

 - [x] ONB-mvu-040 Document the MVU “tick refresh” posture and its implications.
   - Evidence:
     - `ecosystem/fret/src/mvu.rs` (internal `tick` model)
     - `docs/examples/todo-app-golden-path.md`

- [ ] ONB-mvu-041 Explore an opt-in mode for more precise invalidation (future).
  - Decision gate:
    - do we keep MVU always “simple + safe”, and push perf apps toward manual driver wiring?

---

## F. Optional proc-macro derives (ecosystem only)

- [ ] ONB-macro-050 Draft a minimal derive plan for component authors.
  - Candidate targets:
    - derive `UiPatchTarget` for common “builder structs”
    - derive `UiIntoElement` for common “render into AnyElement” patterns
  - Constraints:
    - opt-in (no proc-macro required for golden path)
    - keep trait surface stable-ish before encouraging adoption
  - Alternative/adjacent (lower-risk):
    - export `macro_rules!` integration helpers from `fret-ui-kit` (mirrors internal shadcn wiring).

- [ ] ONB-macro-051 Prototype the macro crate on 1–3 internal components.
  - Leave evidence anchors and avoid broad churn.

- [!] ONB-macro-052 Add a blanket impl: `UiIntoElement` for all `T: fret_ui::element::RenderOnce`.
  - Goal: eliminate repetitive `impl UiIntoElement` boilerplate in third-party crates.
  - Note: blocked by Rust coherence (would conflict with `UiIntoElement for AnyElement` if upstream adds `RenderOnce`).
    Use `fret_ui_kit::ui_into_element_render_once!(Ty)` instead.
  - Evidence:
    - `crates/fret-ui/src/element.rs` (`RenderOnce`)
    - `ecosystem/fret-ui-kit/src/ui_builder.rs` (`UiIntoElement`)

- [ ] ONB-macro-053 Draft a GPUI-like command macro plan (`CommandId` + `CommandMeta` + keybindings).
  - Goal: make third-party ecosystem crates cheap to integrate (install + discoverability).
  - Evidence:
    - `crates/fret-runtime/src/commands.rs` (`CommandMeta`, `DefaultKeybinding`)
    - `crates/fret-app/src/core_commands.rs` (registration patterns)

- [x] ONB-macro-054 Export `macro_rules!` helpers for third-party `.ui()` integration.
  - Goal: make “opt into `UiBuilder`” a 1-line change for external crates.
  - Proposed surface (names bikesheddable):
    - `fret_ui_kit::ui_component_chrome_layout!(Ty);`
    - `fret_ui_kit::ui_component_layout_only!(Ty);`
    - patch-only variants (no `UiIntoElement`) for “sub-surfaces” that should not render directly.
  - Evidence (internal precedent):
    - `ecosystem/fret-ui-shadcn/src/ui_ext/support.rs`
  - Evidence (in-tree external-style example):
    - `ecosystem/fret-ui-ai/src/elements/message.rs`
    - `ecosystem/fret-ui-ai/src/elements/workflow/panel.rs`
    - `docs/component-authoring-contracts.md`

- [x] ONB-macro-055 Decide the “IntoElement-like” adapter strategy for third-party components.
  - Preferred:
    - blanket impl `UiIntoElement` for `T: RenderOnce` (if coherent).
  - Fallbacks (if coherence conflicts appear):
    - `#[derive(UiIntoElement)]` that forwards to `RenderOnce::render_once`, or
    - a secondary trait accepted by `UiBuilder` (avoid breaking existing explicit impls).
  - Evidence:
    - `docs/component-authoring-contracts.md`

---

## G. Demo shells (reduce redundancy)

- [x] ONB-demo-060 Decide the canonical demo shells for native + wasm.
  - Proposed:
    - `apps/fret-demo` (native)
    - `apps/fret-demo-web` (wasm; already used by `fretboard dev web`)
  - Evidence:
    - `apps/fretboard/src/dev.rs` (web dev shells into `apps/fret-demo-web`)
    - `docs/workstreams/onboarding-ergonomics-v1.md` (Decisions)

- [ ] ONB-demo-061 Make “core onboarding” demos runnable on both native + wasm via the same selection mechanism.
  - Targets:
    - `simple-todo`
    - `todo_demo` (if we keep it as the “best practice baseline”)

- [ ] ONB-demo-062 Evaluate whether we still need separate `*-web` shells for individual apps once they are selectable in `fret-demo-web`.
  - Decision gate:
    - land only after we have an explicit deprecation story + doc updates.

- [x] ONB-demo-063 Add `ui_gallery` to the native demo selection surface (`apps/fret-demo`).
  - Goal: one canonical shell works on both native and wasm (native CLI + wasm URL selection).
  - Evidence:
    - wasm already supports `ui_gallery`: `apps/fret-demo-web/src/wasm.rs`
    - native bin: `apps/fret-demo/src/bin/ui_gallery.rs` (run with `cargo run -p fret-demo --features ui-gallery --bin ui_gallery`)

---

## H. Metrics & regression guards (keep friction from creeping back)

- [x] ONB-metrics-070 Add a baseline “authoring friction” metrics table to the workstream design note.
  - Evidence:
    - `docs/workstreams/onboarding-ergonomics-v1.md`

- [ ] ONB-metrics-071 Add a tiny script to refresh baseline counts (and keep a record in the workstream).
  - Suggested shape:
    - `tools/metrics/ui_authoring_noise.ps1` (PowerShell) or `tools/metrics/ui_authoring_noise.py`
  - Metrics to capture (initial):
    - `.into_element(cx)` occurrences in `apps/fret-examples/src`
    - `.into_element(cx)` occurrences in `ecosystem/fret-ui-shadcn/src`
    - `impl_ui_patch_*!(...)` count in `ecosystem/fret-ui-shadcn/src/ui_ext`

- [x] ONB-metrics-072 Define target thresholds for `simple-todo` (composition readability) and enforce them in review.
  - Example target (bikesheddable):
    - “< 10 explicit `.into_element(cx)` calls in template UI code” (conversion moves into ecosystem boundaries).
  - Evidence:
    - `apps/fretboard/src/scaffold/templates.rs` (template test asserts `<= 10`)
