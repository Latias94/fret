# P3 Execution Priority Review - 2026-05-06

Status: priority review; no new implementation lane opened from this note
Last updated: 2026-05-18

## Decision

Keep the current IMUI architecture and owner split. Do not turn the P3 catalogs into a broad
"finish all Dear ImGui widgets" backlog.

The current P3 catalog notes are useful comparison documents, but they are not the execution order
for implementation work. The execution order should be:

1. **Product closure / golden editor workflow**
   - Make the editor-grade path feel coherent across cookbook, `imui_editor_proof_demo`,
     `workspace_shell_demo`, diagnostics, and docking.
   - Owner: app/proof surfaces plus `fret`, `fret-ui-editor`, `fret-docking`, and diagnostics
     consumers.
   - Reason: Dear ImGui feels mature because common editor workflows compose, not because every API
     category is mirrored one-for-one.
2. **Runner/backend multi-window hand-feel**
   - Continue in `docking-multiwindow-imgui-parity`.
   - Owner: runner/backend integrations plus `fret-docking`; not `crates/fret-ui` and not generic
     `fret-ui-kit::imui`.
   - Reason: tear-out, follow-drag, mixed-DPI, hover routing, and platform degradation are the
     largest remaining Dear ImGui-grade editor-feel risks.
3. **Diagnostics / DevTools discoverability**
   - Continue through `diag-fearless-refactor-v2` and the DevTools GUI maintenance note.
   - Owner: `fret-diag`, `fret-bootstrap`, `apps/fret-devtools`, `apps/fret-devtools-mcp`.
   - Reason: Fret is strong on reproducible evidence, but Dear ImGui remains stronger at
     always-available demo/metrics/debug discoverability.
4. **Proof-led API/helper widening**
   - Only after two real first-party proof surfaces need the same helper.
   - Owner depends on the helper: `fret-imui` for policy-light authoring control flow,
     `fret-ui-kit::imui` for generic policy-heavy widgets, `fret-ui-editor` for editor controls,
     `fret-docking` for docking.
   - Reason: current component coverage is already broad enough for the active editor proof.
     Blindly mirroring Dear ImGui API names would add maintenance cost without improving the
     architecture.

5. **Performance discipline**
   - Keep runtime smoothness work in the dedicated perf workstreams:
     `docs/workstreams/ui-perf-zed-smoothness-v1/ui-perf-zed-smoothness-v1.md`,
     `docs/workstreams/diag-perf-attribution-v1/diag-perf-attribution-v1.md`, and the
     `docs/workstreams/imui-imgui-gap-closure-v1/P4_PERFORMANCE_ALIGNMENT_REVIEW_2026-05-06.md`
     comparison note.
   - Reason: performance gaps are review and attribution problems first, not a signal to widen the
     IMUI widget backlog.

## What Is Already Correct

- `fret-imui` is still policy-light and depends only on `fret-authoring` + `fret-ui`.
- `fret-ui-kit::imui` is the right owner for generic immediate widget policy, response signals,
  options, debug draw, menus, popups, tables, tabs, drag/drop, child regions, and virtual lists.
- `fret-ui-editor::imui` remains a thin adapter over declarative editor controls and composites.
- `fret::imui` is the right app-facing optional lane; `fret::app::prelude::*` should stay
  declarative-first.
- The closed `imui-debug-draw-owner-split-v1` follow-on proves that structural cleanup belongs in
  narrow owner lanes, not in a broad parity bucket.

## Priority Corrections

Use this interpretation when resuming the lane:

- The public/component/design/porting/child/collection P3 notes are **readiness catalogs**.
  They compare surfaces and set thresholds.
- They do not justify starting implementation by list order.
- A new implementation lane should start only when it can name:
  - the exact behavior or API,
  - the correct owner layer,
  - two proof surfaces unless it is a thin adapter over an existing declarative control,
  - one focused gate,
  - and the Dear ImGui reference axis being intentionally matched or rejected.

## Current Candidate Verdicts

| Candidate | Current priority | Verdict |
| --- | --- | --- |
| Public facade/API surface | Guardrail | Keep stable; add helpers only after proof budget passes |
| Component surface breadth | Guardrail | Catalog is adequate; do not start a widget backlog |
| Design/style parity | Guardrail | Keep token/preset path; do not copy mutable style stack |
| Porting sugar | Candidate-only | Wait for repeated pain across two proof surfaces |
| Collection helper | Candidate-only | Current behavior remains app-owned despite second-surface evidence |
| Child-region depth | Candidate-only | Manual resize and height-auto layout now have proof; visibility-return, nav flattening, and width/always auto-resize need dedicated proof |
| Diagnostics discoverability | Product priority | Continue on diagnostics/DevTools lanes, not runtime/API widening |
| Multi-window hand-feel | Execution priority | Continue in `docking-multiwindow-imgui-parity` |

## Review Gates

Use these to keep the priority map honest:

```powershell
python tools/audit_crate.py --crate fret-imui
python tools/audit_crate.py --crate fret-ui-kit
python tools/audit_crate.py --crate fret-ui-editor
python tools/audit_crate.py --crate fret
python tools/gate_imui_workstream_source.py
python tools/gate_imui_facade_teaching_source.py
python tools/check_workstream_catalog.py
git diff --check
```

## Gate Results

2026-05-06 local results:

- `python tools/audit_crate.py --crate fret-imui` passed and confirmed the small
  `fret-authoring` + `fret-ui` dependency posture.
- `python tools/audit_crate.py --crate fret-ui-kit` passed and confirmed the policy-heavy IMUI
  owner surface, including `src/imui/facade_writer.rs`.
- `python tools/audit_crate.py --crate fret-ui-editor` passed and confirmed editor controls plus
  the thin `src/imui.rs` adapter surface.
- `python tools/audit_crate.py --crate fret` passed and confirmed the wide app-facing facade
  posture.
- `python tools/gate_imui_workstream_source.py` passed.
- `python tools/gate_imui_facade_teaching_source.py` passed.
- `python tools/check_workstream_catalog.py` passed.
- `git diff --check` passed with only the existing Git line-ending warning for the touched Python
  gate file.
