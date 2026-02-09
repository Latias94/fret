# `fret-executor`

Async execution and task orchestration helpers for Fret.

This crate provides a portable Inbox + Dispatcher-based pattern for background work:

- UI/runtime state is main-thread only.
- Background tasks send **data-only messages** into an `Inbox`.
- A runner-provided `DispatcherHandle` is used to wake the app at a driver boundary.

## Status

Experimental learning project (not production-ready).

## Features

- `tokio`: integrate with Tokio as a background spawner (optional)
- `wasm`: integrate with wasm-bindgen-futures on wasm32 (optional)

## When to use

- You need background work (I/O, parsing, indexing) without blocking the UI thread.
- You want a repeatable, testable message-passing boundary (Inbox + drain).

