use leptos::prelude::*;

/// Collapsible reasoning / chain-of-thought block.
///
/// Renders as a `<details>` element with a "Thinking…" summary.  The actual
/// reasoning content is shown when the user expands it.
#[component]
pub fn ReasoningBlock(content: String) -> impl IntoView {
    view! {
        <details class="group rounded-lg border border-gray-200 bg-gray-50 overflow-hidden">
            <summary class="flex items-center gap-2 px-4 py-2 cursor-pointer text-sm text-gray-500 hover:text-gray-700 select-none">
                <span class="inline-block transition-transform group-open:rotate-90">
                    "▶"
                </span>
                <span class="font-medium">"Thinking…"</span>
            </summary>
            <div class="px-4 pb-3 pt-1 text-sm text-gray-600 whitespace-pre-wrap border-t border-gray-200">
                {content}
            </div>
        </details>
    }
}
