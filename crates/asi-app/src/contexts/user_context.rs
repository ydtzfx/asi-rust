use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// User information returned from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub sub: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub org_id: Option<String>,
}

/// Reactive user context shared through the component tree.
#[derive(Clone)]
pub struct UserContext {
    /// The current user, or `None` if not authenticated.
    pub user: RwSignal<Option<UserInfo>>,
    /// True while fetching user info on mount.
    pub loading: RwSignal<bool>,
    /// Error message if the fetch failed.
    pub error: RwSignal<Option<String>>,
}

/// Helper to access the user context from any child component.
/// Returns `None` if no `UserProvider` is in the tree.
pub fn use_user() -> Option<UserContext> {
    use_context::<UserContext>()
}

/// Wraps the application and provides the current user via Leptos context.
///
/// On mount (CSR), this component fetches `GET /api/user/me` and populates
/// the reactive `UserContext`.  The context is available to every descendent
/// via `use_user()`.
///
/// While the fetch is in flight `loading` is `true`; on error the `error`
/// signal is populated and the component still renders children with
/// `user = None`.
#[component]
pub fn UserProvider(children: Children) -> impl IntoView {
    let user = RwSignal::new(None::<UserInfo>);
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);

    // Client-side: fetch the current user on mount.
    fetch_user_client(user, loading, error);

    provide_context(UserContext {
        user,
        loading,
        error,
    });

    children()
}

/// Spawn a client-side fetch of `/api/user/me` on mount.
///
/// This function is a no-op (empty) when compiled for the server.
#[cfg(not(target_arch = "wasm32"))]
fn fetch_user_client(
    _user: RwSignal<Option<UserInfo>>,
    loading: RwSignal<bool>,
    _error: RwSignal<Option<String>>,
) {
    // Server-side: we never fetch; loading stays true until hydration.
    loading.set(false);
}

/// Client-side: actually fetch the user info from the server.
#[cfg(target_arch = "wasm32")]
fn fetch_user_client(
    user: RwSignal<Option<UserInfo>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    wasm_bindgen_futures::spawn_local(async move {
        match fetch_current_user().await {
            Ok(Some(info)) => {
                user.set(Some(info));
                loading.set(false);
            }
            Ok(None) => {
                // Signed out – user stays None.
                loading.set(false);
            }
            Err(err) => {
                error.set(Some(err));
                loading.set(false);
            }
        }
    });
}

/// Fetch the current user from the server's `/api/user/me` endpoint.
///
/// Client-side only; returns:
/// - `Ok(Some(user))` on success
/// - `Ok(None)` when not authenticated (HTTP 401 / 403)
/// - `Err(msg)` on network or parse failure
#[cfg(target_arch = "wasm32")]
async fn fetch_current_user() -> Result<Option<UserInfo>, String> {
    use gloo_net::http::Request;

    match Request::get("/api/user/me").send().await {
        Ok(response) => {
            let status = response.status();
            if status == 401 || status == 403 {
                return Ok(None);
            }
            if !response.ok() {
                return Err(format!("Request failed: HTTP {}", status));
            }
            let info: UserInfo =
                response.json().await.map_err(|e| format!("JSON parse error: {}", e))?;
            Ok(Some(info))
        }
        Err(e) => Err(format!("Network error: {}", e)),
    }
}
