# Immediate-Mode Authoring Facade ("imui") v2 - TODO Tracker

Status: Complete (M0–M5 implemented; keep for history)
Last updated: 2026-02-03

This tracker covers the fearless v2 consolidation work described in:

- `docs/workstreams/imui-authoring-facade-v2.md`

Related:

- v1 baseline: `docs/workstreams/imui-authoring-facade-v1.md`
- v1 tracker: `docs/workstreams/imui-authoring-facade-v1-todo.md`
- unified patch chain ADR: `docs/adr/0175-unified-authoring-builder-surface-v1.md`
- unified builder workstream: `docs/workstreams/unified-authoring-builder-v1.md`

Legend:

- [ ] open
- [~] in progress
- [x] done
- [!] blocked / needs decision

Tracking format:

- ID: `IMUI2-{area}-{nnn}`
- Areas:
  - `scope` (contracts, invariants, ownership decisions)
  - `api` (public API shape)
  - `bridge` (imui ↔ ui()/UiBuilder integration)
  - `eco` (official ecosystem adoption rules)
  - `demo` (demos and proof points)
  - `test` (tests and harnesses)
  - `docs` (guides and migration notes)

---

## M0 - Lock the v2 seams (decisions first)

Exit criteria:

- v2 “do not break” invariants are copied forward from v1 and re-affirmed.
- Ownership of the “writer” contract is decided (where it lives; what it depends on).
- The “single authoritative widget implementation” rule is written down for official crates.

- [x] IMUI2-scope-001 Decide where the writer trait lives:
  - candidates: `fret-imui`, `fret-ui-kit`, or a new tiny ecosystem crate.
  - recommendation: a new tiny ecosystem crate (e.g. `ecosystem/fret-authoring`) to avoid policy coupling and cycles.
- [x] IMUI2-scope-002 Define the canonical widget rule for official crates:
  - One source-of-truth implementation per widget (prefer a frontend-agnostic core widget/element).
  - Multiple authoring entry points are allowed as thin adapters (imui/ui-kit/shadcn) but must delegate.
  - Avoid duplicated state machines and interaction rules across authoring paths.
- [x] IMUI2-scope-003 Decide v2 public surface stability policy:
  - Treat `fret-authoring::UiWriter` as the shared contract we try to keep stable once ecosystem crates depend on it.
  - Treat bridge utilities (`fret-ui-kit::imui`) and imui ergonomics (`fret-imui`) as unstable during the refactor.
  - The “do not break” invariants remain non-negotiable even while APIs churn.

---

## M1 - Unify the immediate-mode composition contract

Exit criteria:

- A minimal writer contract exists and `ImUi` implements it.
- Third-party widgets can accept a single surface without knowing the concrete frontend.

- [x] IMUI2-api-010 Introduce the writer contract (bikesheddable name; minimal methods only).
- [x] IMUI2-api-011 Update `ImUi` to implement the writer contract.
- [x] IMUI2-api-016 Add `UiWriter` extension helpers for core widgets (so third-party widgets can accept `&mut impl UiWriter`):
  - `button`, `checkbox_model`, `text`, `separator`.
  - Evidence: `ecosystem/fret-ui-kit/src/imui.rs` (`UiWriterImUiFacadeExt`, `ResponseExt`).
- [x] IMUI2-eco-013 Update official ecosystem `imui` adapters to accept `UiWriter` (no concrete `ImUi` coupling).
- [x] IMUI2-eco-014 Decouple official ecosystem `imui` feature gates from `fret-imui` (depend on `UiWriter` only).
  - Evidence: `ecosystem/fret-markdown/Cargo.toml`, `ecosystem/fret-code-view/Cargo.toml`, `ecosystem/fret-docking/Cargo.toml`, `ecosystem/fret-node/Cargo.toml`.
- [x] IMUI2-test-012 Add compile-level smoke tests ensuring the writer surface remains usable across crates.
  - Evidence: `ecosystem/fret-authoring/src/lib.rs` (compile smoke), `ecosystem/fret-imui/src/lib.rs` (bridge smoke).

---

## M2 - Bridge `ui()` / `UiBuilder<T>` into imui (convergence)

Exit criteria:

- App code can write immediate-mode control flow while using `UiBuilder<T>` for patch vocabulary.
- `fret-imui` remains policy-light (no hard dependency on `fret-ui-kit`).

- [x] IMUI2-bridge-020 Add an ecosystem-owned bridge module (in `fret-ui-kit` behind an `imui` feature):
  - extension trait on `UiWriter` to render `UiBuilder<T>` into the current output list.
- [x] IMUI2-bridge-021 Decide and document where token/preset helpers live (kit vs shadcn vs app).
  - Rule: `fret-ui-kit` owns patch vocabulary + generic scales/presets; `fret-ui-shadcn` owns shadcn-aligned tokens and recipes; app owns app-specific tokens.
  - Evidence: `docs/workstreams/imui-authoring-facade-v2.md` (decision snapshot).

---

## M3 - Close the v1 “leftovers” inside v2

Exit criteria:

- The most important v1 follow-ups are addressed (moved into v2 so v1 can remain frozen).

- [x] IMUI2-test-030 Add a wasm-targeted smoke harness entry (compile-only is acceptable initially).
  - Evidence: `cargo check -p fret-authoring -p fret-imui --target wasm32-unknown-unknown`.
- [x] IMUI2-docs-031 Add “when to drop to `cx_mut()`” guidance (canvas, viewport surfaces, docking host).
- [x] IMUI2-docs-032 Add a concise “Golden Path” section + gotchas/FAQ for immediate-style authoring in Fret.
- [x] IMUI2-eco-033 Add at least one more official ecosystem `imui` adapter (`fret-plot` or `fret-chart`).
  - Evidence: `ecosystem/fret-plot/src/imui.rs` (feature `fret-plot/imui`).

---

## M4 - Demos and proof points (keep editor-grade green)

Exit criteria:

- Editor-grade proof demos run unchanged in intent (even if APIs changed).
- The demo suite exercises the multi-window + docking + viewport seams during the refactor.

- [x] IMUI2-demo-040 Migrate `imui_hello_demo` to v2 surface (smoke).
- [x] IMUI2-demo-041 Migrate `imui_node_graph_demo` to v2 surface (retained subtree interop).
- [x] IMUI2-demo-042 Migrate `imui_editor_proof_demo` to v2 surface (multi-window + docking + viewport).
  - Started by rendering the root layout via `ui::v_flex_build` + `UiWriterUiKitExt::add_ui(...)`.
  - Migrated the docking-hosted controls panel root container to `ui::container_build` + `UiBuilder` chrome/layout patches.
  - Removed the last manual `LayoutStyle` usage for docking host embedding by improving `DockSpaceImUiOptions::default()`.

---

## M5 - Delete v1 surface (flag day)

Exit criteria:

- v2 surface is the only supported `imui` authoring API in-tree.
- Docs and workstreams reflect the new reality.

- [x] IMUI2-api-050 Remove v1 `imui` widget methods from `ImUi` (use an ecosystem facade extension trait instead).
  - Evidence: `ecosystem/fret-imui/src/lib.rs` (`fret-imui` stays minimal; widget helpers live in `fret-ui-kit`'s `imui` feature).
- [x] IMUI2-docs-051 Update workstream docs and demos to point to v2.
  - Evidence: `docs/workstreams/imui-authoring-facade-v2.md`, `docs/workstreams/imui-authoring-facade-v1.md`, `apps/fret-examples/src/imui_hello_demo.rs`.
