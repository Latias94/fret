# imui shell transparent payload z-order v1 - TODO

Status: closed board

Last updated: 2026-03-30

Tracking doc: `docs/workstreams/imui-shell-transparent-payload-zorder-v1/DESIGN.md`

Milestones: `docs/workstreams/imui-shell-transparent-payload-zorder-v1/MILESTONES.md`

Contract freeze:

- `docs/workstreams/imui-shell-transparent-payload-zorder-v1/M1_CONTRACT_FREEZE_2026-03-30.md`

Predecessor closeout:

- `docs/workstreams/imui-shell-ghost-choreography-v1/CLOSEOUT_AUDIT_2026-03-30.md`

Closeout:

- `docs/workstreams/imui-shell-transparent-payload-zorder-v1/CLOSEOUT_AUDIT_2026-03-30.md`

This board assumes a fearless refactor posture.
Compatibility shims are explicitly out of scope.

## M0 - Successor lane setup

- [x] Open the successor lane immediately after the first shell ghost choreography closeout.
- [x] Record why this lane is not the same as payload ghost visibility.
- [x] Freeze the first intended proof surface and script corpus.

## M1 - Freeze the transparent payload overlap contract

- [x] Decide the intended owner split for runner truth vs docking preview policy.
- [x] Decide the first launched proof surface and primary script gates.
- [x] Record the minimum diagnostics package required for overlap/z-order regressions.
- [x] Record explicit non-goals before implementation starts.

## M2 - First launched proof pass

- [x] Run the transparent payload z-order switch script with bounded artifact capture.
      Result: pass.
      Evidence:
      `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774861703998-19664`.
- [x] Run the large-layout transparent payload z-order switch script with bounded artifact capture.
      Result: reproducible timeout after transparent payload activation / overlap handoff.
      Evidence:
      `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774861703143-49836`
      and auto-dump rerun
      `target/fret-diag/imui-shell-transparent-payload-zorder-v1/sessions/1774862148351-10948`.
- [x] Record whether current diagnostics are sufficient to explain any failure without raw bundle
      inspection.
      Result:
      sufficient for the base passing case and for locating the large-case failure after initial
      transparent payload activation,
      insufficient for final root-cause certainty because the late timeout has no final bundle.
      See
      `docs/workstreams/imui-shell-transparent-payload-zorder-v1/M2_LAUNCHED_PROOF_READ_2026-03-30.md`.

## M3 - Contract delta and gate hardening

- [x] Land only the smallest delta proven necessary by the launched proof.
      Result:
      a diagnostics-only multi-window pointer-session bridge plus implicit `pointer_move`
      migration/completion fixes were sufficient.
- [x] Explain or fix the large-preset stall that begins at or immediately after
      `raise_window(first_seen)` in the late overlap path.
      Result:
      fixed.
      The real blocker was the late `pointer_move` diagnostics loop at step 19, followed by stale
      graph-signature expectations in the script corpus.
- [x] Add or refine diagnostics predicates only if the current bundle fields are insufficient.
      Result:
      no new predicate family was required.
      Existing dock-routing diagnostics were sufficient once script delivery stopped stalling.
- [x] Keep docking-specific policy out of generic recipe layers and out of `fret-ui-kit::imui`.
      Result:
      preserved.
      The landed cross-window delivery path is diagnostics-only and lives in bootstrap/runtime
      plumbing rather than generic component policy layers.

## M4 - Closeout or split

- [x] Capture a closeout audit if the overlap/z-order contract lands cleanly.
      Result:
      `docs/workstreams/imui-shell-transparent-payload-zorder-v1/CLOSEOUT_AUDIT_2026-03-30.md`
- [x] If the proof reveals a broader shell surface gap, split that into a new successor lane rather
      than widening this lane ad hoc.
      Result:
      not needed for this closeout.
      The lane closed through a narrow runtime/diagnostics delta plus script corpus alignment.
