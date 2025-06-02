use std::pin::Pin;
use std::sync::Arc;
use arc_swap::ArcSwap;
use tokio::sync::{mpsc, Notify};
use tokio_stream::Stream;
use tokio_stream::wrappers::{UnboundedReceiverStream};
use parsett_rust::{parse_batch};
use tonic::{Request, Response, Status};
use crate::configuration::config::AppConfig;
use crate::dmm::page_parser::DmmFileEntryProcessor;
use crate::dmm::repo_manager::DmmRepoManager;
use crate::grpc::mapping::{map_parsed_title, map_to_empty_on_error};
use crate::imdb::{ImdbIngestor, ImdbSearcher};
use crate::proto::zilean_rust_server_server::ZileanRustServer;
use crate::proto::*;

# [allow(unused)]
pub struct SharedState {
    pub searcher: Arc<ArcSwap<ImdbSearcher>>,
    pub ingestor: Arc<ImdbIngestor>,
    pub shutdown_notify: Arc<Notify>,
    pub app_config: Arc<AppConfig>,
    pub dmm_repo_manager: Arc<DmmRepoManager>,
    pub dmm_page_parser: Arc<DmmFileEntryProcessor>,
}

pub struct ZileanService {
    pub state: Arc<SharedState>,
}

#[tonic::async_trait]
impl<'a> ZileanRustServer for ZileanService {
    async fn ingest_imdb(
        &self,
        request: Request<IngestImdbRequest>,
    ) -> Result<Response<IngestImdbResponse>, Status> {
        tracing::info!("Downloading and Indexing IMDb file...");
        let req = request.into_inner();
        let indexed = self.state
            .ingestor
            .ingest_imdb_data(&req).await.unwrap();

        tracing::info!("Ingestion complete, indexed {} new documents", indexed);
        Ok(Response::new(IngestImdbResponse {}))
    }
    type IngestDmmPagesStream = Pin<Box<dyn Stream<Item = Result<ParsedDmmPageEntry, Status>> + Send + 'static>>;

    async fn ingest_dmm_pages(
        &self,
        _request: Request<IngestDmmPagesRequest>,
    ) -> Result<Response<Self::IngestDmmPagesStream>, Status> {
        tracing::info!("Performing DMM ingestion and scrape...");

        if let Err(err) = self.state.dmm_repo_manager.sync_repo() {
            return Err(Status::internal(format!("Failed to sync repo: {err}")));
        }

        let parser = Arc::clone(&self.state.dmm_page_parser);
        let stream = parser.stream_parsed_pages();
        Ok(Response::new(Box::pin(stream)))
    }

    async fn search_imdb(
        &self,
        request: Request<SearchImdbRequest>,
    ) -> Result<Response<SearchImdbResponse>, Status> {
        let req = request.into_inner();
        let searcher = self.state.searcher.load();
        let matches = searcher.search(&req.title, &req.category, req.year);
        Ok(Response::new(SearchImdbResponse { matches }))
    }

    type ParseTorrentTitlesStream = Pin<Box<dyn Stream<Item = Result<TorrentTitleResponse, Status>> + Send>>;


    async fn parse_torrent_titles(
        &self,
        request: Request<tonic::Streaming<TorrentTitleRequest>>,
    ) -> Result<Response<Self::ParseTorrentTitlesStream>, Status> {
        let mut inbound = request.into_inner();
        let (tx, rx) = mpsc::unbounded_channel();

        let mut requests = Vec::new();
        while let Some(req) = inbound.message().await? {
            requests.push(req);
        }

        let titles: Vec<&str> = requests.iter().map(|r| r.title.as_str()).collect();

        let results = parse_batch(titles);

        for (req, parse_result) in requests.into_iter().zip(results.into_iter()) {
            let response = match parse_result {
                Ok(parsed) => map_parsed_title(&req.info_hash, &req.title, parsed),
                Err(_) => map_to_empty_on_error(&req.info_hash, &req.title),
            };
            if tx.send(Ok(response)).is_err() {
                break;
            }
        }

        Ok(Response::new(Box::pin(UnboundedReceiverStream::new(rx))))
    }

    async fn shutdown(
        &self,
        _request: Request<ShutdownRequest>,
    ) -> Result<Response<ShutdownResponse>, Status> {
        tracing::info!("Shutdown requested via gRPC");
        self.state.shutdown_notify.notify_waiters();
        Ok(Response::new(ShutdownResponse {}))
    }
}