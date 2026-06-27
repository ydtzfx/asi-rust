use std::sync::atomic::{AtomicU64, Ordering};

static TOTAL_TOKENS: AtomicU64 = AtomicU64::new(0);

pub fn track_tokens(count: u64) {
    TOTAL_TOKENS.fetch_add(count, Ordering::SeqCst);
}

pub fn get_token_usage() -> u64 {
    TOTAL_TOKENS.load(Ordering::SeqCst)
}
