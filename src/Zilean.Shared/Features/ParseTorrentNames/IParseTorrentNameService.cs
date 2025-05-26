namespace Zilean.Shared.Features.ParseTorrentNames;

public interface IParseTorrentNameService
{
    Task StopServer();
    public Task ParseAndPopulateAsync(Dictionary<string, ExtractedDmmEntry> torrents, List<TorrentInfo> output, int batchSize = 5000);
    public Task<TorrentInfo> ParseAndPopulateTorrentInfoAsync(TorrentInfo torrent);
}
