# Editor Ecosystem Fearless Refactor v1 - Parity Matrix

Last updated: 2026-03-09

This matrix tracks outcome alignment, not 1:1 API parity.

Legend:

- `✅` implemented / clearly owned
- `⚠️` partial / direction exists but not closed
- `❌` missing
- `🧭` intentionally different

| Area | Dear ImGui / egui reference outcome | Fret today | Fret target | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| Immediate-style authoring | `ui.xxx(...)` flow with direct `Response` feedback | ✅ `fret-imui` exists as a small authoring frontend | ✅ | `ecosystem/fret-imui` | Keep the experience, not the second runtime. Evidence: `ecosystem/fret-imui/src/lib.rs`. |
| `imui` is not a second widget library | immediate syntax should still use the same widget semantics | ⚠️ a starter thin adapter now exists for core editor controls, but the surface is still incomplete | ✅ | `ecosystem/fret-ui-editor` | Must stay a thin adapter over the declarative implementation. Evidence: `ecosystem/fret-ui-editor/src/imui.rs`. |
| Single source-of-truth widgets | one implementation per widget, multiple authoring frontends | ⚠️ direction is documented, not fully closed | ✅ | `ecosystem/fret-ui-editor` | Declarative first; `imui` delegates. Evidence: `docs/workstreams/ui-editor-v1/ui-editor-v1.md`. |
| Stable widget identity | explicit identity / no accidental state sharing | ⚠️ partially addressed via `id_source` and keyed helpers | ✅ | `ecosystem/fret-ui-editor`, `ecosystem/fret-imui` | This is an editor-grade correctness issue, not optional polish. |
| Unified widget visuals | ImGui style / egui `Visuals::widgets` class consistency | ⚠️ `EditorWidgetVisuals` exists, but coverage is still closing | ✅ | `ecosystem/fret-ui-editor` | Editor controls should not drift visual states control-by-control. |
| Density / spacing system | one place to tune row height, padding, hit size | ⚠️ `editor.density.*` exists in direction and partial implementation | ✅ | `ecosystem/fret-ui-editor` | This is where "imgui-like dense feel" should live. |
| Numeric drag / scrub | `DragFloat*` / `egui::DragValue` class hand feel | ⚠️ landed in parts, still hardening | ✅ | `ecosystem/fret-ui-editor` | Includes threshold, slow/fast modifiers, commit/cancel semantics. |
| Double-click to type | drag controls can switch to typed edit | ⚠️ partial | ✅ | `ecosystem/fret-ui-editor` | Important for editor-grade numeric workflows. |
| Edit session semantics | begin / update / commit / cancel are consistent | ⚠️ partial | ✅ | `ecosystem/fret-ui-editor` | This is one of the main behaviors we should learn from ImGui. |
| Property grid / property group / panel | inspector starter set exists and feels coherent | ⚠️ partial | ✅ | `ecosystem/fret-ui-editor` | Core reusable editor surface, not app-specific protocol. |
| Color / vec / transform editors | common editor composite set exists | ⚠️ partial | ✅ | `ecosystem/fret-ui-editor` | These are exactly the components we do need to design ourselves. |
| Text field richness | editor text input is solid enough for inspector workflows | ⚠️ partial | ✅ | `ecosystem/fret-ui-editor` with possible `crates/fret-ui` gaps | Password/completion/history hooks are still later. |
| Enum select / popup edit surfaces | compact editor selection and popup editing | ⚠️ partial | ✅ | `ecosystem/fret-ui-editor` + `ecosystem/fret-ui-kit` | Overlay mechanics stay below; editor recipes stay here. |
| Inspector protocol layer | property tree/path/editor-kind lives outside widget crate | ⚠️ currently app-local in `apps/fret-editor` | ✅ | future inspector/property protocol crate | This is why `fret-ui-editor` should not absorb everything inspector-related. |
| Inspector edit session services | popup edit requests and window-scoped edit services are separated from widgets | ⚠️ app-local only | ⚠️ | future inspector session crate or app layer | Not urgent to extract before the protocol layer stabilizes. |
| Workspace shell layer | editor app shell is separate from editor widgets | ✅ `fret-workspace` exists | ✅ | `ecosystem/fret-workspace` | Keep shell chrome separate from control library. |
| Docking remains separate | dock-graph-aware policy does not leak into shell/widget crates | ✅ `fret-docking` exists and is separated | ✅ | `ecosystem/fret-docking` | Keep docking tab/drop/split behavior out of `fret-ui-editor`. |
| Viewport tooling split | generic tool-input glue is separate from gizmo/editor app logic | ✅ `fret-viewport-tooling` exists | ✅ | `ecosystem/fret-viewport-tooling`, `ecosystem/fret-gizmo` | App viewport code should converge here before new extraction. |
| Skinning / preset adapters | editor widgets can look shadcn-like, imgui-like, material-like without forks | ⚠️ boundary is now documented, adapters not closed | ✅ | skin crates / preset modules | One-way dependency only. Core editor crate stays design-system agnostic. |
| Editor token namespace | one stable token vocabulary for editor surfaces | ⚠️ direction documented and partially implemented | ✅ | `ecosystem/fret-ui-editor` | `editor.*` should own the "editor feel". |
| Workspace token namespace | one stable token vocabulary for shell chrome | ⚠️ direction documented, still needs inventory | ✅ | `ecosystem/fret-workspace` | `workspace.*` should own shell chrome, not `fret-ui`. |
| `imui` and declarative share tokens | no facade-only styling vocabulary | ✅ decision locked | ✅ | `ecosystem/fret-imui`, `ecosystem/fret-ui-editor` | No `imui.editor.*`. Same widgets, same tokens, different syntax. |
| Imgui-like preset | dense, low-ceremony visual preset for editor apps | ⚠️ first preset + proof surface landed, broader shell alignment still open | ✅ | adapter/preset layer | `EditorThemePresetV1::ImguiLikeDense` is now wired through the proof demo; this remains a skin/preset, not a separate component implementation. |
| Proof demos and gates | interaction feel is protected by proof surfaces and focused gates | ⚠️ authoring parity now has a passing proof demo + smoke/diag gates, broader matrix still expanding | ✅ | workstream-wide | `imui_editor_proof_demo` is now backed by both `imui_adapter_smoke.rs` and a launched diagnostics gate. A component is not really landed until its editor behavior is gated. |

## Summary decisions captured by this matrix

1. We will support imgui-like authoring ergonomics through `fret-imui`, but we will not build a
   second widget implementation tree.
2. We do need to design editor components ourselves, but as Fret-native editor surfaces
   (`DragValue`, `PropertyGrid`, `VecNEdit`, `InspectorPanel`, etc.), not as API copies of ImGui.
3. We should introduce imgui-like visuals as a preset/skin over `editor.*` and `workspace.*`,
   not as a hard-coded dependency or a separate component library.
