# ASI-Rust 系统风险报告

**日期**: 2026-06-28 | **分支**: master | **提交**: a9e17a4

---

## 严重程度分布

| 严重性 | 数量 |
|--------|------|
| 🔴 Critical | 4 |
| 🟠 High | 14 |
| 🟡 Medium | 16 |
| 🟢 Low | 5 |
| **总计** | **39** |

---

## 🔴 Critical — 立即修复

### RISK-001 13/15 API 路由无认证

**文件**: `crates/asi-server/src/router.rs:9-35`
**CWE**: CWE-306 (Missing Authentication)

仅 `/api/user/me` 有 auth 中间件。`/api/chat`、`/api/search`、`/api/sessions`、`/api/evolve`、`/api/eval` 等全部对匿名调用者开放。`chat.rs:129` 硬编码了 `let user_id = "anonymous"`。

```
攻击者 → POST /api/chat → 消耗 LLM token 配额 → 无需任何凭证
攻击者 → GET /api/search?q=password → 读取所有用户会话数据
```

**修复**: 在 `api_routes` Router 层级应用 `require_auth` 中间件，仅对 `/health` 和 `/version` 显式豁免。

---

### RISK-002 特性开关端点无认证 — 可绕过注入防御

**文件**: `crates/asi-server/src/routes/flags.rs:13-35`
**CWE**: CWE-306

`POST /api/flags?set=prompt-injection-defense` 无认证即可切换任意 feature flag。攻击者可禁用 prompt-injection-defense，然后向 `/api/chat` 发送越狱 prompt。

**修复**: 要求 EVOLVE_SECRET 或管理员级 JWT claim 才可修改 flags。

---

### RISK-003 Agent 任务在客户端断开后继续执行（资源泄漏）

**文件**: `crates/asi-server/src/routes/chat.rs:285` + `crates/asi-ai-sdk/src/agent/tool_loop.rs:95`
**CWE**: CWE-404 (Improper Resource Shutdown)

SSE 流启动后立即释放 concurrency slot（line 285），但 agent 任务通过 `tokio::spawn` 异步运行且无取消机制。客户端断开时 agent 继续执行至 `max_steps=20`，消耗 LLM token 和计算资源。所有 `tx.send()` 的错误都被 `let _ =` 丢弃，agent 无法感知消费者已消失。

```
攻击者: 打开4个SSE连接 → 立即断开 → 4个 agent 继续运行 ~200秒
→ concurrency slot 已释放 → 可以再创建4个 → 8个 agent 同时运行
→ 服务器资源耗尽
```

**修复**: 向 agent loop 传递 `tokio_util::sync::CancellationToken`，在每次迭代检查 `token.is_cancelled()`。SSE stream drop 时触发取消。

---

### RISK-004 ConcurrencyLimiter::release() panic 可导致服务器崩溃

**文件**: `crates/asi-lib/src/concurrency.rs:42-44`
**CWE**: CWE-617 (Reachable Assertion)

```rust
pub fn release(&self) {
    let prev = self.active.fetch_sub(1, Ordering::Release);
    assert!(prev > 0, "...");  // ← panic on double-release
}
```

任何代码路径错误导致 `release()` 被调用两次，整个服务器进程 panic 崩溃。

**修复**: 使用 `saturating_sub` 或 RAII guard（`Drop` 实现），消除 panic 可能性。

---

## 🟠 High — 尽快修复

### RISK-005 `node` 在命令 allowlist 中允许任意代码执行

**文件**: `crates/asi-server/src/agent/tools/run_command.rs:113-127`
**CWE**: CWE-78 (OS Command Injection)

`SAFE_COMMANDS` 中 `node` 条目: `subcommands: None`（允许任意子命令）+ `reject_flags: false`（允许 flag）。LLM 可被诱导执行 `node -e "require('child_process').execSync('...')"` 实现任意 OS 命令执行。

**修复**: 移除 `node` 和 `npx` 或不限制子命令的条目，或设 `reject_flags: true` + 限制特定脚本路径。

---

### RISK-006 Permissive CORS + Cookie Auth = CSRF

**文件**: `crates/asi-server/src/router.rs:26-29`
**CWE**: CWE-942

```rust
CorsLayer::new()
    .allow_origin(Any)       // ← 任意来源
    .allow_methods(Any)
    .allow_headers(Any)
```

结合 cookie-based Clerk JWT（`__session` cookie），任意网站可代表已登录用户发起认证请求。

**修复**: 限制 `allow_origin` 为前端具体域名。

---

### RISK-007 搜索端点泄漏所有用户会话数据

**文件**: `crates/asi-server/src/routes/search.rs:54-82`
**CWE**: CWE-200 (Information Exposure)

`GET /api/search?q=<query>` 无认证 + 无 user_id 过滤 → 搜索所有用户的会话，返回包含 `context_json`（完整对话历史）的结果。

**修复**: 要求认证，强制 `user_id` 匹配认证用户，不在列表接口返回 `context_json`。

---

### RISK-008 会话端点接受任意 user_id（IDOR）

**文件**: `crates/asi-server/src/routes/sessions.rs:15-29,98-105`
**CWE**: CWE-639 (Insecure Direct Object Reference)

所有会话端点的 `user_id` 由客户端提供，默认 `"anonymous"`。`export_session` 完全不做所有权检查。

```
攻击者: GET /api/sessions?user_id=victim_123 → 读取他人所有会话
攻击者: GET /api/sessions/any_id/export → 导出他人完整对话历史
```

**修复**: 从 JWT 派生 user_id，移除客户端提供的 user_id 参数。

---

### RISK-009 DeepSeekProvider HTTP client 无超时

**文件**: `crates/asi-ai-sdk/src/provider/deepseek.rs:18-19`
**CWE**: CWE-1088

`Client::new()` 无超时 → DeepSeek API 变慢或挂起时，agent 永久卡住，占用 concurrency slot 直至进程重启。

> OllamaProvider 本次已修复（加了 300s 超时），DeepSeek 仍需修复。

**修复**: `Client::builder().timeout(Duration::from_secs(120)).build()`

---

### RISK-010 会话清理定时器是空桩 — 数据库无限增长

**文件**: `crates/asi-server/src/startup.rs:24-32`
**CWE**: CWE-404

`clean_stale_sessions` 函数已实现且有测试覆盖，但 startup hook 只打日志，从未真正调用。会话表和 `context_json` 永久积累。

**修复**: 将清理定时器连接到 `asi_db::session_cleanup::clean_stale_sessions(pool, 7_days)`。

---

### RISK-011 Provider 故障时无运行时回退

**文件**: `crates/asi-server/src/routes/chat.rs:244-260`
**CWE**: CWE-754

`model-fallback` feature flag 已定义但从未在代码中检查。provider 选定后不可变，DeepSeek 故障 → 所有请求 500。Ollama fallback model 已配置但未被使用。

**修复**: 在 `provider.chat()` 返回可重试错误且 `flag("model-fallback")` 为 true 时，自动切换到 Ollama provider。

---

### RISK-012 Agent 对话缓冲区无界增长

**文件**: `crates/asi-ai-sdk/src/agent/tool_loop.rs:149-151`
**CWE**: CWE-770

每次迭代 clone 整个 conversation，tool 结果持续追加。20 steps × 8KB tool results = 160KB+ 的请求体发送给 LLM，且 `max_tokens: None`。

**修复**: 设置 `max_tokens` 上限，对过长的 tool-result 消息做上下文窗口裁剪。

---

### RISK-013 工具结果截断可能 panic（UTF-8 字节边界）

**文件**: `crates/asi-ai-sdk/src/agent/tool_loop.rs:257`
**CWE**: CWE-130

```rust
let display = output[..MAX_RESULT_LEN].to_string();  // ← 可能在多字节字符中间截断
```

字符串切片在非 UTF-8 字符边界会导致 panic。

**修复**: 使用 `output.char_indices()` 找到合法截断点。

---

### RISK-014 DB 连接池太小（默认 5）

**文件**: `crates/asi-db/src/lib.rs:22`
**CWE**: CWE-770

4 个并发 agent + health check + stats + metrics = 5 连接耗尽。`try_acquire()` 返回 None → `/api/ready` 报不健康。

**修复**: 默认增至 10+，支持 `DATABASE_POOL_SIZE` 环境变量配置。

---

### RISK-015 硬编码密钥已提交到仓库

**文件**: `.env:1-2`（已提交）
**CWE**: CWE-798

`CLERK_SECRET_KEY=sk_test_xxx` 和 `NEXT_PUBLIC_CLERK_PUBLISHABLE_KEY=pk_test_xxx` 已提交。虽然当前是占位值，但 `.env` 不在 `.gitignore` 中，未来可能提交真实密钥。

**修复**: 将 `.env` 加入 `.gitignore`，轮换密钥。

---

### RISK-016 Evolve 端点使用同步阻塞 Command + 无超时

**文件**: `crates/asi-server/src/routes/evolve.rs:96-112`
**CWE**: CWE-1088 / CWE-400

`std::process::Command::output()` 同步阻塞 tokio worker 线程。`cargo clippy` + `cargo test` 可能运行数分钟到数小时。

**修复**: 使用 `tokio::process::Command` + `tokio::time::timeout`，或 `spawn_blocking`。

---

### RISK-017 无代理任务取消机制（客户端断开 → 资源浪费）

**文件**: `crates/asi-ai-sdk/src/agent/tool_loop.rs:165-229`
**CWE**: CWE-252 (Unchecked Return Value)

所有 `tx.send()` 返回 `let _ =`，agent 无法感知接收端已断开。浪费 LLM API 调用和计算资源。

**修复**: 检查 `tx.send()` 返回值，错误时 break loop。

---

### RISK-018 无消息长度限制

**文件**: `crates/asi-server/src/routes/chat.rs:132-136`
**CWE**: CWE-770

用户消息直接传给 LLM provider，无长度/数量限制。攻击者可提交数 MB 的 payload 消耗 token 配额。

**修复**: 限制总内容长度（如 100KB）和消息数量（如 50 条）。

---

## 🟡 Medium — 计划修复

| ID | 描述 | 文件 |
|----|------|------|
| RISK-019 | JWT 验证使用用户控制的 `kid` 无证书链验证 | `asi-auth/src/clerk.rs:64-73` |
| RISK-020 | JWKS fetch 无超时、无证书固定 | `asi-auth/src/clerk.rs:41-47` |
| RISK-021 | EVOLVE_SECRET 可暴力破解（无速率限制） | `routes/evolve.rs:39-53` |
| RISK-022 | prompt-injection-defense 仅用 7 个正则，易绕过 | `asi-lib/src/prompt_guard.rs` |
| RISK-023 | Agent 路由基于用户内容（`/review` 前缀） | `routes/chat.rs:267` |
| RISK-024 | `resolve_safe_path` TOCTOU 竞争条件 | `asi-lib/src/safe_path.rs:32-48` |
| RISK-025 | 速率限制使用静态 key `"unknown"` — 实际无效 | `routes/chat.rs:89` |
| RISK-026 | 仅 `/api/chat` 有限速，其他端点无保护 | 多个 route 文件 |
| RISK-027 | SQL LIKE 模式中的通配符未转义 | `routes/search.rs:51` |
| RISK-028 | DeepSeek base URL 硬编码不可配置 | `asi-ai-sdk/src/provider/deepseek.rs:22` |
| RISK-029 | warmup 是空桩，每次请求仍调用 | `asi-lib/src/warmup.rs:5-11` |
| RISK-030 | stats 端点数据库错误静默返回 0 | `routes/stats.rs:4-40` |
| RISK-031 | audit log 失败静默丢弃（`let _ =`） | `routes/chat.rs:229-238` |
| RISK-032 | ReadFile 无文件大小限制（可 OOM） | `agent/tools/read_file.rs:45` |
| RISK-033 | config 启动时未验证关键环境变量 | `asi-lib/src/config.rs:14-26` |
| RISK-034 | auto-evolve 定时器是空桩 | `startup.rs:13-20` |

---

## 🟢 Low — 后续优化

| ID | 描述 | 文件 |
|----|------|------|
| RISK-035 | Tier limits 已定义但从未执行 | `asi-lib/src/tier.rs` |
| RISK-036 | nanoid 使用 modulo bias | `asi-lib/src/utils/ids.rs:10` |
| RISK-037 | 测试 provider 使用 `unimplemented!()` | `agent/code_agent.rs:62,72` |
| RISK-038 | 速率限制 map 无过期清理 | `asi-lib/src/rate_limit.rs:43` |
| RISK-039 | telemetry counters 使用原始 URI（含动态 ID） | `asi-lib/src/telemetry.rs:14` |

---

## 修复优先级路线图

### Phase 1: 安全基线（本周）

1. **全局认证中间件** — RISK-001, RISK-002, RISK-007, RISK-008, RISK-005
2. **CORS 收紧** — RISK-006
3. **command allowlist 收紧** — RISK-005
4. **`.env` → `.gitignore`** — RISK-015

### Phase 2: 可靠性（下周）

5. **Agent 取消令牌** — RISK-003, RISK-017
6. **DeepSeek 超时** — RISK-009
7. **ConcurrencyLimiter 修复** — RISK-004
8. **Provider fallback** — RISK-011

### Phase 3: 运维加固（本月）

9. **会话清理** — RISK-010
10. **DB 连接池调优** — RISK-014
11. **速率限制完善** — RISK-025, RISK-026
12. **Evolve 异步化** — RISK-016

### Phase 4: 深度防御

13. **提示注入升级**
14. **JWKS 验证强化**
15. **输入限制**
16. **配置验证**
