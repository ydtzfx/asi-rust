use leptos::prelude::*;
use leptos_meta::*;

/// Sign-in page that embeds Clerk's `<clerk-sign-in>` custom element.
#[component]
pub fn SignInPage() -> impl IntoView {
    let clerk_pk = option_env!("CLERK_PUBLISHABLE_KEY")
        .unwrap_or("pk_test_placeholder_key")
        .to_string();

    // Build the Clerk script tag using the builder pattern to set custom
    // data attributes that Leptos's typed view! macro doesn't support.
    let clerk_script = leptos::html::script()
        .attr("src", "https://js.clerk.com/v1/clerk.js")
        .attr("data-clerk-publishable-key", clerk_pk.clone())
        .attr("crossorigin", "anonymous");

    view! {
        <Title text="Sign In — ASI"/>
        <Meta name="description" content="Sign in to your ASI account."/>

        {clerk_script}

        <div class="min-h-[70vh] flex items-center justify-center px-4 py-16">
            <div class="w-full max-w-md">
                <div class="text-center mb-8">
                    <h1 class="text-3xl font-bold tracking-tight">"Welcome back"</h1>
                    <p class="mt-2 text-gray-500">
                        "Sign in to your ASI account to continue."
                    </p>
                </div>

                <div id="clerk-sign-in" class="w-full"></div>

                <p class="mt-6 text-center text-sm text-gray-400">
                    "Don't have an account? "
                    <a href="/sign-up" class="text-blue-600 hover:underline">
                        "Sign up"
                    </a>
                </p>
            </div>
        </div>
    }
}
