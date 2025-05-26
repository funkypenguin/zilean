namespace Zilean.Shared.Features.Imdb;

public interface IImdbMatchingService
{
    public Task<ConcurrentQueue<TorrentInfo>?> MatchImdbIdsForBatchAsync(IEnumerable<TorrentInfo> batch, bool returnQueue = true);
    public Task PopulateImdbData();
    public void DisposeImdbData();
}
