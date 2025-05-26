use std::path::Path;
use std::sync::Arc;
use tokio::net::UnixListener;
use tokio::sync::Notify;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server;
use crate::grpc::constants;
use crate::grpc::handler::ZileanService;
use crate::imdb::{ImdbIngestor, ImdbSearcher};
use crate::proto::zilean_rust_server_server::ZileanRustServerServer;

fn cleanup_socket() -> anyhow::Result<()> {
    if Path::new(constants::ZILEAN_SOCKET_PATH).exists() {
        std::fs::remove_file(constants::ZILEAN_SOCKET_PATH)?;
    }
    Ok(())
}

pub async fn start_server() -> anyhow::Result<()> {
    cleanup_socket()?;
    
    let listener = UnixListener::bind(constants::ZILEAN_SOCKET_PATH)?;
    let incoming = UnixListenerStream::new(listener);

    let searcher = Arc::new(tokio::sync::RwLock::new(ImdbSearcher::new()?));
    let ingestor = Arc::new(ImdbIngestor::new(searcher.clone()));
    let shutdown_notify = Arc::new(Notify::new());

    let state = Arc::new(crate::grpc::handler::SharedState {
        searcher,
        ingestor,
        shutdown_notify: shutdown_notify.clone(),
    });

    let service = ZileanService { state };

    tracing::info!("Zilean gRPC server listening on unix://{}", constants::ZILEAN_SOCKET_PATH);

    Server::builder()
        .add_service(ZileanRustServerServer::new(service))
        .serve_with_incoming_shutdown(incoming, shutdown_notify.notified())
        .await?;

    cleanup_socket()?;
    tracing::info!("Zilean server shutdown complete");
    Ok(())
}