use leptos::prelude::*;

use super::reasoning_block::ReasoningBlock;
use super::tool_block::ToolBlock;
use super::{ChatMessage, MessagePart};

/// Returns the Tailwind CSS class for the bubble background given a role string.
pub fn role_style(role: &str) -> &'static str {
    match role {
        "user" => "bg-blue-50 border border-blue-100",
        "assistant" => "bg-white border border-gray-200",
        _ => "bg-gray-50 border border-gray-200",
    }
}

/// Render one or more message parts into a view fragment.
fn render_parts(parts: Vec<MessagePart>) -> Vec<AnyView> {
    parts
        .into_iter()
        .map(|part| match part {
            MessagePart::Text(content) => {
                view! { <div class="whitespace-pre-wrap break-words">{content}</div> }.into_any()
            }
            MessagePart::Reasoning(content) => {
                view! { <ReasoningBlock content=content.clone()/> }.into_any()
            }
            MessagePart::ToolCall {
                name,
                arguments,
                result,
                ..
            } => view! {
                <ToolBlock
                    name=name.clone()
                    arguments=arguments.clone()
                    result=result.clone()
                />
            }
            .into_any(),
        })
        .collect()
}

/// A single chat message rendered with awareness of its parts.
///
/// Three display modes:
/// - **text** — rendered in a `<div>` with `whitespace-pre-wrap`
/// - **reasoning** — a collapsible `<details>` element (via `ReasoningBlock`)
/// - **tool_call** — a collapsible JSON block (via `ToolBlock`)
#[component]
pub fn MessageItem(
    /// The message to render.
    message: ChatMessage,
) -> impl IntoView {
    let is_user = message.role == "user";
    let style = role_style(&message.role);

    let alignment = if is_user {
        "justify-end"
    } else {
        "justify-start"
    };
    let bubble_rounding = if is_user {
        "rounded-2xl rounded-tr-md"
    } else {
        "rounded-2xl rounded-tl-md"
    };

    let role_label = if is_user { "You" } else { "ASI" };
    let role_color = if is_user {
        "text-blue-600"
    } else {
        "text-gray-800"
    };

    let container_class = format!("flex {} mb-4", alignment);
    let bubble_class = format!("max-w-[80%] px-4 py-3 {} {}", style, bubble_rounding);
    let label_class = format!(
        "text-xs font-semibold uppercase tracking-wider mb-1 {}",
        role_color
    );

    let parts = render_parts(message.parts);

    view! {
        <div class={container_class}>
            <div class={bubble_class}>
                // Role label
                <div class={label_class}>
                    {role_label}
                </div>

                // Render each part
                {parts}
            </div>
        </div>
    }
}
