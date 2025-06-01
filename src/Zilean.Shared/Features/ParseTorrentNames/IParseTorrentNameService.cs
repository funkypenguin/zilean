namespace Zilean.Shared.Features.ParseTorrentNames;

public interface IParseTorrentNameService
{
    Task StopServer();
    Task ParseAndPopulateAsync(Dictionary<string, ExtractedDmmEntry> torrents, List<TorrentInfo> output, int batchSize = 5000);
    Task<TorrentInfo> ParseAndPopulateTorrentInfoAsync(TorrentInfo torrent);
}
