# Pressable Clean Geometry Propagation v1 - Milestones

Status: Closed
Last updated: 2026-05-18

## M0 - Scope And Evidence Freeze

Exit criteria:

- The lane is created as a narrow follow-on to `retained-layout-orchestration-v1`.
- `Pressable` side-effect risks are named before runtime code changes.
- The first proof tasks and gates are explicit.
- Workstream catalog and `WORKSTREAM.json` validate.

Primary evidence:

- `docs/workstreams/pressable-clean-geometry-propagation-v1/DESIGN.md`
- `docs/workstreams/pressable-clean-geometry-propagation-v1/TODO.md`
- `docs/workstreams/retained-layout-orchestration-v1/CLOSEOUT_AUDIT_2026-05-18.md`

Status: Complete.

## M1 - Pressable Source Audit

Exit criteria:

- Layout, hit-test, focus, hover, pressed-state, capture, and activation responsibilities are
  mapped to source anchors.
- The audit states whether any side effect requires rerunning `Pressable` layout on width-only clean
  geometry propagation.
- The first RED test shape is confirmed or revised.

Primary evidence:

- `crates/fret-ui/src/declarative/host_widget/layout.rs`
- `crates/fret-ui/src/declarative/host_widget/event/pressable.rs`
- `crates/fret-ui/src/tree/dispatch/hover.rs`
- `crates/fret-ui/src/tree/layout/clean_geometry.rs`
- `docs/workstreams/pressable-clean-geometry-propagation-v1/PGP_020_030_SOURCE_AUDIT_AND_RED_PROOF_2026-05-18.md`

Status: Complete.

## M2 - First RED Proof

Exit criteria:

- A focused test demonstrates the current `Pressable` wrapper propagation gap, or records that the
  current code no longer reproduces it.
- Side-effect invariants are covered by existing or new focused tests.
- The proof is small enough to run during iteration.

Primary gates:

```bash
cargo nextest run -p fret-ui clean_geometry_small_resize_propagates_through_pressable_wrapper --no-fail-fast
cargo nextest run -p fret-ui pressable_on_activate_hook_runs_on_pointer_activation pressable_on_hover_change_hook_runs_on_pointer_move --no-fail-fast
```

Status: Complete. The focused layout proof is RED as expected with
`layout_nodes_performed=2`, while existing activation, hover, and pressed/capture cleanup gates
pass.

## M3 - Minimal Runtime Slice Or No-Change Verdict

Exit criteria:

- If safe, `Pressable` is added to the clean-geometry execution allowlist with no unrelated
  refactor.
- If unsafe or unproven, the lane records a no-change verdict with the blocking side effect or
  evidence gap.
- Layout, interaction, layering, and format gates pass.

Primary gates:

```bash
cargo nextest run -p fret-ui clean_geometry_small_resize_propagates_through_pressable_wrapper --no-fail-fast
cargo nextest run -p fret-ui layout_engine pressable --no-fail-fast
python3 tools/check_layering.py
cargo fmt --check
git diff --check
```

Status: Complete. `Pressable` is now in the clean-geometry execution allowlist, the focused
propagation test passes, and the interaction guard suite remains green.

## M4 - Perf Confirmation And Closeout

Exit criteria:

- UI Gallery resize-jitter evidence is captured or explicitly deferred with a reason.
- The closeout note explains whether `Pressable` remained a hotspot, moved, or was left unchanged.
- `WORKSTREAM.json`, `TODO.md`, `MILESTONES.md`, `EVIDENCE_AND_GATES.md`, and `HANDOFF.md` agree.
- Follow-on candidates are split rather than appended to this lane.

Primary gates:

```bash
target/release/fretboard-dev diag stats <bundle.schema2.json> --sort time --top 20
python3 tools/check_workstream_catalog.py
```

Status: Complete. PGP-050 captured fresh UI Gallery resize-jitter evidence after the `Pressable`
allowlist change. `Pressable` no longer appears in the worst-frame layout hotspot list; the remaining
local owners are `ViewCache`, `Scroll`, and a small `Flex` owner.
