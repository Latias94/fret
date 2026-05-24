# Fret Node Low-Level Adapter v1 - Handoff

Updated: 2026-05-25

## Current State

This lane is open and has not moved code yet. It follows ADR 0330 and the retained public-surface
exit. The first task is an audit of retained context usage inside
`ecosystem/fret-node/src/ui/canvas/widget/**`.

## Next Command

```bash
rg -n "EventCx|LayoutCx|PaintCx|PrepaintCx|SemanticsCx|CommandCx|Widget" ecosystem/fret-node/src/ui/canvas/widget
```

Pick one behavior family from that output and turn it into an adapter proof.
