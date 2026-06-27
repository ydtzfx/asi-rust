use leptos::prelude::*;

#[component]
pub fn ServicesPage() -> impl IntoView {
    view! {
        <div class="max-w-4xl mx-auto px-4 py-16">
            <h1 class="text-4xl font-bold mb-6">"Services"</h1>
            <p class="text-lg text-gray-600 mb-10">
                "ASI offers a comprehensive suite of AI-powered development services."
            </p>

            <div class="grid md:grid-cols-2 gap-8">
                <div class="border border-gray-200 rounded-lg p-6">
                    <h2 class="text-2xl font-semibold mb-3">"Code Generation"</h2>
                    <p class="text-gray-600 mb-4">
                        "Generate production-ready code from natural language descriptions. Support for
                        multiple languages and frameworks including TypeScript, Rust, Python, and more."
                    </p>
                    <ul class="space-y-2 text-gray-600">
                        <li class="flex items-center gap-2">
                            <span class="text-green-500">"✓"</span>
                            <span>"Full project scaffolding"</span>
                        </li>
                        <li class="flex items-center gap-2">
                            <span class="text-green-500">"✓"</span>
                            <span>"Component generation"</span>
                        </li>
                        <li class="flex items-center gap-2">
                            <span class="text-green-500">"✓"</span>
                            <span>"API endpoint creation"</span>
                        </li>
                    </ul>
                </div>

                <div class="border border-gray-200 rounded-lg p-6">
                    <h2 class="text-2xl font-semibold mb-3">"Code Review"</h2>
                    <p class="text-gray-600 mb-4">
                        "Automated code reviews that catch bugs, security vulnerabilities, and
                        style issues before they reach production."
                    </p>
                    <ul class="space-y-2 text-gray-600">
                        <li class="flex items-center gap-2">
                            <span class="text-green-500">"✓"</span>
                            <span>"Security vulnerability detection"</span>
                        </li>
                        <li class="flex items-center gap-2">
                            <span class="text-green-500">"✓"</span>
                            <span>"Best practice enforcement"</span>
                        </li>
                        <li class="flex items-center gap-2">
                            <span class="text-green-500">"✓"</span>
                            <span>"Performance optimization"</span>
                        </li>
                    </ul>
                </div>

                <div class="border border-gray-200 rounded-lg p-6">
                    <h2 class="text-2xl font-semibold mb-3">"Self-Evolution"</h2>
                    <p class="text-gray-600 mb-4">
                        "Continuous autonomous improvement of your codebase through automated
                        analysis and optimization cycles."
                    </p>
                    <ul class="space-y-2 text-gray-600">
                        <li class="flex items-center gap-2">
                            <span class="text-green-500">"✓"</span>
                            <span>"Automated refactoring"</span>
                        </li>
                        <li class="flex items-center gap-2">
                            <span class="text-green-500">"✓"</span>
                            <span>"Dependency updates"</span>
                        </li>
                        <li class="flex items-center gap-2">
                            <span class="text-green-500">"✓"</span>
                            <span>"Test generation"</span>
                        </li>
                    </ul>
                </div>

                <div class="border border-gray-200 rounded-lg p-6">
                    <h2 class="text-2xl font-semibold mb-3">"Multi-Agent Collaboration"</h2>
                    <p class="text-gray-600 mb-4">
                        "Multiple specialized AI agents working together to solve complex
                        development tasks autonomously."
                    </p>
                    <ul class="space-y-2 text-gray-600">
                        <li class="flex items-center gap-2">
                            <span class="text-green-500">"✓"</span>
                            <span>"Parallel task execution"</span>
                        </li>
                        <li class="flex items-center gap-2">
                            <span class="text-green-500">"✓"</span>
                            <span>"Context sharing between agents"</span>
                        </li>
                        <li class="flex items-center gap-2">
                            <span class="text-green-500">"✓"</span>
                            <span>"Coordinated workflows"</span>
                        </li>
                    </ul>
                </div>
            </div>
        </div>
    }
}
