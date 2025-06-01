namespace Zilean.Shared.Features.Torrents;

public static class TorrentInfoExtensions
{
    public static string CacheKey(this TorrentInfo torrentInfo) =>
        $"{torrentInfo.ParsedTitle}-{torrentInfo.Category}-{torrentInfo.Year}";
}
