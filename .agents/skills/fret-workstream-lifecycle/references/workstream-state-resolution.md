# Workstream state resolution

Use this note when an existing workstream folder has accumulated design notes, audits, and
closeout records.

## First-open order

1. Repo-wide stance
   - `docs/roadmap.md`
   - `docs/workstreams/README.md`
   - `docs/todo-tracker.md`
2. Machine-readable lane state
   - `WORKSTREAM.json` when present
3. Lane positioning
   - `README.md` or `<slug>.md`
4. Current target surface
   - `DESIGN.md`
   - `TARGET_INTERFACE_STATE.md` when present
5. Execution docs
   - `TODO.md`
   - `MILESTONES.md`
   - `EVIDENCE_AND_GATES.md`
6. Shipped verdict
   - `CLOSEOUT_AUDIT_*.md`
   - `FINAL_STATUS.md`
   - explicit top-of-file status notes
7. Supporting evidence only after the above
   - audits
   - migration matrices
   - inventories
   - parity notes

## Precedence rules

- `WORKSTREAM.json` is a machine-readable index, not a second source of truth.
- If `WORKSTREAM.json` conflicts with a closeout audit or explicit status note, fix the state file.
- A closeout audit or explicit status note beats an older TODO checklist.
- The roadmap/workstreams index beats stale wording buried in an old design note.
- `EVIDENCE_AND_GATES.md` beats guessed commands from memory.
- `TARGET_INTERFACE_STATE.md` beats early brainstorming language in `DESIGN.md` when both exist.

## Continue vs follow-on

Continue the existing lane when:

- the roadmap still treats it as active or maintenance,
- `TODO.md` / `MILESTONES.md` still define the next slice,
- and there is no explicit closeout saying broad scope is frozen.

Create or propose a narrower follow-on when:

- a closeout audit says the lane is closed,
- the requested change widens scope beyond maintenance/docs/gates,
- or fresh evidence shows a new problem not covered by the closed lane’s target state.
