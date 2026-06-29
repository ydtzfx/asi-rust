//! Knowledge seeding — automatically extracts project context for the agent.
//! Runs at startup to populate the knowledge base with repository structure.

use asi_evolution::knowledge::KnowledgeBase;

/// Seed the knowledge base with project structure and key facts.
pub fn seed_knowledge(kb: &KnowledgeBase) {
    // Project identity
    kb.upsert("project_name", "ASI-Rust — Enterprise AI Coding Assistant Platform");
    kb.upsert("project_type", "Rust workspace with 22 crates, Axum web server, Ollama/DeepSeek AI");
    kb.upsert("rust_edition", "Rust Edition 2024, minimum rustc 1.96");

    // Architecture
    kb.upsert("architecture", "as i-server (HTTP+SSE) → asi-ai-sdk (agents) → asi-db (SQLite). Frontend: static/index.html");
    kb.upsert("crates_list", "asi-server, asi-ai-sdk, asi-evolution, asi-automation, asi-devops, asi-defense, asi-cortex, asi-mesh, asi-twin, asi-business, asi-events, asi-compliance, asi-analytics, asi-cli, asi-grpc, asi-gateway, asi-loop, asi-auth, asi-db, asi-lib, asi-app");

    // Key files
    kb.upsert("entrypoint", "crates/asi-server/src/main.rs — single binary entrypoint, binds port 3000");
    kb.upsert("chat_handler", "crates/asi-server/src/routes/chat.rs — main chat API handler with SSE streaming");
    kb.upsert("agent_loop", "crates/asi-ai-sdk/src/agent/tool_loop.rs — ToolLoopAgent core loop with tool calling");

    // Tools available
    kb.upsert("tools", "readFile, writeFile, listDirectory, runCommand (git/cargo/npm/npx). Shell metacharacters blocked. node excluded.");
    kb.upsert("path_safety", "All file ops use resolve_safe_path() with TOCTOU protection via verify_path_after_write()");

    // AI configuration
    kb.upsert("ai_providers", "Primary: Ollama (gemma4:31b-cloud). Fallback: qwen3:4b. DeepSeek available via DEEPSEEK_API_KEY");
    kb.upsert("streaming", "First agent iteration uses chat_stream() for real-time token output. Subsequent use chat()");

    // Security
    kb.upsert("security_auth", "Clerk JWT with RS256. Dev-mode requires sk_test_ key + ASI_DEV_AUTH_BYPASS=true");
    kb.upsert("security_rate_limit", "GlobalRateLimitLayer: 60 req/min per endpoint. Chat: 20/min per user. Concurrency: max 4 agents");

    // Testing
    kb.upsert("testing", "~170 tests across 59 suites. cargo test --workspace. Coverage gate: minimum 100 tests in CI");

    tracing::info!("Knowledge base seeded — {} entries", kb.size());
}
