use crate::configuration::config::AppConfig;
use crate::dmm::db_service::PgDmmDbService;
use crate::dmm::page_parser::DmmFileEntryProcessor;
use crate::dmm::repo_manager::DmmRepoManager;
use crate::grpc::constants;
use crate::grpc::handler::{SharedState, ZileanService};
use crate::imdb::{ImdbIngestor, ImdbSearcher};
use crate::proto::zilean_rust_server_server::ZileanRustServerServer;
use arc_swap::ArcSwap;
use std::path::Path;
use std::sync::Arc;
use tokio::net::UnixListener;
use tokio::sync::Notify;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server;

const DESCRIPTOR_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/descriptor.bin"));

fn cleanup_socket() -> anyhow::Result<()> {
    if Path::new(constants::ZILEAN_SOCKET_PATH).exists() {
        std::fs::remove_file(constants::ZILEAN_SOCKET_PATH)?;
    }
    Ok(())
}

pub async fn start_server(app_config: AppConfig) -> anyhow::Result<()> {
    cleanup_socket()?;

    let listener = UnixListener::bind(constants::ZILEAN_SOCKET_PATH)?;
    let incoming = UnixListenerStream::new(listener);
    let shutdown_notify = Arc::new(Notify::new());

    let state = construct_state(app_config, shutdown_notify.clone()).await;

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(DESCRIPTOR_BYTES)
        .build_v1()?;

    let service = ZileanService { state };

    tracing::info!(
        "Zilean gRPC server listening on unix://{}",
        constants::ZILEAN_SOCKET_PATH
    );

    Server::builder()
        .max_frame_size(10 * 1024 * 1024)
        .add_service(ZileanRustServerServer::new(service))
        .add_service(reflection_service)
        .serve_with_incoming_shutdown(incoming, shutdown_notify.notified())
        .await?;

    cleanup_socket()?;
    tracing::info!("Zilean server shutdown complete");
    Ok(())
}

async fn construct_state(app_config: AppConfig, shutdown_notify: Arc<Notify>) -> Arc<SharedState> {
    let app_config = Arc::new(app_config);

    let searcher = Arc::new(ArcSwap::new(Arc::new(
        ImdbSearcher::new(app_config.imdb_minimum_score)
            .expect("Failed to initialize ImdbSearcher"),
    )));

    let ingestor = Arc::new(ImdbIngestor::new(searcher.clone()));

    let shutdown_notify = shutdown_notify;

    let dmm_repo_manager = Arc::new(DmmRepoManager::new(
        &app_config.dmm_repo_url,
        &app_config.dmm_local_path,
    ));

    let dmm_service = Arc::new(
        PgDmmDbService::connect(&app_config.database_url)
            .await
            .unwrap(),
    );

    let dmm_page_parser = Arc::new(DmmFileEntryProcessor::new(
        dmm_service,
        searcher.clone(),
        app_config.dmm_local_path.clone(),
    ));

    Arc::new(SharedState {
        searcher,
        ingestor,
        dmm_repo_manager,
        dmm_page_parser,
        shutdown_notify: shutdown_notify.clone(),
        app_config,
    })
}
