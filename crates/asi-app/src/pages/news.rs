use leptos::prelude::*;

#[component]
pub fn NewsPage() -> impl IntoView {
    view! {
        <div class="max-w-4xl mx-auto px-4 py-16">
            <h1 class="text-4xl font-bold mb-6">"News"</h1>
            <p class="text-lg text-gray-600 mb-10">
                "Latest updates and announcements from the ASI team."
            </p>

            <div class="space-y-8">
                <article class="border-b border-gray-200 pb-8">
                    <time class="text-sm text-gray-400">"June 27, 2026"</time>
                    <h2 class="text-2xl font-semibold mt-2 mb-3">
                        "ASI Platform v1.0 Released"
                    </h2>
                    <p class="text-gray-600">
                        "We are excited to announce the release of ASI Platform v1.0, featuring
                        our new multi-agent architecture, improved code generation capabilities,
                        and the self-evolution system that continuously improves your codebase."
                    </p>
                    <a href="#" class="inline-block mt-3 text-blue-600 hover:underline">
                        "Read more →"
                    </a>
                </article>

                <article class="border-b border-gray-200 pb-8">
                    <time class="text-sm text-gray-400">"June 15, 2026"</time>
                    <h2 class="text-2xl font-semibold mt-2 mb-3">
                        "Rust Rewrite: Performance Milestone"
                    </h2>
                    <p class="text-gray-600">
                        "The ASI Rust rewrite is progressing well. We have achieved a 3x
                        performance improvement in code generation latency and significantly
                        reduced memory usage compared to the TypeScript version."
                    </p>
                    <a href="#" class="inline-block mt-3 text-blue-600 hover:underline">
                        "Read more →"
                    </a>
                </article>

                <article class="border-b border-gray-200 pb-8">
                    <time class="text-sm text-gray-400">"May 28, 2026"</time>
                    <h2 class="text-2xl font-semibold mt-2 mb-3">
                        "DeepSeek Integration Now Available"
                    </h2>
                    <p class="text-gray-600">
                        "ASI now supports DeepSeek as a primary AI provider alongside Ollama.
                        Users can leverage DeepSeek's powerful code generation capabilities
                        with automatic fallback to local models."
                    </p>
                    <a href="#" class="inline-block mt-3 text-blue-600 hover:underline">
                        "Read more →"
                    </a>
                </article>

                <article class="pb-8">
                    <time class="text-sm text-gray-400">"May 10, 2026"</time>
                    <h2 class="text-2xl font-semibold mt-2 mb-3">
                        "Introducing Self-Evolution Capability"
                    </h2>
                    <p class="text-gray-600">
                        "We have implemented a novel self-evolution system that allows ASI to
                        autonomously identify and implement improvements to its own codebase,
                        making the platform continuously better over time."
                    </p>
                    <a href="#" class="inline-block mt-3 text-blue-600 hover:underline">
                        "Read more →"
                    </a>
                </article>
            </div>
        </div>
    }
}
