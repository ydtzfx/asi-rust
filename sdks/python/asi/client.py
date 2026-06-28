"""ASI API Client with SSE streaming support."""

import json
import requests
from typing import Generator, Optional
from .types import Message, ChatRequest

class ASIClient:
    """Client for the ASI AI Coding Assistant API."""

    def __init__(self, base_url: str = "http://localhost:3000", api_key: Optional[str] = None):
        self.base_url = base_url.rstrip("/")
        self.session = requests.Session()
        if api_key:
            self.session.headers["Authorization"] = f"Bearer {api_key}"

    def health(self) -> dict:
        """Check server health."""
        return self.session.get(f"{self.base_url}/api/health").json()

    def ready(self) -> dict:
        """Check server readiness (DB + AI provider)."""
        return self.session.get(f"{self.base_url}/api/ready").json()

    def version(self) -> dict:
        """Get server version info."""
        return self.session.get(f"{self.base_url}/api/version").json()

    def list_models(self) -> list:
        """List available AI models."""
        return self.session.get(f"{self.base_url}/api/model").json()

    def chat(self, messages: list[Message], agent: str = "code",
             session_id: Optional[str] = None) -> Generator[dict, None, None]:
        """Send a chat message and stream SSE events.

        Yields dicts with keys: event (text/tool_call/tool_result/done/error) and data.
        """
        body = {
            "messages": [{"role": m.role, "content": m.content} for m in messages],
            "agent": agent,
        }
        if session_id:
            body["session_id"] = session_id

        resp = self.session.post(
            f"{self.base_url}/api/chat",
            json=body,
            stream=True,
            headers={"Accept": "text/event-stream"},
        )
        resp.raise_for_status()

        buffer = ""
        for chunk in resp.iter_content(chunk_size=None, decode_unicode=True):
            if not chunk:
                continue
            buffer += chunk
            while "\n\n" in buffer:
                block, buffer = buffer.split("\n\n", 1)
                event_type = ""
                data = ""
                for line in block.split("\n"):
                    if line.startswith("event: "):
                        event_type = line[7:]
                    elif line.startswith("data: "):
                        data += line[6:]
                if event_type or data:
                    yield {"event": event_type or "text", "data": data}

    def chat_sync(self, messages: list[Message], agent: str = "code",
                  session_id: Optional[str] = None) -> str:
        """Send a chat message and return the complete response text."""
        text = ""
        for evt in self.chat(messages, agent, session_id):
            if evt["event"] in ("text", ""):
                text += evt["data"]
            elif evt["event"] == "done":
                break
            elif evt["event"] == "error":
                raise RuntimeError(evt["data"])
        return text

    def metrics(self) -> str:
        """Get Prometheus metrics."""
        return self.session.get(f"{self.base_url}/api/metrics").text

    def openapi(self) -> dict:
        """Get OpenAPI specification."""
        return self.session.get(f"{self.base_url}/api/openapi.json").json()
