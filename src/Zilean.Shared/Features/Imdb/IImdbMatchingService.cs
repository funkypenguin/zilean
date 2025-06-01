namespace Zilean.Shared.Features.Imdb;

public interface IImdbMatchingService
{
    Task<ConcurrentQueue<TorrentInfo>?> MatchImdbIdsForBatchAsync(IEnumerable<TorrentInfo> batch, bool returnQueue = true);
    Task PopulateImdbData();
    void DisposeImdbData();
}
