# AIS development agent operating model (v1)

These are project-scoped **Claude Code development subagents** under
`.claude/agents/`. They do not create additional runtime executors in
`asi-ai-sdk::agent::Coordinator`; that requires a separate Rust implementation,
integration tests and an authorization model.

## Delegation matrix

| Agent | Job | Allowed scope |
| --- | --- | --- |
| planner | requirements, task graph, acceptance criteria | read-only |
| rust-engineer | smallest code changes plus focused tests | isolated worktree |
| ci-diagnostician | diagnose exact failures from actual CI logs | read-only |
| verifier | independent build, test and CI evidence matrix | non-mutating verification |
| reviewer | correctness, API and regression review | read-only |
| security-auditor | privilege, secrets and trust-boundary review | read-only |

The parent assistant is the accountable orchestrator: choose subagents by
specialty, keep task scope and context explicit, reconcile worktree patches,
re-read the current PR HEAD and its checks before any GitHub write or merge.
Never run two writers against the same branch/worktree concurrently.
Delegate only material work; trivial single-file lookups can be direct.

## Workflow and completion criteria

1. Planner decomposes significant work and defines explicit DoD and rollback.
2. Rust engineer implements one scoped task inside an isolated worktree.
3. Reviewer checks every nontrivial patch; security auditor checks all changes
   affecting auth, agent tools, external input, secrets, self-change or deployment.
4. Verifier executes the repository's actual quality commands and records output.
5. CI diagnostician inspects failing **job logs**, not red status alone.
6. Parent updates the specific feature branch, waits for fresh CI on the exact
   HEAD, verifies all required checks and only then performs any authorized merge.

For PR #3, every required gate on the exact current HEAD MUST have a GitHub
success result: Ubuntu build, Ubuntu tests, test-count >=200, Clippy with
`-D warnings`, format check, Windows build and Windows tests. Pending,
skipped, cancelled, failed and unverified checks block merge. Do not change
`master` directly or introduce lint suppressions as a substitute for a fix.
Use an expected-head-SHA precondition on merge to prevent stale-head merges.

Auto-approval of routine project work does not grant permission to bypass
platform controls, mandatory CI, secrets approvals, branch protection or
high-impact deployment/financial actions.

## Agent task contract

Input: task ID, goal, affected files/crates, non-goals, constraints, allowed
tools, acceptance tests and rollback strategy.
Output: actual changes or diagnosis, supporting evidence, tests run,
remaining risks, next handoff and reusable runbook improvements.
Report failure, partial progress and unknown status explicitly; never claim
simulated actions, assumed production capability or unrun validation as done.

## Installation check

Open a Claude Code session on this branch and inspect `/agents`, or ask
the parent to delegate to each named agent. Confirm the six role definitions
load, then run a harmless read-only reviewer task. This is a manual runtime
check, not covered by repository file validation.

## Production AIS follow-up

The current Rust coordinator routes actual work to code/review tool-loop
instances; adding these subagents does not change that behavior. To deploy
six-role AIS orchestration in the product, design role adapters, per-tool
authorization, structured task state, cancellation propagation, verified
outcome capture, persistent memory, evaluator benchmarks and rollback tests.
