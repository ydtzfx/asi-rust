use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::components::{Footer, Navbar};
use crate::pages::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Html/>
        <Title text="ASI — AI Coding Assistant"/>
        <Meta charset="UTF-8"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Meta name="description" content="ASI — AI-powered multi-agent coding assistant platform with autonomous code generation, review, and self-evolution."/>

        <Router>
            <Navbar/>
            <main class="min-h-screen">
                <Routes fallback=|| view! { <NotFound/> }>
                    <Route path=path!("") view=LandingPage/>
                    <Route path=path!("about") view=AboutPage/>
                    <Route path=path!("services") view=ServicesPage/>
                    <Route path=path!("news") view=NewsPage/>
                    <Route path=path!("contact") view=ContactPage/>
                    <Route path=path!("sign-in") view=SignInPage/>
                    <Route path=path!("sign-up") view=SignUpPage/>
                    <Route path=path!("dashboard") view=DashboardPage/>
                </Routes>
            </main>
            <Footer/>
        </Router>
    }
}

#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center py-20">
            <h1 class="text-6xl font-bold text-gray-300">"404"</h1>
            <p class="mt-4 text-xl text-gray-500">"Page not found"</p>
            <a href="/" class="mt-6 text-blue-600 hover:underline">"Go home"</a>
        </div>
    }
}

#[component]
pub fn SignInPage() -> impl IntoView {
    view! {
        <div class="max-w-md mx-auto px-4 py-20 text-center">
            <h1 class="text-3xl font-bold">"Sign In"</h1>
            <p class="mt-4 text-gray-500">"Sign in to access your dashboard and agents."</p>
            <div class="mt-8">
                <a href="/dashboard" class="px-6 py-3 bg-black text-white rounded-lg font-medium hover:bg-gray-800 transition">
                    "Continue to Dashboard"
                </a>
            </div>
        </div>
    }
}

#[component]
pub fn SignUpPage() -> impl IntoView {
    view! {
        <div class="max-w-md mx-auto px-4 py-20 text-center">
            <h1 class="text-3xl font-bold">"Sign Up"</h1>
            <p class="mt-4 text-gray-500">"Create an account to get started with ASI."</p>
            <div class="mt-8">
                <a href="/sign-in" class="px-6 py-3 bg-black text-white rounded-lg font-medium hover:bg-gray-800 transition">
                    "Sign In Instead"
                </a>
            </div>
        </div>
    }
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    view! {
        <div class="max-w-4xl mx-auto px-4 py-20 text-center">
            <h1 class="text-3xl font-bold">"Dashboard"</h1>
            <p class="mt-4 text-gray-500">"Your coding assistant dashboard — coming soon."</p>
        </div>
    }
}
