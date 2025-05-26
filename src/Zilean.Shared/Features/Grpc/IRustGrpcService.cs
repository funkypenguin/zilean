namespace Zilean.Shared.Features.Grpc;

public interface IRustGrpcService
{
    Task StartServer();
    Task IngestImdbData(IngestImdbRequest ingestImdbRequest);
    Task ParseAndPopulateAsync(Dictionary<string, ExtractedDmmEntry> torrents, List<TorrentInfo> output, int batchSize = 5000);
    Task<TorrentInfo> ParseAndPopulateTorrentInfoAsync(TorrentInfo torrent);
    Task<ConcurrentQueue<TorrentInfo>?> MatchImdbIdsForBatchAsync(IEnumerable<TorrentInfo> batch, bool returnQueue = true);
    Task<SearchImdbResponse?> SearchImdbData(SearchImdbRequest searchRequest);
    Task StopServer();
}
