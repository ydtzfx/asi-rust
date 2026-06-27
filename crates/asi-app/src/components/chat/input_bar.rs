use std::rc::Rc;

use leptos::prelude::*;

/// A controlled chat input bar.
///
/// Features:
/// - Controlled `textarea` bound to a `RwSignal<String>`
/// - Enter key submits the message (Shift+Enter for a newline)
/// - Submit button with SVG icon
/// - Loading state disables both the textarea and the button
/// - Clears the input after a successful submit
#[component]
pub fn InputBar(
    /// Callback invoked with the trimmed message text when the user submits.
    on_submit: Box<dyn Fn(String)>,
    /// When `true`, the input and button are disabled and a spinner is shown.
    #[prop(default = RwSignal::new(false))]
    loading: RwSignal<bool>,
) -> impl IntoView {
    let input = RwSignal::new(String::new());
    let textarea_ref: NodeRef<leptos::html::Textarea> = NodeRef::new();

    let is_loading = move || loading.get();

    // Wrap on_submit in Rc so the closure that captures it can be cloned.
    let on_submit = Rc::new(on_submit);

    // Build the submit logic once, share via Rc.
    let submit_logic: Rc<dyn Fn()> = {
        let on_submit = on_submit.clone();
        Rc::new(move || {
            let text = input.get().trim().to_string();
            if text.is_empty() || is_loading() {
                return;
            }
            on_submit(text);
            input.set(String::new());
            if let Some(el) = textarea_ref.get() {
                let _ = el.focus();
            }
        })
    };

    let on_keydown = {
        let submit = submit_logic.clone();
        move |ev: leptos::ev::KeyboardEvent| {
            if ev.key() == "Enter" && !ev.shift_key() {
                ev.prevent_default();
                submit();
            }
        }
    };

    let on_input = move |ev: leptos::ev::Event| {
        input.set(event_target_value(&ev));
    };

    let on_click = {
        let submit = submit_logic.clone();
        move |_| {
            submit();
        }
    };

    let btn_disabled = move || is_loading();
    let input_disabled = move || is_loading();

    view! {
        <div class="border-t border-gray-200 bg-white px-4 py-3">
            <div class="flex items-end gap-2 max-w-4xl mx-auto">
                <textarea
                    node_ref=textarea_ref
                    class="flex-1 resize-none rounded-xl border border-gray-300 px-4 py-3
                           text-sm focus:outline-none focus:ring-2 focus:ring-black
                           focus:border-transparent disabled:bg-gray-100 disabled:cursor-not-allowed
                           transition-colors placeholder:text-gray-400"
                    placeholder="Type a message… (Shift+Enter for newline)"
                    rows="1"
                    prop:value=input
                    on:input=on_input
                    on:keydown=on_keydown
                    disabled=input_disabled
                />

                <button
                    class="flex-shrink-0 flex items-center justify-center w-10 h-10
                           rounded-xl bg-black text-white hover:bg-gray-800
                           disabled:bg-gray-300 disabled:cursor-not-allowed
                           transition-colors"
                    on:click=on_click
                    disabled=btn_disabled
                    aria-label="Send message"
                >
                    {move || if is_loading() {
                        view! {
                            <svg class="animate-spin h-5 w-5" viewBox="0 0 24 24" fill="none">
                                <circle class="opacity-25" cx="12" cy="12" r="10"
                                    stroke="currentColor" stroke-width="4"/>
                                <path class="opacity-75" fill="currentColor"
                                    d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z"/>
                            </svg>
                        }.into_any()
                    } else {
                        view! {
                            <svg class="h-5 w-5" viewBox="0 0 24 24" fill="none"
                                stroke="currentColor" stroke-width="2"
                                stroke-linecap="round" stroke-linejoin="round">
                                <line x1="22" y1="2" x2="11" y2="13"/>
                                <polygon points="22 2 15 22 11 13 2 9 22 2"/>
                            </svg>
                        }.into_any()
                    }}
                </button>
            </div>
        </div>
    }
}
