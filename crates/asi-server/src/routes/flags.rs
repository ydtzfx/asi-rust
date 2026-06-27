use axum::{Json, Router, extract::Query, routing::get};
use serde_json::Value;
use std::collections::HashMap;

/// GET /api/flags — returns all flags with their current values.
async fn list_flags() -> Json<Value> {
    let flags = asi_lib::flags::get_all_flags();
    let map: HashMap<String, bool> = flags.into_iter().collect();
    Json(serde_json::json!(map))
}

/// POST /api/flags?set=name — toggles the named flag.
async fn toggle_flag(Query(params): Query<HashMap<String, String>>) -> Json<Value> {
    let flag_name = match params.get("set") {
        Some(name) if !name.is_empty() => name.clone(),
        _ => {
            return Json(serde_json::json!({
                "error": "Missing 'set' query parameter",
            }));
        }
    };

    let current = asi_lib::flags::flag(&flag_name);
    if current {
        asi_lib::flags::reset_flag(&flag_name);
    } else {
        asi_lib::flags::set_flag(&flag_name);
    }

    Json(serde_json::json!({
        "flag": flag_name,
        "previous": current,
        "current": !current,
    }))
}

pub fn routes() -> Router {
    Router::new().route("/flags", get(list_flags).post(toggle_flag))
}
