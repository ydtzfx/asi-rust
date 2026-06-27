use leptos::prelude::*;

#[component]
pub fn LandingPage() -> impl IntoView {
    view! {
        <div>
            // Hero section
            <section class="max-w-4xl mx-auto px-4 py-20 text-center">
                <h1 class="text-5xl font-bold tracking-tight">
                    "ASI — AI-Powered Coding Assistant"
                </h1>
                <p class="mt-6 text-xl text-gray-500 max-w-2xl mx-auto">
                    "Multi-agent coding platform with autonomous code generation, review, and self-evolution capabilities."
                </p>
                <div class="mt-10 flex gap-4 justify-center">
                    <a href="/dashboard" class="px-6 py-3 bg-black text-white rounded-lg font-medium hover:bg-gray-800 transition">
                        "Get Started"
                    </a>
                    <a href="/about" class="px-6 py-3 border border-gray-300 rounded-lg font-medium hover:bg-gray-50 transition">
                        "Learn More"
                    </a>
                </div>
            </section>

            // Features section
            <section class="bg-gray-50 py-20">
                <div class="max-w-5xl mx-auto px-4">
                    <h2 class="text-3xl font-bold text-center mb-12">"Key Features"</h2>
                    <div class="grid md:grid-cols-3 gap-8">
                        <div class="bg-white p-6 rounded-lg shadow-sm">
                            <h3 class="text-xl font-semibold mb-3">"Multi-Agent Architecture"</h3>
                            <p class="text-gray-600">
                                "Specialized agents for code generation, review, and optimization working together autonomously."
                            </p>
                        </div>
                        <div class="bg-white p-6 rounded-lg shadow-sm">
                            <h3 class="text-xl font-semibold mb-3">"Self-Evolution"</h3>
                            <p class="text-gray-600">
                                "The system continuously learns and improves its own codebase through automated evolution cycles."
                            </p>
                        </div>
                        <div class="bg-white p-6 rounded-lg shadow-sm">
                            <h3 class="text-xl font-semibold mb-3">"Multi-Model Support"</h3>
                            <p class="text-gray-600">
                                "Seamless integration with DeepSeek, Ollama, and other AI providers with automatic fallback."
                            </p>
                        </div>
                    </div>
                </div>
            </section>

            // CTA section
            <section class="max-w-4xl mx-auto px-4 py-20 text-center">
                <h2 class="text-3xl font-bold mb-4">"Ready to transform your development workflow?"</h2>
                <p class="text-gray-500 mb-8">"Join the future of AI-assisted coding."</p>
                <a href="/sign-up" class="px-6 py-3 bg-black text-white rounded-lg font-medium hover:bg-gray-800 transition">
                    "Get Started Free"
                </a>
            </section>
        </div>
    }
}
