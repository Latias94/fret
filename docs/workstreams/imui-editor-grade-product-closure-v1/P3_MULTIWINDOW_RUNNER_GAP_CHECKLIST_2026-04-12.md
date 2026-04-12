# P3 Multi-Window Runner Gap Checklist - 2026-04-12

Status: focused P3 scope freeze / runner-owned parity checklist

Related:

- `DESIGN.md`
- `TODO.md`
- `MILESTONES.md`
- `EVIDENCE_AND_GATES.md`
- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
- `docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v1.md`

## Purpose

P3 needs one narrow answer before it grows into platform-specific implementation work:

> what short runner/backend parity checklist should this lane freeze so future multi-window hand-feel
> work does not drift back into `imui` helper growth or `crates/fret-ui` contract widening?

This note freezes that checklist without pretending the bounded P3 gate already exists.

## Audited evidence

- `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
- `docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md`
- `docs/workstreams/standalone/imui-imgui-parity-audit-v1.md`
- `apps/fret-examples/src/workspace_shell_demo.rs`
- `DESIGN.md`

## Assumptions-first resume set

1. Confident: the default P3 owners are still `crates/fret-launch`, runner/backend integrations,
   and `ecosystem/fret-docking`, not `crates/fret-ui`.
   Evidence:
   - `DESIGN.md` already freezes the P3 owner split that way,
   - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
     treats hovered-window selection, z-order, click-through payloads, and follow behavior as
     runner/backend work.
   Consequence if wrong:
   - this lane would start leaking platform heuristics into the runtime contract or immediate
     facade surface.
2. Confident: the current parity story is spread across several notes, but the must-not-forget
   behavior families are already stable enough to freeze as one short checklist.
   Evidence:
   - the cross-platform parity note already names hovered-window, peek-behind, transparent payload,
     and DPI/follow concerns,
   - the macOS note turns the same area into concrete hand-feel expectations.
   Consequence if wrong:
   - this slice would be premature and would freeze unstable categories instead of real runner gaps.
3. Likely: the fastest current P3 reopen surface is still the existing shell/docking proof rather
   than a new `imui`-specific demo.
   Evidence:
   - `apps/fret-examples/src/workspace_shell_demo.rs` is already the coherent shell-mounted proof,
   - the docking parity workstream is where the runner/platform evidence already lives.
   Consequence if wrong:
   - the next slice would need a different launched target before it can promote a bounded P3 gate.
4. Confident: this slice should freeze the checklist first and leave the promoted parity gate as a
   separate open task.
   Evidence:
   - `TODO.md` still keeps the bounded P3 gate as the next explicit checkbox,
   - `EVIDENCE_AND_GATES.md` still says P3 cannot claim closure with prose alone.
   Consequence if wrong:
   - this note would pretend a proof package exists when the lane has not named one yet.

## Current gap

The repo already has real multi-window parity evidence, but it is split across:

- the docking parity lane,
- the standalone macOS detail note,
- and the older `imui` parity audit.

That is enough for a maintainer who already knows the history, but not enough for this lane to
reopen P3 quickly and consistently. Without one short checklist, future work risks:

- reopening generic `imui` helper growth to paper over runner issues,
- widening `crates/fret-ui` for platform-specific heuristics,
- or promoting a gate that only proves one platform quirk instead of the full hand-feel budget.

## Frozen P3 runner/backend checklist

From this point forward, the default P3 checklist for this lane is:

1. Hovered-window selection stays runner-owned
   - "window under cursor" quality is a platform capability / routing question,
   - not an `imui` query semantic and not a reason to widen `crates/fret-ui`.
2. Peek-behind while moving a tear-off window stays runner-owned
   - the moving DockFloating payload must not block hover targeting of the window behind it,
   - and the solution belongs in click-through / routing / platform-window behavior.
3. Transparent payload overlap behavior stays runner-owned
   - temporary transparency, hit-test passthrough, and temporary top-most behavior belong to the
     runner window-style path and docking policy,
   - not to immediate helper growth.
4. Mixed-DPI follow-drag correctness stays runner-owned
   - follow behavior, cursor-to-window positioning, and cross-monitor DPI transitions are backend
     semantics,
   - not a runtime contract gap unless ADR-backed evidence proves otherwise.

These four items are the minimum P3 parity budget. Future P3 work may add more detail, but it
should not silently drop any of the four categories above.

## Frozen owner split

For this lane:

- `crates/fret-launch` owns runner integration and tracked multi-window follow behavior,
- runner/backend integrations own hovered-window quality, z-order, cursor positioning, and
  capability-specific degradation,
- `ecosystem/fret-docking` owns docking policy and the translation between dock intent and
  runner-visible window requests,
- `crates/fret-ui` remains the mechanism layer and should not absorb platform heuristics just
  because a runner/backend parity gap is painful,
- and generic `imui` helpers remain out of scope unless a separate proof-budget decision reopens
  them with stronger evidence.

## Current reopen surface and gate posture

Use this bounded posture when reopening P3:

1. Start from the existing shell-mounted proof:
   - `cargo run -p fret-demo --bin workspace_shell_demo`
2. Read parity evidence in this order:
   - `docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md`
   - `docs/workstreams/standalone/macos-docking-multiwindow-imgui-parity.md`
   - `docs/workstreams/standalone/imui-imgui-parity-audit-v1.md`
3. Treat the gate as still open:
   - this slice only freezes what the future bounded P3 gate must explicitly name,
   - it does not claim that the gate already exists.

## Decision

From this point forward:

1. P3 is a runner/backend closure problem by default, not an `imui` API backlog.
2. The short checklist for this lane is now frozen as:
   hovered-window, peek-behind, transparent payload, and mixed-DPI follow-drag.
3. Do not widen `crates/fret-ui` or generic immediate helpers to compensate for those runner gaps
   without ADR-backed evidence that the runtime contract is actually insufficient.
4. The next real P3 slice should promote one bounded parity gate or diag suite that explicitly
   names all four checklist items above.
5. If that work becomes mostly platform implementation, continue in the docking parity lane or a
   narrower runner follow-on instead of bloating this folder.

## Immediate execution consequence

For this lane:

- treat this note as the first-open P3 checklist,
- use it to reject runtime/helper growth that is really a runner parity gap,
- and use the four frozen checklist items as the minimum coverage floor for the next promoted P3
  gate.
