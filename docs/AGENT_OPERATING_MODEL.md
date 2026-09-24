# AIS development agent operating model (v1)

These are project-scoped **Claude Code development subagents** under
`.claude/agents/`. They do not create additional runtime executors in
`asi-ai-sdk::agent::Coordinator`; that requires a separate Rust implementation,
integration tests and an authorization model.

## Delegation matrix

| Agent | Job | Allowed scope |
| --- | --- | --- |
| planner | requirements, task graph, acceptance criteria | read-only |
| rust-engineer | smallest code changes plus focused tests; parent runs checks | isolated worktree; no Bash |
| ci-diagnostician | diagnose parent-supplied exact-HEAD original CI logs | read-only |
| verifier | independently audit captured local checks and exact-HEAD CI logs supplied by parent | read-only, no Bash |
| reviewer | correctness, API and regression review | read-only |
| security-auditor | privilege, secrets and trust-boundary review | read-only |

The parent assistant is the accountable orchestrator: choose subagents by
specialty, keep task scope and context explicit, reconcile worktree patches,
re-read the current PR HEAD and its checks before any GitHub write or merge.
Never run two writers against the same branch/worktree concurrently.
Delegate only material work; trivial single-file lookups can be direct.

## Workflow and completion criteria

1. Planner decomposes significant work and defines explicit DoD and rollback.
2. Parent checks `git rev-parse HEAD` and `git status` in the prepared isolated worktree and supplies the exact target PR branch, SHA and verification evidence to Rust engineer BEFORE delegation. Worktrees may otherwise start from master. Rust engineer has no Bash tool; without the parent's SHA/isolation proof it must stop and request re-delegation. The parent executes formatting, tests, Clippy and all git operations; the engineer edits only its assigned worktree.
3. Reviewer checks every nontrivial patch; security auditor checks all changes
   affecting auth, agent tools, external input, secrets, self-change or deployment.
4. Parent executes the repository's exact quality commands, preserving logs and exit codes. The verifier has no Bash tool and independently audits captured output plus GitHub job metadata for the exact PR HEAD.
5. Parent fetches original failing CI job logs, job ID, run ID and exact PR HEAD, then passes them to CI diagnostician. A red status alone is insufficient; if no logs are available, diagnosis must stop.
6. Parent updates the specific feature branch, waits for fresh CI on the exact
   HEAD, verifies all required checks and only then performs any authorized merge.

For PR #3, every required gate on the exact current HEAD MUST have a GitHub
success result: Ubuntu build, Ubuntu tests, test-count >=200 (enforced by the `.github/workflows/ci.yml` threshold on this branch), Clippy with
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

Start a fresh Claude Code session on this branch (restart if `.claude/agents/` did not exist when an old session began). Explicitly @-mention each of the six named agents in harmless scoped tasks and check its actual tools and role. In particular delegate a read-only review to reviewer and supply sample original CI logs to ci-diagnostician. If you choose to validate `rust-engineer`, supply an exact target ref/SHA in a disposable isolated worktree. This is a manual runtime test: presence of config files alone does not prove agent loading or permission enforcement.

## Production AIS follow-up

The current Rust coordinator routes actual work to code/review tool-loop
instances; adding these subagents does not change that behavior. To deploy
six-role AIS orchestration in the product, design role adapters, per-tool
authorization, structured task state, cancellation propagation, verified
outcome capture, persistent memory, evaluator benchmarks and rollback tests.
