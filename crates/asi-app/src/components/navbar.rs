use leptos::prelude::*;

#[component]
pub fn Navbar() -> impl IntoView {
    let mobile_open = RwSignal::new(false);

    let toggle_mobile = move |_| {
        mobile_open.update(|v| *v = !*v);
    };

    let close_mobile = move |_| {
        mobile_open.set(false);
    };

    view! {
        <nav class="bg-white border-b border-gray-200">
            <div class="max-w-6xl mx-auto px-4">
                <div class="flex items-center justify-between h-16">
                    // Logo
                    <a href="/" class="text-xl font-bold tracking-tight hover:text-gray-700 transition">
                        "ASI"
                    </a>

                    // Desktop navigation
                    <div class="hidden md:flex items-center gap-6">
                        <a href="/" class="text-gray-600 hover:text-black transition">"Home"</a>
                        <a href="/about" class="text-gray-600 hover:text-black transition">"About"</a>
                        <a href="/services" class="text-gray-600 hover:text-black transition">"Services"</a>
                        <a href="/news" class="text-gray-600 hover:text-black transition">"News"</a>
                        <a href="/contact" class="text-gray-600 hover:text-black transition">"Contact"</a>
                    </div>

                    // Desktop auth buttons
                    <div class="hidden md:flex items-center gap-3">
                        <a href="/sign-in" class="px-4 py-2 text-sm text-gray-700 hover:text-black transition">"Sign In"</a>
                        <a href="/dashboard"
                            class="px-4 py-2 text-sm bg-black text-white rounded-lg font-medium hover:bg-gray-800 transition"
                        >
                            "Dashboard"
                        </a>
                    </div>

                    // Mobile hamburger
                    <button
                        class="md:hidden p-2 rounded-lg hover:bg-gray-100 transition"
                        on:click=toggle_mobile
                        aria-label="Toggle navigation menu"
                    >
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                d="M4 6h16M4 12h16M4 18h16"
                            />
                        </svg>
                    </button>
                </div>

                // Mobile menu
                <Show when=move || mobile_open.get()>
                    <div class="md:hidden pb-4 border-t border-gray-100 pt-4">
                        <div class="flex flex-col gap-3">
                            <a href="/" on:click=close_mobile class="text-gray-600 hover:text-black transition">"Home"</a>
                            <a href="/about" on:click=close_mobile class="text-gray-600 hover:text-black transition">"About"</a>
                            <a href="/services" on:click=close_mobile class="text-gray-600 hover:text-black transition">"Services"</a>
                            <a href="/news" on:click=close_mobile class="text-gray-600 hover:text-black transition">"News"</a>
                            <a href="/contact" on:click=close_mobile class="text-gray-600 hover:text-black transition">"Contact"</a>
                            <hr class="my-2"/>
                            <a href="/sign-in" on:click=close_mobile class="text-gray-700 hover:text-black transition">"Sign In"</a>
                            <a href="/dashboard" on:click=close_mobile
                                class="px-4 py-2 text-center bg-black text-white rounded-lg font-medium hover:bg-gray-800 transition"
                            >
                                "Dashboard"
                            </a>
                        </div>
                    </div>
                </Show>
            </div>
        </nav>
    }
}
