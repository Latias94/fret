# Action-First Authoring + View Runtime (Fearless Refactor v1) — App Entry Caller Inventory

Last updated: 2026-03-09

This inventory turns the app-entry policy draft into an execution list.

Scope:

- public `fret::App::ui(...)`
- public `fret::App::ui_with_hooks(...)`
- direct in-tree consumers that still rely on those surfaces

Non-scope:

- builder/patch `.ui()` on composed widgets (`Card::build(...).ui()`, etc.)
- lower-level `run_native_with_compat_driver(...)` consumers (tracked separately in `HARD_DELETE_GAP_ANALYSIS.md`)
- internal constructor/forwarder methods that define the API but are not end-user call sites

Decision context:

- Policy draft: `docs/workstreams/action-first-authoring-fearless-refactor-v1/APP_ENTRY_POLICY_DECISION_DRAFT.md`
- Hard-delete blockers: `docs/workstreams/action-first-authoring-fearless-refactor-v1/HARD_DELETE_GAP_ANALYSIS.md`

---

## Classification labels

- `migrate-to-view` — should move to `App::view::<V>()` or `App::view_with_hooks::<V>(...)`
- `move-lower-level` — should likely stop using the facade entry API and drop to bootstrap/driver-level wiring
- `keep-temporarily` — acceptable bridge user for now, but should not remain in first-contact surfaces
- `done` ? already migrated to `App::view::<V>()` / `run_view::<V>()`; retained here until the full inventory is burned down

Current conclusion:

- The in-tree `App::ui*` callers are overwhelmingly **migration debt**, not evidence that the closure surface must remain a co-equal long-term API.
- Most current consumers should move to `View` / `view_with_hooks`, not to lower-level bootstrap APIs.

---

## Current in-tree `ui_with_hooks(...)` callers

| File | Current role | Recommended class | Notes |
|---|---|---|---|
| `apps/fret-examples/src/assets_demo.rs` | advanced asset/event demo | `done` | migrated on 2026-03-08 to `view_with_hooks::<AssetsDemoView>(...)`; proves driver event hooks do not by themselves require closure-root `ui_with_hooks(...)` |
| `apps/fret-examples/src/embedded_viewport_demo.rs` | advanced viewport interop demo | `done` | migrated on 2026-03-08 to `view_with_hooks::<EmbeddedViewportDemoView>(...)`; `EmbeddedViewportView` now lets retained viewport recording compose with `ViewWindowState<V>` |
| `apps/fret-examples/src/external_texture_imports_demo.rs` | advanced external texture interop | `done` | migrated on 2026-03-09 to `view_with_hooks::<ExternalTextureImportsView>(...)`; shows that one Batch C interop demo also fits the view runtime hook path without closure-root state |
| `apps/fret-examples/src/external_video_imports_avf_demo.rs` | platform/media interop demo | `done` | migrated on 2026-03-09 to `view_with_hooks::<ExternalVideoImportsAvfView>(...)`; confirms the remaining AVF/macOS path also fits the view runtime hook path |
| `apps/fret-examples/src/external_video_imports_mf_demo.rs` | platform/media interop demo | `done` | migrated on 2026-03-09 to `view_with_hooks::<ExternalVideoImportsMfView>(...)`; narrows the remaining closure-root app-entry risk to the AVF/macOS video path |
| `apps/fret-examples/src/image_heavy_memory_demo.rs` | memory/perf-oriented demo | `done` | migrated on 2026-03-08 to `view_with_hooks::<ImageHeavyMemoryView>(...)`; confirms frame-recorder-only demos also fit the view runtime hook path |
| `apps/fret-examples/src/imui_editor_proof_demo.rs` | IMUI/editor proof demo | `done` | migrated on 2026-03-09 to `view_with_hooks::<ImUiEditorProofView>(...)`; confirms the editor-grade docking + embedded viewport proof also fits the view runtime hook path |

## Current in-tree `ui(...)` callers

| File | Current role | Recommended class | Notes |
|---|---|---|---|
| `apps/fret-examples/src/chart_declarative_demo.rs` | declarative chart demo | `done` | migrated on 2026-03-08 to `run_view::<ChartDeclarativeView>()`; confirms a plain declarative chart demo also does not need closure-root `App::ui(...)` |
| `apps/fret-examples/src/imui_floating_windows_demo.rs` | IMUI demo | `done` | migrated on 2026-03-09 to `run_view::<ImUiFloatingWindowsView>()`; confirms a floating-window IMUI surface also does not need closure-root app entry |
| `apps/fret-examples/src/imui_hello_demo.rs` | minimal IMUI demo | `done` | migrated on 2026-03-08 to `run_view::<ImUiHelloView>()`; keep as the first Batch A proof point until the remaining callers are burned down |
| `apps/fret-examples/src/imui_node_graph_demo.rs` | IMUI + node-graph demo | `done` | migrated on 2026-03-09 to `run_view::<ImUiNodeGraphView>()`; confirms retained IMUI node-graph demos also fit the default view entry path |
| `apps/fret-examples/src/imui_response_signals_demo.rs` | IMUI response demo | `done` | migrated on 2026-03-08 to `run_view::<ImUiResponseSignalsView>()`; proves response-signal-heavy IMUI demos also fit the default view entry path |
| `apps/fret-examples/src/imui_shadcn_adapter_demo.rs` | IMUI + shadcn adapter demo | `done` | migrated on 2026-03-09 to `run_view::<ImUiShadcnAdapterView>()`; confirms adapter-heavy IMUI demos also fit the default view entry path |
| `apps/fret-examples/src/node_graph_demo.rs` | node-graph demo | `done` | migrated on 2026-03-08 to `run_view::<NodeGraphDemoView>()`; confirms a retained-model canvas demo also fits the default view entry path |

---

## Non-consumer references that still need cleanup attention

These are not part of the migration table above, but they still matter for policy closure:

| File | Why it matters |
|---|---|
| `ecosystem/fret/src/lib.rs` | rustdoc still contains closure-style `App::ui(...)` example text and should eventually follow the final policy decision |
| `ecosystem/fret/src/app_entry.rs` | defines the public `ui(...)` / `ui_with_hooks(...)` surface; deprecation warnings now live here while removal is sequenced separately |
| `ecosystem/fret/README.md` | now marks `ui(...)` / `ui_with_hooks(...)` as deprecated advanced bridges and is covered by an in-crate policy test |

---

## Suggested migration order

## Batch A — easiest proof points

These should move first because they are small and make the policy credible quickly:

- `apps/fret-examples/src/imui_hello_demo.rs` _(done on 2026-03-08)_
- `apps/fret-examples/src/imui_response_signals_demo.rs` _(done on 2026-03-08)_
- `apps/fret-examples/src/chart_declarative_demo.rs` _(done on 2026-03-08)_
- `apps/fret-examples/src/node_graph_demo.rs` _(done on 2026-03-08)_

Success criterion:

- the repo can show multiple non-trivial examples using `View` entry without relying on closure-root `App::ui(...)`.

Status update:

- Batch A is now complete: all four planned proof points have moved off `App::ui(...)`.

## Batch B — hook-preserving migrations

These should move next to prove `view_with_hooks` is sufficient for advanced-but-facade-level demos:

- `apps/fret-examples/src/assets_demo.rs` _(done on 2026-03-08)_
- `apps/fret-examples/src/image_heavy_memory_demo.rs` _(done on 2026-03-08)_
- `apps/fret-examples/src/imui_editor_proof_demo.rs` _(done on 2026-03-09)_
- `apps/fret-examples/src/embedded_viewport_demo.rs` _(done on 2026-03-08)_

Success criterion:

- the repo can show that driver hooks do not force the facade back to closure-root entry.

Status update:

- Batch B is now complete: `assets_demo`, `embedded_viewport_demo`, `image_heavy_memory_demo`, and `imui_editor_proof_demo` all run through `view_with_hooks::<...>(...)`.

## Batch C — highest-risk interop demos

These should be migrated only after the policy and hook path are already proven by B:

- `apps/fret-examples/src/external_texture_imports_demo.rs` _(done on 2026-03-09)_
- `apps/fret-examples/src/external_video_imports_avf_demo.rs` _(done on 2026-03-09)_
- `apps/fret-examples/src/external_video_imports_mf_demo.rs` _(done on 2026-03-09)_

Decision gate:

- if one of these truly needs a lower-level bootstrap/driver path, document that explicitly and stop treating it as evidence for keeping `App::ui_with_hooks(...)` in the default facade.

Status update:

- Batch C is now complete: `external_texture_imports_demo`, `external_video_imports_mf_demo`, and `external_video_imports_avf_demo` all run through `view_with_hooks::<...>(...)`.

---

## Practical verdict

Based on the current caller set, the repo is **not blocked by lack of a `View`-based app entry API**.
There are now no in-tree example/demo callers left on `App::ui(...)` / `ui_with_hooks(...)`.
The remaining work is staged deprecation/removal sequencing plus any final lower-level exceptions that are intentionally kept outside the facade.
