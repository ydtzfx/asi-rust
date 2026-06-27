use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::*;
use leptos_router::path;

use crate::auth::*;
use crate::components::{Footer, Navbar};
use crate::contexts::UserProvider;
use crate::pages::*;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="ASI — AI Coding Assistant"/>
        <Meta charset="UTF-8"/>
        <Meta name="viewport" content="width=device-width, initial-scale=1.0"/>
        <Meta name="description" content="ASI — AI-powered multi-agent coding assistant platform with autonomous code generation, review, and self-evolution."/>

        <Router>
            <UserProvider>
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
            </UserProvider>
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
