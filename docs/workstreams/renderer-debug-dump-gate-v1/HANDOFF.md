# Renderer Debug Dump Gate v1 - Handoff

Status: Closed
Last updated: 2026-05-18

## Current State

This lane is complete. Native renderer debug dump frame gating is centralized in
`debug_dump_gate`, while dump-specific naming and JSON assembly remain in their owners.

## Important Invariant

Do not share one-shot `AtomicBool` state across dump owners. Render-plan dumps and text dumps must
continue to gate independently.

## Future Work

If another renderer dump surface grows the same `*_DUMP`, `*_DUMP_FRAME`,
`*_DUMP_AFTER_FRAMES`, and `*_DUMP_EVERY` pattern, use `DumpFrameEnv` instead of copying parser
logic.
