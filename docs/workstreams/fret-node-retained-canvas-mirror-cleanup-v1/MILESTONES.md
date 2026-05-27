# `fret-node` Retained Canvas Mirror Cleanup (v1) - Milestones

Status: complete
Last updated: 2026-05-27

## Global Success Criteria

- Retained canvas mirrors are named as compatibility state, not hidden top-level authority.
- Store-backed retained canvas sync still works after mirror cleanup.
- Public retained compatibility constructors keep compiling while this lane is active.
- Source-policy tests block accidental reintroduction of top-level retained mirror fields.
- Remaining retained cleanup is split or explicitly deferred at closeout.

## M0 - Scope And Evidence Freeze

Status target: NCM-010 complete
Current status: complete

Done criteria:

- Workstream docs exist and agree on scope.
- The closed runtime/store lane remains untouched.
- First task is a behavior-preserving mirror-boundary cleanup.

## M1 - Retained Canvas Mirror Owner

Status target: NCM-020 complete
Current status: complete

Done criteria:

- `NodeGraphCanvasWith` owns graph/view/editor-config model mirrors through
  `NodeGraphCanvasMirrors`.
- Existing retained constructors and middleware conversion preserve the same models.
- Source-policy coverage names the mirror owner and rejects top-level mirror fields.
- Focused compat tests pass.

## M2 - Store-First Retained Sync Audit

Status target: NCM-030 complete or split
Current status: complete

Done criteria:

- Store-backed retained sync is audited after mirror quarantine.
- At least one redundant mirror update path is deleted or deliberately retained with documented
  compatibility evidence.
- Required compat gates pass.

## M3 - Closeout

Status target: workstream complete or split follow-on
Current status: complete

Done criteria:

- Fresh closeout gates are recorded.
- Remaining retained work is not left only in chat.
- `WORKSTREAM.json` and `HANDOFF.md` identify the next action or mark the lane closed.

Closeout evidence:

- `CLOSEOUT_AUDIT_2026-05-27.md`
- `EVIDENCE_AND_GATES.md`
