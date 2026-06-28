---
name: run
description: Start the ASI server with Ollama after building
disable-model-invocation: true
---

# Run ASI Server

Start (or restart) the development server with Ollama.

## Steps

1. Stop any existing server process listening on port 3000
2. Build with `cargo build --release`
3. Load env vars from `.env` and start the server:
   ```bash
   export $(grep -v '^#' .env | xargs) && ./target/release/asi-server.exe
   ```
4. Verify with `curl -s http://localhost:3000/api/health`

The server listens on `http://localhost:3000`. The homepage serves the chat UI.
API routes are under `/api/`. Dev-mode auth requires `X-User-ID` header.
