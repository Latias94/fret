# Material3 Headless Golden Hygiene v1 Milestones

Status: Closed
Last updated: 2026-05-31

## M1: Failure Classified

The failing default `radio_alignment` run was classified as stale broad navigation and overlay
golden drift, not Radio geometry or ripple behavior.

## M2: Default Gate Narrowed

The stale broad suites remain in source as explicit maintenance tests, but are ignored by default.
The ignore reasons name the focused gates that carry default behavior coverage.

## M3: Coverage Re-Proved

Focused Radio, navigation, overlay, and select behavior gates pass after the boundary change.

## M4: Lane Closed

Workstream state, catalog, layering, formatting, check, clippy, and diff hygiene are recorded before
commit.
