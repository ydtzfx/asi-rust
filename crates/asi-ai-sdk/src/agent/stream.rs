use crate::agent::tool_loop::AgentEvent;
use axum::response::sse::{Event as SseEvent, Sse};
use futures_core::Stream;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::UnboundedReceiverStream;

/// Convert an AgentEvent receiver into an Axum SSE response stream.
pub fn agent_events_to_sse(
    rx: tokio::sync::mpsc::UnboundedReceiver<AgentEvent>,
) -> Sse<impl Stream<Item = Result<SseEvent, std::convert::Infallible>>> {
    let stream = UnboundedReceiverStream::new(rx).map(|event| {
        let sse_event = match event {
            AgentEvent::TextDelta { content } => SseEvent::default().data(content).event("text"),
            AgentEvent::ToolCall { name, arguments } => SseEvent::default()
                .data(serde_json::json!({ "name": name, "arguments": arguments }).to_string())
                .event("tool_call"),
            AgentEvent::ToolResult {
                name,
                result,
                truncated,
            } => SseEvent::default()
                .data(
                    serde_json::json!({ "name": name, "result": result, "truncated": truncated })
                        .to_string(),
                )
                .event("tool_result"),
            AgentEvent::Done { usage } => {
                let data = usage
                    .map(|u| serde_json::to_string(&u).unwrap())
                    .unwrap_or_default();
                SseEvent::default().data(data).event("done")
            }
            AgentEvent::Error { message } => SseEvent::default().data(message).event("error"),
        };
        Ok(sse_event)
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("keep-alive"),
    )
}
