use axum::{
    Router,
    http::{HeaderMap, HeaderValue, StatusCode},
    routing::get,
};

/// GET /api/metrics — Prometheus-style text output.
async fn get_metrics() -> (StatusCode, HeaderMap, String) {
    let stats = asi_lib::telemetry::get_api_stats();
    let mut output = String::new();

    output.push_str("# HELP asi_api_calls_total Total API call count\n");
    output.push_str("# TYPE asi_api_calls_total counter\n");
    for (route, count) in &stats {
        output.push_str(&format!(
            "asi_api_calls_total{{route=\"{}\"}} {}\n",
            route, count
        ));
    }

    output.push_str("\n# HELP asi_server_info ASI server metadata\n");
    output.push_str("# TYPE asi_server_info gauge\n");
    output.push_str(&format!(
        "asi_server_info{{version=\"{}\",name=\"asi-server\"}} 1\n",
        env!("CARGO_PKG_VERSION")
    ));

    output.push_str("\n# HELP asi_db_pool_size Database connection pool size\n");
    output.push_str("# TYPE asi_db_pool_size gauge\n");
    let pool = asi_db::get_db();
    let size = pool.size() as u64;
    output.push_str(&format!("asi_db_pool_size {} 1\n", size));

    let mut headers = HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );

    (StatusCode::OK, headers, output)
}

pub fn routes() -> Router {
    Router::new().route("/metrics", get(get_metrics))
}
