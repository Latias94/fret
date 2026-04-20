# UiCx Compat Alias Release Retirement v1 — TODO

Status: Closed

## Baseline evidence

- [x] UICX-RR-001 Record why the previous broad lane stays closed and why this narrower lane exists.
- [x] UICX-RR-002 Freeze the current `UiCx*` inventory across:
  - root compatibility aliases,
  - explicit app/advanced exports,
  - and deprecated hidden carrier aliases.
- [x] UICX-RR-003 Freeze the current canonical teaching posture on
  `AppComponentCx<'a>` / `AppRenderCx<'a>` / `AppRenderContext<'a>`.

## Release policy

- [x] UICX-RR-010 Decide the release-window posture for the explicit compatibility aliases:
  - accepted verdict on 2026-04-20: remove now instead of carrying a deprecated release window
- [x] UICX-RR-011 Decide whether the hidden deprecated carriers retire:
  - accepted verdict on 2026-04-20: retire in the same reviewed slice as the explicit aliases
- [x] UICX-RR-012 Write the explicit removal criteria:
  - accepted verdict on 2026-04-20: no further downstream migration window is required for this
    repo; the publish-facing release note must instead carry the explicit breaking-change callout

## Implementation / migration

- [x] UICX-RR-020 If the verdict is retain, keep deprecation wording and release docs consistent.
  - N/A after the delete-now verdict.
- [x] UICX-RR-021 If the verdict is remove, delete the chosen alias set in one explicit reviewed
  slice or an explicitly justified split slice.
- [x] UICX-RR-022 Refresh tests, source-policy gates, and release preflight after the verdict
  lands.

## Closeout

- [x] UICX-RR-030 Close the lane with one explicit verdict:
  - removed with release-facing evidence and migration notes.
