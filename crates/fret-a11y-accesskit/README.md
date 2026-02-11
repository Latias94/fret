# `fret-a11y-accesskit`

AccessKit accessibility bridge for Fret UI trees.

This crate converts Fret's portable semantics snapshots (`fret-core`) into AccessKit node updates.
It is used by native runners to expose UI accessibility to OS assistive technologies.

## Status

Experimental learning project (not production-ready).

## When to use

- You are wiring accessibility for a native runner.
- You need to inspect or validate the semantics tree mapping.

If you are building components, you usually do not depend on this crate directly.

