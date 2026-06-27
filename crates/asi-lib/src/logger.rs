use tracing::{debug as trace_debug, error as trace_error, info as trace_info, warn as trace_warn};

pub fn debug(msg: &str, fields: &[(&str, &str)]) {
    trace_debug!(target: "asi", msg, fields = ?fields);
}

pub fn info(msg: &str, fields: &[(&str, &str)]) {
    trace_info!(target: "asi", msg, fields = ?fields);
}

pub fn warn(msg: &str, fields: &[(&str, &str)]) {
    trace_warn!(target: "asi", msg, fields = ?fields);
}

pub fn error(msg: &str, fields: &[(&str, &str)]) {
    trace_error!(target: "asi", msg, fields = ?fields);
}

pub fn init_logger() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .with_timer(tracing_subscriber::fmt::time::UtcTime::rfc_3339())
        .init();
}
