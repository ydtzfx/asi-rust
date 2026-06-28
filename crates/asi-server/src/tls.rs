/// TLS configuration.
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

impl TlsConfig {
    /// Load TLS config from environment. Returns `None` if not configured.
    pub fn from_env() -> Option<Self> {
        let cert = std::env::var("TLS_CERT_PATH").ok()?;
        let key = std::env::var("TLS_KEY_PATH").ok()?;
        Some(Self {
            cert_path: cert,
            key_path: key,
        })
    }
}
