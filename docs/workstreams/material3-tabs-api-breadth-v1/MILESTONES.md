# Material3 Tabs API Breadth v1 - Milestones

Status: Active
Last updated: 2026-05-30

## M0 - Lane Setup

Open the workstream with source-backed facts for primary vs secondary tabs and a narrow task ledger.

Exit criteria:

- Workstream docs exist.
- Secondary tabs source gap is classified as API breadth plus Compose-backed aliasing.
- Catalog and JSON gates are listed.

## M1 - Secondary Variant Slice

Add the first implementation slice for secondary tabs without duplicating the whole component.

Exit criteria:

- `TabsVariant::Secondary` is public and opt-in.
- Primary remains the default.
- Secondary fixed and scrollable indicators use full tab width.
- Primary fixed indicator keeps content-sized geometry.
- v30 theme config resolves every new literal Material token used by the slice.

## M2 - Closeout

Close this narrow lane once the secondary variant is landed and tested, or split richer tab breadth
into separate lanes.

Exit criteria:

- TODO reviews are current.
- Evidence log includes fresh command results.
- Remaining work is explicitly out of scope or assigned to follow-ons.
