# Material 3 Snackbar Parts Selector Packet v1 - Closeout Audit

Date: 2026-05-28
Status: Closed

## Scope

Close `M3CAS-070-F3` by adding kit-level toast part selectors for action/close automation while
keeping Material Snackbar as a request skin over the shared toast layer.

## Completion Evidence

- Kit renderer derives `action`, `cancel`, and `close` ids from `ToastRequest::test_id`.
- Kit toast semantics snapshot proves root/action/cancel/close ids are present when all parts are
  rendered.
- Material automation proves `Snackbar::test_id` reaches live root/action/close selectors through
  `SnackbarHost`.
- The component matrix and overlay feedback packet mark the follow-on resolved.

## Boundary Audit

- `fret-ui-kit`: owns toast viewport, action/cancel/close rendering, action dispatch, dismissal,
  live-region semantics, and part-id derivation.
- `fret-ui-material3`: owns Material snackbar request skinning and forwards the root id.
- `fret-ui-material3::foundation`: unchanged.
- `crates/*`: unchanged; no mechanism gap was found.

## Gates

Required closeout gates are recorded in `EVIDENCE_AND_GATES.md`. This audit is valid only with fresh
passing command evidence in the current worktree.

## Follow-On Split

Message/supporting-text toast part selectors remain consumer-driven. They were not part of the
closed follow-on and should not be added without a concrete automation requirement.
