using Zilean.Shared.Features.Torrents;

namespace Zilean.Scraper.Features.Ingestion.Processing;

public static class TorrentInfoExtensions
{
    public static void FilterBlacklistedTorrents(
        this Dictionary<string, ExtractedDmmEntry> torrents,
        HashSet<string> blacklistedHashes,
        ILogger logger,
        ProcessedCounts processedCount)
    {
        if (torrents.Count == 0)
        {
            logger.LogDebug("No torrents to filter, skipping blacklist checks");
            return;
        }

        if (blacklistedHashes.Count == 0)
        {
            return;
        }

        int before = torrents.Count;

        logger.LogDebug("Filtering blacklisted torrents, {Count} items in blacklist", blacklistedHashes.Count);

        foreach (var torrent in blacklistedHashes)
        {
            torrents.Remove(torrent);
        }

        int removed = before - torrents.Count;

        logger.LogDebug("Filtered out {Count} blacklisted torrents", removed);
        processedCount.AddBlacklistedRemoved(removed);
    }

    public static async Task FilterExistingTorrents(this Dictionary<string, ExtractedDmmEntry> torrents, ITorrentInfoService torrentInfoService, ILogger logger)
    {
        logger.LogDebug("Filtering existing torrents");

        if (torrents.Count == 0)
        {
            logger.LogDebug("No torrents to filter, skipping");
            return;
        }

        var startCount = torrents.Count;

        var existingInfoHashes = await torrentInfoService.GetExistingInfoHashesAsync(torrents.Keys);

        foreach (var existingInfoHash in existingInfoHashes)
        {
            torrents.Remove(existingInfoHash);
        }

        logger.LogDebug("Filtered out {Count} torrents already in the database", startCount - torrents.Count);
    }
}
