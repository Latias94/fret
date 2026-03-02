# Workspace TabStrip (Fearless Refactor v1) — Parity Matrix

Legend:

- `Yes` implemented and gated
- `Partial` implemented with known gaps
- `No` not implemented
- `N/A` not applicable for v1

| Capability | Target | Status | Notes |
|---|---:|---:|---|
| End-drop surface (`insert_index == tab_count`) | Zed | Yes | `drop_end` anchor + diag scripts |
| Close does not activate | Zed | Yes | Overflow close gate + click arbitration |
| Overflow menu close vs activate arbitration | Zed | Yes | `overflow_entry.*.close` + activate-hidden gate |
| Reorder within pane | Zed | Yes | Reorder-to-end gates |
| Move tab across panes | Zed | Yes | Cross-pane move gate |
| Active tab always visible | Zed | Yes | `workspace_tab_strip_active_visible_is` gate exists in the suite |
| Pinned tabs | Zed | Partial | Pinned flags + boundary exist; tighten semantics + gates |
| Preview tab slot | Zed | Partial | Preview flags exist; tighten semantics + gates |
| Dirty close confirmation | Zed | No | M3 |
