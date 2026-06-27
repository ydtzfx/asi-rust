use leptos::prelude::*;

use super::ChatMessage;
use super::message_item::MessageItem;

/// Renders a scrollable list of chat messages with auto-scroll.
///
/// When new messages are added the viewport automatically scrolls to the
/// bottom so the user always sees the latest content.
#[component]
pub fn MessageList(
    /// Reactive list of messages to display.
    messages: RwSignal<Vec<ChatMessage>>,
) -> impl IntoView {
    // Scroll anchor — a div at the bottom of the list that we scroll into view.
    let scroll_anchor: NodeRef<leptos::html::Div> = NodeRef::new();

    // Auto-scroll when messages change.
    Effect::watch(
        move || messages.get().len(),
        move |_len, _prev_len, _| {
            if let Some(el) = scroll_anchor.get() {
                el.scroll_into_view();
            }
        },
        false,
    );

    // If there's an "assistant" message being streamed, it is the last
    // message with no content yet — show a cursor.
    let show_cursor = move || {
        messages
            .get()
            .last()
            .map(|m| m.role == "assistant" && m.content.is_empty())
            .unwrap_or(false)
    };

    view! {
        <div class="flex-1 overflow-y-auto px-4 py-6 space-y-1">
            <Show when=move || messages.get().is_empty()>
                <div class="flex items-center justify-center h-full text-gray-400">
                    <div class="text-center">
                        <p class="text-lg font-medium">"Start a conversation"</p>
                        <p class="text-sm mt-1">
                            "Send a message to begin working with the ASI coding agent."
                        </p>
                    </div>
                </div>
            </Show>

            <For
                each=move || messages.get()
                key=|msg| msg.id.clone()
                children=move |msg| {
                    view! { <MessageItem message=msg/> }
                }
            />

            <Show when=show_cursor>
                <div class="flex justify-start mb-4">
                    <div class="bg-white border border-gray-200 rounded-2xl rounded-tl-md px-4 py-3">
                        <div class="flex gap-1">
                            <span class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"
                                style="animation-delay: 0ms"
                            ></span>
                            <span class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"
                                style="animation-delay: 150ms"
                            ></span>
                            <span class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"
                                style="animation-delay: 300ms"
                            ></span>
                        </div>
                    </div>
                </div>
            </Show>

            // Scroll anchor div
            <div node_ref=scroll_anchor></div>
        </div>
    }
}
