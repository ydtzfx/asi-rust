use leptos::prelude::*;
use leptos_meta::*;

/// Sign-up page that embeds Clerk's `<clerk-sign-up>` custom element.
#[component]
pub fn SignUpPage() -> impl IntoView {
    let clerk_pk = option_env!("CLERK_PUBLISHABLE_KEY")
        .unwrap_or("pk_test_placeholder_key")
        .to_string();

    let clerk_script = leptos::html::script()
        .attr("src", "https://js.clerk.com/v1/clerk.js")
        .attr("data-clerk-publishable-key", clerk_pk.clone())
        .attr("crossorigin", "anonymous");

    view! {
        <Title text="Create Account — ASI"/>
        <Meta name="description" content="Create your ASI account."/>

        {clerk_script}

        <div class="min-h-[70vh] flex items-center justify-center px-4 py-16">
            <div class="w-full max-w-md">
                <div class="text-center mb-8">
                    <h1 class="text-3xl font-bold tracking-tight">"Create your account"</h1>
                    <p class="mt-2 text-gray-500">
                        "Start building with ASI's multi-agent coding platform."
                    </p>
                </div>

                <div id="clerk-sign-up" class="w-full"></div>

                <p class="mt-6 text-center text-sm text-gray-400">
                    "Already have an account? "
                    <a href="/sign-in" class="text-blue-600 hover:underline">
                        "Sign in"
                    </a>
                </p>
            </div>
        </div>
    }
}
