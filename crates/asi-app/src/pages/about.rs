use leptos::prelude::*;

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <div class="max-w-4xl mx-auto px-4 py-16">
            <h1 class="text-4xl font-bold mb-6">"About ASI"</h1>

            <div class="prose prose-gray max-w-none">
                <p class="text-lg text-gray-600 mb-6">
                    "ASI (Autonomous Software Intelligence) is a next-generation AI-powered coding assistant
                    platform that leverages multi-agent architecture to autonomously generate, review, and
                    optimize code."
                </p>

                <h2 class="text-2xl font-semibold mt-10 mb-4">"Our Mission"</h2>
                <p class="text-gray-600 mb-6">
                    "To accelerate software development by providing intelligent, autonomous coding assistance
                    that adapts to each developer's workflow. We believe in a future where AI and human
                    developers collaborate seamlessly to build better software, faster."
                </p>

                <h2 class="text-2xl font-semibold mt-10 mb-4">"How It Works"</h2>
                <div class="space-y-4 mb-6">
                    <div class="flex gap-4">
                        <span class="text-blue-600 font-bold text-xl">1.</span>
                        <div>
                            <h3 class="font-semibold">"Code Generation"</h3>
                            <p class="text-gray-600">
                                "Describe what you want to build, and our agents generate production-ready code."
                            </p>
                        </div>
                    </div>
                    <div class="flex gap-4">
                        <span class="text-blue-600 font-bold text-xl">2.</span>
                        <div>
                            <h3 class="font-semibold">"Automated Review"</h3>
                            <p class="text-gray-600">
                                "Every change is reviewed for bugs, security issues, and best practices."
                            </p>
                        </div>
                    </div>
                    <div class="flex gap-4">
                        <span class="text-blue-600 font-bold text-xl">3.</span>
                        <div>
                            <h3 class="font-semibold">"Self-Evolution"</h3>
                            <p class="text-gray-600">
                                "The system continuously improves itself through automated evolution cycles."
                            </p>
                        </div>
                    </div>
                </div>

                <h2 class="text-2xl font-semibold mt-10 mb-4">"Technology"</h2>
                <p class="text-gray-600 mb-6">
                    "Built with Rust for performance and safety, leveraging the Leptos framework for
                    full-stack reactivity, and integrating with leading AI providers for state-of-the-art
                    code intelligence."
                </p>
            </div>
        </div>
    }
}
