# ImUi Text Control Chrome Stability v1 - Milestones

Status: closed
Last updated: 2026-04-28

## M0 - Routing and Evidence

Exit criteria:

- Old closed lanes are read before editing.
- The follow-on owns only IMUI text-control chrome stability.
- Repro, gate, and evidence surfaces are named in `WORKSTREAM.json`.

Status: complete.

## M1 - Compact Text Chrome

Exit criteria:

- IMUI `input_text` and `textarea` use compact IMUI chrome rather than shadcn input recipe chrome.
- Focus does not configure an external `RingStyle`.
- Tests assert the rendered element props, not just source strings.

Status: complete.

## M2 - Closeout

Exit criteria:

- Focused gates pass.
- `EVIDENCE_AND_GATES.md` reflects the executed commands.
- The lane is either closed or left with a specific narrow next slice.

Status: complete.
