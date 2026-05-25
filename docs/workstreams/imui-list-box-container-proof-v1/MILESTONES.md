# IMUI List Box Container Proof v1 - Milestones

Status: Closed
Last updated: 2026-05-25

## M0 - Boundary Locked

Exit criteria:

- The closed no-helper-widening collection verdict remains intact.
- The new lane names exactly one helper shape: ListBox container.

## M1 - Container Shipped

Exit criteria:

- [x] `list_box_with_options` exists.
- [x] `ListBoxOptions` covers only layout, scroll, diagnostics, and multiselectable semantics.
- [x] Children can host existing selectable rows.

## M2 - Proof Closed

Exit criteria:

- [x] Focused composition test passes.
- [x] Source-policy gate rejects generic collection helper drift.
- [x] Catalog, JSON, format, and whitespace gates pass.
