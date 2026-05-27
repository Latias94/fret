# Material 3 Snackbar Parts Selector Packet v1 - Milestones

Status: Closed
Last updated: 2026-05-28

## M0 - Packet Opened

Exit criteria:

- Boundary is explicit: kit owns toast part selectors, Material owns root forwarding.
- Truth set names the action/close selector outcomes.
- Workstream JSON is valid.

Result: Complete.

## M1 - Kit Selector Surface

Exit criteria:

- Rendered toast action/cancel/close buttons derive ids from `ToastRequest::test_id`.
- Missing parts do not synthesize ids.
- Kit semantics snapshot gate proves the selector surface.

Result: Complete.

## M2 - Material Closeout

Exit criteria:

- Material automation proves `Snackbar::test_id` reaches root/action/close live selectors.
- Matrix and overlay packet mark `M3CAS-070-F3` resolved.
- Catalog, JSON, formatting, focused tests, check, and clippy gates pass.

Result: Complete.
