# Material3 Field Family Production Alignment v1 Handoff

Status: Closed
Last updated: 2026-05-31

## Current State

This lane is closed. Continue from a narrower follow-on if more field-family parity work is needed.

## What Changed

- TextField leading/trailing icon slots now use logical inline edges under RTL.
- TextField icon-driven input padding is centralized in `foundation::field`.
- Select trigger leading icon now has a stable part `test_id`.
- Select trigger icon order has RTL proof in diagnostics.

## Next Best Follow-On

Open a small lane for multiline field padding if Material3 multiline fields need asymmetric icon
padding. That requires checking `TextAreaStyle` ownership before changing recipe code.
