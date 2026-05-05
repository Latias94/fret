# ImUi Editor-Grade Product Closure Goal Completion Audit - 2026-05-04

Status: Not complete. Continue through narrow follow-ons.

This audit maps the active user goal to current repo evidence. It is intentionally stricter than a
test-green summary: a passing verifier only counts when it covers the explicit requirement being
claimed.

## Objective Restatement

The active goal asks for:

1. Continue on the `main` branch.
2. Use workstream/TODO/milestone documentation when useful.
3. Apply fearless refactoring: remove unsuitable design and compatibility ballast instead of
   preserving it by inertia.
4. Compare the IMUI lane against the local Dear ImGui reference under `repo-ref/imgui`.
5. Preserve the clean Fret architecture: `fret-imui` stays thin, policy lives in
   `fret-ui-kit::imui` / `fret-ui-editor`, and runtime mechanism stays in `crates/fret-ui`.
6. Deliver a user-usable IMUI layer.
7. Do not mark the overall goal complete until the evidence covers the full editor-grade target.

## Prompt-To-Artifact Checklist

| Requirement | Current evidence | Verdict |
| --- | --- | --- |
| Work on `main` | `git branch --show-current` returned `main`. | Met |
| Workstream tracking exists | `docs/workstreams/README.md` validates with 321 dedicated directories after the text-model splits, color edit follow-ons, color-edit model/popup/numeric/picker/preview/swatches splits, alpha preview options, color drag/drop payloads, and debug-draw follow-ons. | Met |
| Fearless refactor removes obsolete shape | `ecosystem/fret-imui/src/tests/models_text.rs` was retired after moving coverage into capability modules. | Met for this slice |
| Dear ImGui reference was used | `repo-ref/imgui/imgui.h` and `repo-ref/imgui/imgui.cpp` exist; `docs/audits/imui-imgui-gap-audit-2026-04-22.md` records the local snapshot and gap analysis. | Met |
| Keep mechanism/policy boundaries clean | `docs/audits/imui-imgui-gap-audit-2026-04-22.md` keeps `fret-imui` thin and routes policy to `fret-ui-kit::imui` / `fret-ui-editor`; `python tools/check_layering.py` passes. | Met |
| User-facing IMUI lane exists | `ecosystem/fret/src/lib.rs` exposes `fret::imui::{prelude, kit, editor, docking}`; cookbook/examples include `imui_action_basics` and `imui_editor_controls_basics`. | Partially met |
| Text input depth is reviewable | `cargo nextest run -p fret-imui models_text --no-fail-fast` runs 26 tests across picker/filter/mode/command/textarea/basic/lifecycle/identity modules and passes. | Met for this subsystem |
| Editor `ColorEdit` depth is materially usable and reviewable | Popup presets, alpha policy/preview modes, typed color drag/drop payloads, AlphaBar, HSV picker, numeric readout, editable RGB/HSV numeric input, per-control popup defaults, and the internal model/popup/numeric/picker/preview/swatches splits are tracked by closed 2026-05-04/2026-05-05 follow-ons. | Met for this subsystem |
| Debug draw baseline, shape floor, stroke policy, clip stack, and image overlays exist | `docs/workstreams/imui-debug-draw-baseline-v1/CLOSEOUT_AUDIT_2026-05-04.md` records the canvas-backed line/rect/filled-rect/text helper; `docs/workstreams/imui-debug-draw-shape-primitives-v1/CLOSEOUT_AUDIT_2026-05-04.md` adds polyline, triangle, and circle primitives; `docs/workstreams/imui-debug-draw-stroke-style-v1/CLOSEOUT_AUDIT_2026-05-04.md` adds cap/join/miter/dash stroke policy; `docs/workstreams/imui-debug-draw-clip-stack-v1/CLOSEOUT_AUDIT_2026-05-04.md` adds push/pop clip rect commands; `docs/workstreams/imui-debug-draw-image-overlay-v1/CLOSEOUT_AUDIT_2026-05-04.md` adds registered image, image-region, SVG image, and SVG mask icon overlays. | Met for styled clipped image/shape floor only |
| Docking P3 local non-interactive gates refreshed | `docs/workstreams/docking-multiwindow-imgui-parity/M12_LOCAL_NONINTERACTIVE_GATE_REFRESH_2026-05-04.md` records green source-policy, manifest, and non-GUI behavior gates; the launched bounded campaign timed out and is not acceptance evidence. | Met for non-interactive gates only |
| Full Dear ImGui-class maturity | The current audit still names missing draw-list parity, full color-picker depth, and OS-window multi-viewport hand-feel. | Not met |

## Current Strengths

- The public authoring lane is coherent: authors can import through `fret::imui::{prelude, kit,
  editor, docking}` instead of directly depending on lower-level crates.
- Single-line and multiline text policy is materially deeper than the older baseline: read-only,
  password paint, select-all-on-focus, named/custom filters, command routing, undo/redo routing,
  completion/history picker UI, keyboard navigation, active-descendant semantics, and lifecycle
  signals have focused proof.
- The `models_text.rs` aggregate hazard is gone. Text-model coverage now lives in:
  - `models_text_picker.rs`
  - `models_text_filters.rs`
  - `models_text_modes.rs`
  - `models_text_commands.rs`
  - `models_text_area.rs`
  - `models_text_basic.rs`
  - `models_text_lifecycle.rs`
  - `models_text_identity.rs`
- Editor `ColorEdit` is no longer a popup stub: the current sequence covers presets,
  alpha-preserving RGB edits, alpha preview modes, color drag/drop payloads, AlphaBar, HSV picker affordances,
  numeric readout, editable RGB/HSV numeric entry, and per-control popup defaults without moving
  editor policy into `fret-imui` or the runtime. The follow-on model and popup splits now keep
  parsing, formatting, HSV/RGB conversion, coordinate math, popup UI composition, editable numeric
  row commit handling, HSV/SV/Hue and AlphaBar composition, gradient/thumb helpers, and popup-local
  pointer helpers, checkerboard/fill-layout/color-preview helpers, and preset swatch activation
  out of the public control wiring file.
- A small canvas-backed `debug_draw` baseline now exists in `fret-ui-kit::imui`, with a first shape
  floor for polylines, triangles, circles, explicit cap/join/miter/dash stroke policy, and a
  push/pop clip-rect stack, plus image/SVG overlay commands for already-owned resources. This gives
  demos an immediate-style debug visualization path without treating `fret-imui` as a renderer.

## Missing Or Weakly Verified Requirements

- **Full draw-list parity is not implemented.** Dear ImGui's `DrawList` family is now represented
  by a canvas-backed baseline lane, but not by full `DrawList` parity yet.
- **Multi-viewport OS-window hand-feel remains a separate docking/runner gap.** The current repo
  has in-window floating and docking proofs, but Dear ImGui-style OS-window viewport behavior still
  depends on the active docking/multiwindow parity lane. M12 refreshed local non-interactive gates,
  but the launched bounded campaign timed out and is not counted as passing evidence.
- **Full `ColorPicker4` / `ColorEdit4` parity is not closed.** The editor `ColorEdit` popup has
  moved beyond a stub, including alpha, HSV, numeric-readout, and editable numeric-input slices, but
  history/palette customization, HueWheel fidelity, eyedropper behavior, and full picker polish
  remain narrower follow-ons.
- **Other IMUI mega-tests still deserve decomposition.** `models_text.rs` is retired, but
  `interaction.rs`, `models.rs`, `floating.rs`, and `popup_hover.rs` still concentrate broad
  behavior coverage.
- **A user-usable layer exists, but "ability can reach imgui" is not complete.** Current evidence
  supports a useful Fret-native IMUI lane, not full Dear ImGui parity.

## Verification Snapshot

Commands run for the final text-model split:

```bash
cargo fmt --package fret-imui
cargo nextest run -p fret-imui models_text --no-fail-fast
python tools/check_layering.py
python tools/check_workstream_catalog.py
python -m json.tool docs/workstreams/imui-models-text-final-test-split-v1/WORKSTREAM.json
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```

All passed.

Additional focused gates recorded later on 2026-05-04:

```bash
cargo nextest run -p fret-ui-editor --features imui color_edit --no-fail-fast
cargo nextest run -p fret-ui-editor --features imui --test imui_surface_policy --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui debug_draw --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --no-fail-fast
cargo nextest run -p fret-ui-kit --features imui --test imui_response_contract_smoke --no-fail-fast
python tools/gate_imui_workstream_source.py
python tools/check_workstream_catalog.py
python tools/check_layering.py
python .agents/skills/fret_skills.py validate --strict --check-anchors --check-symbols
git diff --check
```

The docking local non-interactive refresh also passed the manifest and non-GUI gates listed in
`docs/workstreams/docking-multiwindow-imgui-parity/M12_LOCAL_NONINTERACTIVE_GATE_REFRESH_2026-05-04.md`.

## Next Concrete Follow-Ons

1. Pick one product-facing missing capability rather than continuing generic helper growth:
   color history/HueWheel/eyedropper depth or docking multi-window hand-feel.
2. Keep the implementation in the correct owner layer:
   `fret-ui-editor` for editor controls, `fret-docking`/runner crates for multi-window, and a
   dedicated ecosystem lane for debug draw if it becomes a first-party need.
3. Continue test decomposition only where it removes real refactor risk; do not recreate broad
   aggregate files.
