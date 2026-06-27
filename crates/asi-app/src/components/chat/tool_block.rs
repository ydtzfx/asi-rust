use leptos::prelude::*;

/// Collapsible tool-call block.
///
/// Renders as a `<details>` element whose summary shows the uppercase tool
/// name.  When expanded, the call arguments are displayed as pretty-printed
/// JSON.  If a `result` is available it is shown as a second JSON block.
#[component]
pub fn ToolBlock(
    /// Tool name (displayed as uppercase in the summary).
    name: String,
    /// JSON arguments for the tool call.
    arguments: serde_json::Value,
    /// Optional tool result (JSON or text).
    result: Option<String>,
) -> impl IntoView {
    let pretty_args = serde_json::to_string_pretty(&arguments).unwrap_or_default();

    let result_section = move || {
        result.clone().map(|res| {
        // Attempt to pretty-print the result if it's valid JSON.
        let pretty_res = serde_json::from_str::<serde_json::Value>(&res)
            .ok()
            .and_then(|v| serde_json::to_string_pretty(&v).ok())
            .unwrap_or(res);

        view! {
            <div class="mt-2">
                <div class="text-xs font-medium text-gray-500 uppercase tracking-wider mb-1">
                    "Result"
                </div>
                <pre class="text-xs text-gray-700 whitespace-pre-wrap font-mono">{pretty_res}</pre>
            </div>
        }
    })
    };

    view! {
        <details class="group rounded-lg border border-gray-200 bg-gray-50 overflow-hidden">
            <summary class="flex items-center gap-2 px-4 py-2 cursor-pointer text-sm text-gray-500 hover:text-gray-700 select-none">
                <span class="inline-block transition-transform group-open:rotate-90">
                    "▶"
                </span>
                <span class="font-mono font-semibold text-orange-600">
                    {name.to_uppercase()}
                </span>
                <span class="text-gray-400">"tool call"</span>
            </summary>
            <div class="px-4 pb-3 pt-1 border-t border-gray-200">
                <div class="mb-2">
                    <div class="text-xs font-medium text-gray-500 uppercase tracking-wider mb-1">
                        "Arguments"
                    </div>
                    <pre class="text-xs text-gray-700 whitespace-pre-wrap font-mono">{pretty_args}</pre>
                </div>
                {result_section}
            </div>
        </details>
    }
}
