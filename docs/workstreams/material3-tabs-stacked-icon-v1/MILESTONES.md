# Material3 Tabs Stacked Icon v1 - Milestones

Status: Closed
Last updated: 2026-05-30

## M0 - Source Decision

Complete. The lane uses Compose `TabBaselineLayout` as the runtime truth and documents the Material
Web 64px token divergence.

## M1 - Implementation

Complete. `TabItem` now has placement-aware icon API and stacked icon tabs raise the row to 72px.

## M2 - Evidence

Complete. Focused tabs geometry tests and v30 token tests cover the shipped behavior.
