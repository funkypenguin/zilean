namespace Zilean.Shared.Features.Torrents;

public interface ITorrentInfoService
{
    Task StoreTorrentInfo(List<TorrentInfo> torrents, int batchSize);
    Task BulkCopyTorrentsAsync(List<TorrentInfo> torrents, CancellationToken cancellationToken);
    Task<TorrentInfo[]> SearchForTorrentInfoByOnlyTitle(string query);
    Task<TorrentInfo[]> SearchForTorrentInfoFiltered(TorrentInfoFilter filter, int? limit = null);
    Task<HashSet<string>> GetExistingInfoHashesAsync(IEnumerable<string> infoHashes);
    Task<HashSet<string>> GetBlacklistedItems();
    Task VaccumTorrentsIndexes(CancellationToken cancellationToken);
}
