use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="bg-gray-50 border-t border-gray-200">
            <div class="max-w-6xl mx-auto px-4 py-12">
                <div class="grid md:grid-cols-4 gap-8">
                    // Brand
                    <div>
                        <h3 class="text-lg font-bold mb-4">"ASI"</h3>
                        <p class="text-sm text-gray-600">
                            "AI-powered multi-agent coding assistant platform."
                        </p>
                    </div>

                    // Product links
                    <div>
                        <h4 class="text-sm font-semibold text-gray-900 uppercase mb-4">"Product"</h4>
                        <ul class="space-y-2">
                            <li><a href="/" class="text-sm text-gray-600 hover:text-black transition">"Home"</a></li>
                            <li><a href="/about" class="text-sm text-gray-600 hover:text-black transition">"About"</a></li>
                            <li><a href="/services" class="text-sm text-gray-600 hover:text-black transition">"Services"</a></li>
                            <li><a href="/news" class="text-sm text-gray-600 hover:text-black transition">"News"</a></li>
                            <li><a href="/contact" class="text-sm text-gray-600 hover:text-black transition">"Contact"</a></li>
                        </ul>
                    </div>

                    // Resources
                    <div>
                        <h4 class="text-sm font-semibold text-gray-900 uppercase mb-4">"Resources"</h4>
                        <ul class="space-y-2">
                            <li><a href="#" class="text-sm text-gray-600 hover:text-black transition">"Documentation"</a></li>
                            <li><a href="#" class="text-sm text-gray-600 hover:text-black transition">"API Reference"</a></li>
                            <li><a href="#" class="text-sm text-gray-600 hover:text-black transition">"GitHub"</a></li>
                            <li><a href="#" class="text-sm text-gray-600 hover:text-black transition">"Status"</a></li>
                        </ul>
                    </div>

                    // Company
                    <div>
                        <h4 class="text-sm font-semibold text-gray-900 uppercase mb-4">"Company"</h4>
                        <ul class="space-y-2">
                            <li><a href="/about" class="text-sm text-gray-600 hover:text-black transition">"About Us"</a></li>
                            <li><a href="/contact" class="text-sm text-gray-600 hover:text-black transition">"Contact"</a></li>
                            <li><a href="#" class="text-sm text-gray-600 hover:text-black transition">"Privacy Policy"</a></li>
                            <li><a href="#" class="text-sm text-gray-600 hover:text-black transition">"Terms of Service"</a></li>
                        </ul>
                    </div>
                </div>

                <div class="mt-12 pt-8 border-t border-gray-200 text-center">
                    <p class="text-sm text-gray-500">
                        "© " {2026} " ASI. All rights reserved."
                    </p>
                </div>
            </div>
        </footer>
    }
}
