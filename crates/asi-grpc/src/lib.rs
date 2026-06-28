//! ASI gRPC — high-performance gRPC API server.
//! Alternative to HTTP/SSE for internal service-to-service communication.

pub mod server;

/// gRPC server configuration.
pub struct GrpcConfig {
    pub host: String,
    pub port: u16,
}

impl Default for GrpcConfig {
    fn default() -> Self {
        Self { host: "0.0.0.0".into(), port: 50051 }
    }
}

/// Start the gRPC server (placeholder — full implementation with tonic-build).
pub async fn start_grpc_server(config: GrpcConfig) -> Result<(), Box<dyn std::error::Error>> {
    let addr: std::net::SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    tracing::info!("gRPC server starting on {}", addr);
    // tonic::transport::Server::builder()
    //     .add_service(AsiServer::new(AsiServiceImpl::new()))
    //     .serve(addr)
    //     .await?;
    tracing::info!("gRPC server ready (placeholder — awaiting tonic-build proto generation)");
    // Keep the function valid: bind but don't actually serve yet.
    let _ = tokio::net::TcpListener::bind(addr).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_grpc_config() {
        let config = GrpcConfig::default();
        assert_eq!(config.port, 50051);
    }

    #[test]
    fn test_grpc_server_bind() {
        // Verify the config is valid.
        let config = GrpcConfig::default();
        let addr = format!("{}:{}", config.host, config.port);
        assert!(addr.parse::<std::net::SocketAddr>().is_ok());
    }
}
