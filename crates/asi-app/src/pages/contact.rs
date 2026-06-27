use leptos::prelude::*;

#[component]
pub fn ContactPage() -> impl IntoView {
    view! {
        <div class="max-w-4xl mx-auto px-4 py-16">
            <h1 class="text-4xl font-bold mb-6">"Contact Us"</h1>
            <p class="text-lg text-gray-600 mb-10">
                "Have questions or feedback? We'd love to hear from you."
            </p>

            <div class="grid md:grid-cols-2 gap-12">
                <div>
                    <form class="space-y-6">
                        <div>
                            <label for="name" class="block text-sm font-medium text-gray-700 mb-1">
                                "Name"
                            </label>
                            <input
                                type="text"
                                id="name"
                                name="name"
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                placeholder="Your name"
                            />
                        </div>
                        <div>
                            <label for="email" class="block text-sm font-medium text-gray-700 mb-1">
                                "Email"
                            </label>
                            <input
                                type="email"
                                id="email"
                                name="email"
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                placeholder="you@example.com"
                            />
                        </div>
                        <div>
                            <label for="subject" class="block text-sm font-medium text-gray-700 mb-1">
                                "Subject"
                            </label>
                            <select
                                id="subject"
                                name="subject"
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                            >
                                <option value="general">"General Inquiry"</option>
                                <option value="support">"Technical Support"</option>
                                <option value="billing">"Billing Question"</option>
                                <option value="other">"Other"</option>
                            </select>
                        </div>
                        <div>
                            <label for="message" class="block text-sm font-medium text-gray-700 mb-1">
                                "Message"
                            </label>
                            <textarea
                                id="message"
                                name="message"
                                rows=5
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
                                placeholder="Your message..."
                            ></textarea>
                        </div>
                        <button
                            type="submit"
                            class="px-6 py-3 bg-black text-white rounded-lg font-medium hover:bg-gray-800 transition"
                        >
                            "Send Message"
                        </button>
                    </form>
                </div>

                <div class="space-y-8">
                    <div>
                        <h2 class="text-xl font-semibold mb-3">"Email"</h2>
                        <p class="text-gray-600">
                            <a href="mailto:hello@asi.dev" class="text-blue-600 hover:underline">
                                "hello@asi.dev"
                            </a>
                        </p>
                    </div>

                    <div>
                        <h2 class="text-xl font-semibold mb-3">"GitHub"</h2>
                        <p class="text-gray-600">
                            <a href="https://github.com/asi" class="text-blue-600 hover:underline">
                                "github.com/asi"
                            </a>
                        </p>
                    </div>

                    <div>
                        <h2 class="text-xl font-semibold mb-3">"Documentation"</h2>
                        <p class="text-gray-600">
                            "Visit our documentation for guides, API references, and tutorials."
                        </p>
                        <a href="#" class="inline-block mt-2 text-blue-600 hover:underline">
                            "View Documentation →"
                        </a>
                    </div>

                    <div>
                        <h2 class="text-xl font-semibold mb-3">"Office"</h2>
                        <p class="text-gray-600">
                            "ASI Headquarters"<br/>
                            "123 Innovation Drive"<br/>
                            "San Francisco, CA 94105"
                        </p>
                    </div>
                </div>
            </div>
        </div>
    }
}
