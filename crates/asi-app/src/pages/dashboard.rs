use leptos::prelude::*;
use leptos_meta::*;

use crate::components::chat::{
    ChatMessage, InputBar, MessageList,
};
use crate::contexts::use_user;

/// Dashboard chat page.
///
/// Full-height layout with a scrollable message list at the top and a
/// chat input bar pinned to the bottom.  Messages are sent to
/// `POST /api/chat` and the SSE stream is parsed into text / reasoning /
/// tool-call parts.
#[component]
pub fn DashboardPage() -> impl IntoView {
    let messages: RwSignal<Vec<ChatMessage>> = RwSignal::new(Vec::new());
    let loading: RwSignal<bool> = RwSignal::new(false);
    let error: RwSignal<Option<String>> = RwSignal::new(None);

    // Consume the user context (provided by UserProvider in the app shell).
    let _user_ctx = use_user();

    // ------------------------------------------------------------------
    // Submit handler
    // ------------------------------------------------------------------
    let on_submit = Box::new(move |text: String| {
        if text.trim().is_empty() || loading.get() {
            return;
        }

        error.set(None);
        loading.set(true);

        // 1. Add the user message to the list
        let msg_idx = messages.get().len();
        let user_msg = ChatMessage::new_user(
            format!("user-{}", msg_idx),
            text.clone(),
        );
        messages.update(|msgs| msgs.push(user_msg));

        // 2. Create a placeholder assistant message that we'll stream into
        let assistant_id = format!("assistant-{}", msg_idx);
        let assistant_msg = ChatMessage::new_assistant(assistant_id);
        messages.update(|msgs| msgs.push(assistant_msg));

        // 3. Build the request body from existing messages
        let body = build_chat_request_body(&messages.get());

        // 4. Fire off the SSE stream (client-side only)
        #[cfg(target_arch = "wasm32")]
        wasm_bindgen_futures::spawn_local(async move {
            stream_chat_response(body, messages, loading, error).await;
        });

        // Server-side: no streaming – just clear loading.
        #[cfg(not(target_arch = "wasm32"))]
        let _ = body;

        // Server-side: just unset loading immediately.
        #[cfg(not(target_arch = "wasm32"))]
        loading.set(false);
    });

    // ------------------------------------------------------------------
    // Render
    // ------------------------------------------------------------------
    view! {
        <Title text="Dashboard — ASI"/>
        <Meta name="description" content="ASI coding assistant dashboard."/>

        <div class="flex flex-col h-[calc(100vh-4rem)] max-w-5xl mx-auto">
            // Header
            <div class="flex-shrink-0 border-b border-gray-200 px-4 py-3 bg-white">
                <div class="flex items-center justify-between">
                    <div>
                        <h1 class="text-lg font-semibold">"Coding Assistant"</h1>
                        <p class="text-xs text-gray-400">"Multi-agent AI code generation"</p>
                    </div>
                </div>
            </div>

            // Error banner
            <Show when=move || error.get().is_some()>
                <div class="flex-shrink-0 mx-4 mt-2 px-4 py-2 bg-red-50 border border-red-200
                            rounded-lg text-sm text-red-700">
                    {move || error.get().unwrap_or_default()}
                </div>
            </Show>

            // Message list (takes remaining height, scrollable)
            <MessageList messages=messages/>

            // Input bar (pinned at the bottom)
            <InputBar on_submit loading=loading/>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Chat API interaction (WASM-only)
// ---------------------------------------------------------------------------

/// Build the JSON body for a POST to `/api/chat`.
///
/// Translates our internal `ChatMessage` list into the wire format that
/// the chat endpoint expects (array of `{ role, content }` objects).
fn build_chat_request_body(messages: &[ChatMessage]) -> serde_json::Value {
    let wire_messages: Vec<serde_json::Value> = messages
        .iter()
        .filter(|m| !m.content.is_empty() || m.role == "user")
        .map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": m.content,
            })
        })
        .collect();

    serde_json::json!({
        "messages": wire_messages,
    })
}

/// Call `POST /api/chat` and parse the SSE stream into the message list.
///
/// SSE events:
/// - `event: text` → text delta, appended to the current assistant message
/// - `event: tool_call` → `{ name, arguments }` → tool-call part
/// - `event: tool_result` → `{ name, result, truncated }` → set result on last tool
/// - `event: error` → surface error to the user
/// - `event: done` → stream complete, set loading to false
#[cfg(target_arch = "wasm32")]
async fn stream_chat_response(
    body: serde_json::Value,
    messages: RwSignal<Vec<ChatMessage>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    use wasm_bindgen::JsCast;

    let js_body = serde_wasm_bindgen::to_value(&body).unwrap_throw();

    let opts = {
        let mut opts = web_sys::RequestInit::new();
        opts.set_method("POST");
        opts.set_body(&js_body);
        let headers = web_sys::Headers::new().unwrap_throw();
        headers.append("Content-Type", "application/json").unwrap_throw();
        opts.set_headers(&headers);
        opts
    };

    let request = web_sys::Request::new_with_str_and_init("/api/chat", &opts).unwrap_throw();

    match web_sys::window()
        .unwrap_throw()
        .fetch_with_request(&request)
        .await
    {
        Ok(resp) => {
            let resp: web_sys::Response = resp.dyn_into().unwrap_throw();
            match resp.text() {
                Ok(text_promise) => {
                    let text = wasm_bindgen_futures::JsFuture::from(text_promise)
                        .await
                        .ok()
                        .and_then(|v| v.as_string())
                        .unwrap_or_default();

                    parse_sse_into_messages(&text, &messages);
                    loading.set(false);
                }
                Err(_) => {
                    error.set(Some("Failed to read response body".into()));
                    loading.set(false);
                }
            }
        }
        Err(e) => {
            let err_msg = format!("Request failed: {:?}", e);
            error.set(Some(err_msg));
            loading.set(false);
        }
    }
}

/// Parse SSE-formatted text and stream the events into the messages list.
///
/// The server sends newline-delimited SSE:
/// ```text
/// event: text
/// data: Hello
///
/// event: tool_call
/// data: {"name":"read_file","arguments":{...}}
///
/// event: done
/// data: ...
/// ```
#[cfg(target_arch = "wasm32")]
fn parse_sse_into_messages(sse_text: &str, messages: &RwSignal<Vec<ChatMessage>>) {
    let mut current_event: Option<String> = None;
    let mut current_data = String::new();

    for line in sse_text.lines() {
        if line.starts_with("event: ") {
            current_event = Some(line["event: ".len()..].to_string());
        } else if line.starts_with("data: ") {
            current_data = line["data: ".len()..].to_string();
        } else if line.is_empty() {
            // Dispatch the completed event.
            if let Some(ref event) = current_event {
                handle_sse_event(event, &current_data, messages);
            }
            current_event = None;
            current_data.clear();
        }
    }

    // Handle any trailing event without a trailing blank line.
    if let Some(ref event) = current_event {
        handle_sse_event(event, &current_data, messages);
    }
}

/// Dispatch a single SSE event to the messages list.
#[cfg(target_arch = "wasm32")]
fn handle_sse_event(
    event: &str,
    data: &str,
    messages: &RwSignal<Vec<ChatMessage>>,
) {
    messages.update(|msgs| {
        // Find the last assistant message (the one being streamed to).
        let last_assistant = msgs.iter_mut().rev().find(|m| m.role == "assistant");
        let Some(msg) = last_assistant else { return };

        match event {
            "text" => {
                msg.push_text(data);
            }
            "reasoning" => {
                msg.push_reasoning(data);
            }
            "tool_call" => {
                if let Ok(payload) =
                    serde_json::from_str::<serde_json::Value>(data)
                {
                    let name = payload["name"].as_str().unwrap_or("unknown").to_string();
                    let args = payload.get("arguments").cloned().unwrap_or(serde_json::Value::Null);
                    let id = payload["id"].as_str().unwrap_or("tc-0").to_string();
                    msg.push_tool_call(id, name, args);
                }
            }
            "tool_result" => {
                if let Ok(payload) =
                    serde_json::from_str::<serde_json::Value>(data)
                {
                    let name = payload["name"].as_str().unwrap_or("unknown");
                    let result = payload["result"].as_str().unwrap_or("");
                    msg.push_tool_result(name, result.to_string());
                }
            }
            "error" => {
                // Errors during streaming — append as text.
                msg.push_text(&format!("\n[Error: {}]", data));
            }
            _ => {}
        }
    });
}
