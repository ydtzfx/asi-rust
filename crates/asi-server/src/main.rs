use asi_lib::config::Config;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    asi_lib::logger::init_logger();
    let config = Config::from_env();
    config.validate();

    let pool = asi_db::init_db(&config.database_url)
        .await
        .expect("DB init failed");
    asi_db::set_db_pool(pool);
    asi_lib::logger::info("Database initialized", &[("url", &config.database_url)]);

    asi_server::startup::run_startup_hooks();

    let leptos_options = leptos::config::LeptosOptions::builder()
        .output_name("asi-app")
        .site_root(".")
        .site_pkg_dir("pkg")
        .env(leptos::config::Env::DEV)
        .build();

    let app = asi_server::router::build_router(leptos_options);
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    asi_lib::logger::info("Server listening", &[("addr", &addr.to_string())]);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(asi_server::shutdown::shutdown_signal())
        .await
        .unwrap();
}
