# ImUi Debug Draw Command Metadata v1 Milestones

Status: Closed.

## M0 - API Shape

Exit criteria:

- Command kinds are public and stable enough for tests/diagnostics.
- Per-command metadata stays detached from private command payload variants.
- Aggregate metadata separates raster image, SVG, text, clip, and mesh counts.

Result: Complete.

## M1 - Channel-Aware Introspection

Exit criteria:

- Active channel split state can be summarized without mutating or merging the list.
- Summary order matches eventual merge order.

Result: Complete.

## M2 - Evidence

Exit criteria:

- Unit and public smoke tests cover the new surface.
- Workstream/audit indexes record that command metadata is no longer a total gap.
- Residual callback/raw-buffer gaps stay explicit.

Result: Complete.
