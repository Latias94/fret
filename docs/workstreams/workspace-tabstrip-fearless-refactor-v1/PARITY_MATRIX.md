# Workspace TabStrip (Fearless Refactor v1) — Parity Matrix

Legend:

- `Yes` implemented and gated
- `Partial` implemented with known gaps
- `No` not implemented
- `N/A` not applicable for v1

| Capability | Target | Status | Notes |
|---|---:|---:|---|
| End-drop surface (`insert_index == tab_count`) | Zed | No | Add diag + nextest gates in M1 |
| Close does not activate | Zed | Partial | Policy exists in `fret-ui-kit`; ensure adapters follow |
| Overflow menu close vs activate arbitration | Zed | Partial | Needs adapter conformance + tests |
| Reorder within pane | Zed | No | M2 |
| Move tab across panes | Zed | No | M2 |
| Active tab always visible | Zed | No | Add diagnostics gate in M1 |
| Pinned tabs | Zed | No | M3 |
| Preview tab slot | Zed | No | M3 |
| Dirty close confirmation | Zed | No | M3 |
