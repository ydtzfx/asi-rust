# Agent Protocol v1.1 adapter

This branch adds an isolated conformance adapter in `crates/asi-ai-sdk/src/protocol_v1_1.rs`.

## Boundary

The adapter does **not** migrate or replace Rust business logic. In particular, M4 does not change:

- `ToolLoopAgent` execution behavior;
- provider/model implementations;
- server routes or deployment logic;
- existing business tools;
- database schemas or production persistence;
- memory, evolution, automation, or orchestration modules.

Existing Rust primitives already provide production foundations for several contract behaviors: `asi-lib::safe_path` confines filesystem paths, `agent::tool_loop::CancelToken` provides cooperative cancellation, `ToolLoopAgent::max_steps` bounds execution, and `ToolMap` is the runtime tool registry.

The v1.1 adapter defines the language-neutral observable contract used by the shared conformance vectors. Its in-memory `IdempotencyJournal` exists only to specify replay behavior for C013; it is **not** a claim of durable exactly-once production persistence. C014 likewise specifies the protocol recovery state transition without introducing a new Rust business journal.

## Canonical vectors

The Rust repository intentionally does not carry a snapshot of the vectors. The integration test `crates/asi-ai-sdk/tests/protocol_v1_1_conformance.rs` reads `AGENT_PROTOCOL_VECTORS`, which is supplied by the `ydtzfx/agi-platform` cross-language gate and points to its canonical `conformance/vectors/c001-c016.json`.

This prevents schema/vector drift between the monorepo and the Rust runtime.
