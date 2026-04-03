# Workstream assumptions-first reopen

Use this note when resuming an existing lane in a large workstream folder.

The goal is to turn first-open reading into a small, evidence-backed assumption set before asking
the user questions or editing code.

## Why this exists

Large workstream folders create two common failures:

- the agent reads one stale TODO and treats it as the whole lane state,
- or the agent asks the user questions that current docs already answer.

Assumptions-first reopen keeps the first pass bounded:

1. read the lane,
2. state what you now believe,
3. attach evidence,
4. mark confidence,
5. ask only about the residual uncertainty.

## Output shape

Before coding, write 3-7 assumptions in this format:

- `Area`
- `Assumption`
- `Evidence`
- `Confidence`
  - `Confident`
  - `Likely`
  - `Unclear`
- `Consequence if wrong`

Example:

- Area: lane status
  - Assumption: this lane is still active and should continue rather than fork.
  - Evidence: `WORKSTREAM.json`, `CURRENT_STATUS_AND_PRIORITIES.md`, `TODO.md`
  - Confidence: Confident
  - Consequence if wrong: work could be added to a historical lane instead of a new follow-on.

## Confidence rules

- `Confident`
  - directly supported by `WORKSTREAM.json`, a closeout note, or an explicit current-status doc
- `Likely`
  - supported by multiple docs, but not stated as an explicit lane rule
- `Unclear`
  - docs conflict, or the lane leaves the question unresolved

Bias toward `Unclear` when the evidence is thin.

## Ask-user rule

Only ask the user about:

- `Unclear` assumptions,
- conflicts between a closeout note and older execution docs,
- or preference choices that the lane intentionally leaves open.

Do not ask the user to restate what current lane docs already settle.

## Continue vs follow-on rule

When assumptions suggest:

- `active` or `maintenance` lane → continue current lane by default
- `closed` or `historical` lane → propose a narrow follow-on by default

Do not reopen a closed lane just because old checklist rows still exist.

## Resume recipe

1. Read repo stance and `WORKSTREAM.json` when present.
2. Read the lane's authoritative docs.
3. Write the assumptions set.
4. Resolve only the `Unclear` items with the user when needed.
5. Pick one smallest slice from `TODO.md` / `MILESTONES.md`.
6. Name one repro, one gate, and one evidence set before editing code.
