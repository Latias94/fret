# ImUi Facade Internal Modularization v1 - Evidence & Gates

Goal: keep internal `fret-ui-kit::imui` module motion tied to one current proof build, one focused
test floor, and one explicit source-policy gate while the public facade stays frozen.

Status note (2026-05-01): this lane is now closed. The gate set below remains the closeout proof
surface for the landed owner decomposition; future structural or policy work should start a
narrower follow-on instead of reopening this folder by default.

## Evidence anchors (current)

- `docs/workstreams/imui-facade-internal-modularization-v1/DESIGN.md`
- `docs/workstreams/imui-facade-internal-modularization-v1/M0_BASELINE_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-facade-internal-modularization-v1/M1_OPTIONS_RESPONSE_SLICE_2026-04-21.md`
- `docs/workstreams/imui-facade-internal-modularization-v1/M2_INTERACTION_RUNTIME_SLICE_2026-04-21.md`
- `docs/workstreams/imui-facade-internal-modularization-v1/M3_ROOT_FACADE_HUB_SLICE_2026-04-21.md`
- `docs/workstreams/imui-facade-internal-modularization-v1/M4_FACADE_WRITER_GLUE_SLICE_2026-04-21.md`
- `docs/workstreams/imui-facade-internal-modularization-v1/CLOSEOUT_AUDIT_2026-04-21.md`
- `docs/workstreams/imui-facade-internal-modularization-v1/TODO.md`
- `docs/workstreams/imui-facade-internal-modularization-v1/MILESTONES.md`
- `docs/workstreams/imui-facade-internal-modularization-v1/WORKSTREAM.json`
- `docs/workstreams/README.md`
- `docs/roadmap.md`
- `docs/todo-tracker.md`
- `tools/gate_imui_workstream_source.py`
- `ecosystem/fret-ui-kit/src/imui.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_writer.rs`
- `ecosystem/fret-ui-kit/src/imui/facade_support.rs`
- `ecosystem/fret-ui-kit/src/imui/floating_options.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime.rs`
- `ecosystem/fret-ui-kit/src/imui/interaction_runtime/`
- `ecosystem/fret-ui-kit/src/imui/options.rs`
- `ecosystem/fret-ui-kit/src/imui/options/`
- `ecosystem/fret-ui-kit/src/imui/response.rs`
- `ecosystem/fret-ui-kit/src/imui/response/`
- `apps/fret-examples/src/lib.rs`

## First-open repro surfaces

Use these before opening older historical `imui` notes in depth:

1. Crate snapshot / hotspot scan
   - `python3 tools/audit_crate.py --crate fret-ui-kit`
2. First-party proof build that should keep compiling under unchanged public surface
   - `cargo build -p fret-demo --bin imui_editor_proof_demo`
3. Interaction floor for the runtime-owned state machine
   - `cargo nextest run -p fret-imui --no-fail-fast`
4. Focused public-surface floor
   - `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast`
5. Lane-local source-policy floor
   - `python tools/gate_imui_workstream_source.py`

## Current gates

### Lane-local source-policy gate

- `python tools/gate_imui_workstream_source.py`

This gate proves:

- the new lane stays explicit in the workstream map, roadmap, and todo tracker,
- the lane remains a follow-on of the immediate-mode umbrella,
- and the lane keeps the landed structural slices explicit instead of becoming an undocumented
  cleanup bucket.

### Interaction floor

- `cargo nextest run -p fret-imui --no-fail-fast`

This floor proves:

- the current immediate interaction behavior still compiles and passes after the `interaction_runtime`
  owner split,
- and the lane is no longer relying only on facade smoke once the internal state machine moved.

### Public-surface floor

- `cargo nextest run -p fret-ui-kit --features imui --test imui_adapter_seam_smoke --test imui_response_contract_smoke --no-fail-fast`

This floor proves:

- the public `fret-ui-kit::imui` adapter seam still compiles and behaves as before,
- and `ResponseExt` / related helper contracts still satisfy the existing smoke expectations after
  internal module motion.

### First-party proof build

- `cargo build -p fret-demo --bin imui_editor_proof_demo`

This keeps a real first-party immediate consumer compiling while the internal module graph changes.

### Lane hygiene gates

- `python3 tools/check_layering.py`
- `git diff --check`
- `python3 tools/check_workstream_catalog.py`
- `python3 -m json.tool docs/workstreams/imui-facade-internal-modularization-v1/WORKSTREAM.json > /dev/null`

## Future follow-on rule

Reuse this proof floor only if a future narrow follow-on is explicitly about a subset of the landed
writer/runtime/options/response owner split.
Do not reopen this folder for additive helper or policy pressure.
