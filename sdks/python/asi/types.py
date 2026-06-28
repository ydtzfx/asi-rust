"""Type definitions for ASI API."""

from dataclasses import dataclass, field
from typing import Optional

@dataclass
class Message:
    role: str  # "user" | "assistant" | "system"
    content: str
    tool_calls: Optional[list] = None
    tool_call_id: Optional[str] = None

@dataclass
class ToolCall:
    name: str
    arguments: str

@dataclass
class ToolResult:
    name: str
    result: str
    truncated: bool = False

@dataclass
class ChatRequest:
    messages: list[Message]
    agent: Optional[str] = None  # "code" | "review" | "deep"
    session_id: Optional[str] = None
