# shadcn Menu/Select Policy Follow-on v1

Status: Closed
Last updated: 2026-05-17

Status note (2026-05-17): this narrow follow-on is closed by
`CLOSEOUT_AUDIT_2026-05-17.md`. The pointer-open + ArrowDown Select contract is resolved in
`SMS-010`; broader roving/typeahead, submenu intent, and dismissal/focus-restore cleanup should
start as narrower follow-ons only when fresh cross-surface evidence exists.

## Why This Lane Exists

ASF-060 in `architecture-surface-fearless-refactor-v1` proved the first shared menu/select policy
owner by moving input-modality-gated entry focus into `fret-ui-headless::entry_focus`. That was
enough for the architecture-surface lane, but it also exposed remaining shadcn recipe policy drift.

The immediate signal was `select_keyboard_navigation`: after pointer-open + ArrowDown + Enter, the
implementation selected the first enabled item (`"apple"`), while the integration test expected the
second item (`"banana"`). `SMS-010` resolved that conflict with source-backed behavior instead of
incidental edits inside the broader architecture lane.

The older `menu-surfaces-alignment-v1` lane is completed and focused on OS/in-window menubar MVP
behavior. This follow-on is narrower: shadcn select/dropdown/context/menubar policy ownership and
conformance.

## Authority

- `docs/adr/0094-menu-open-modality-and-entry-focus.md`
- `docs/adr/0154-ecosystem-crate-taxonomy-glue-and-ui-kit-split-v1.md`
- `docs/reference-stack-ui-behavior.md`
- `docs/workstreams/architecture-surface-fearless-refactor-v1/EVIDENCE_AND_GATES.md`
- Local references under `repo-ref/primitives`, `repo-ref/base-ui`, and `repo-ref/ui` when
  resolving Radix/Base UI/shadcn behavior.

## Target State

- Select pointer-open and keyboard-open navigation semantics are explicit, source-backed, and
  covered by focused gates.
- Shared pure policy lives in `fret-ui-headless`.
- Runtime/a11y/model adapters live in `fret-ui-kit::primitives`.
- shadcn recipe files consume those owners and keep visual taxonomy, styling, and part composition.
- Any old expectations that contradict the chosen source-backed behavior are updated or deleted with
  evidence, not preserved as compatibility.

## Out Of Scope

- Reopening the completed OS/in-window `menu-surfaces-alignment-v1` lane.
- Visual redesign of shadcn menu/select surfaces.
- Full rewrite of the large recipe files without a concrete behavior owner and gate.

## Shipped Repro

```bash
cargo test -p fret-ui-shadcn --locked --test select_keyboard_navigation -j 1
```

Observed during ASF-060: the command ran and failed because the implementation selected `"apple"`
while the test expected `"banana"` after pointer-open + ArrowDown + Enter. After `SMS-010`, the same
gate passes and the focused unit tests lock the active-descendant transition.
