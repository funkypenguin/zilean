namespace Zilean.Shared.Features.Grpc;

public static class RustGrpcMappers
{
    public static ParseTorrentTitleResponse ParseTorrentTitleResponse(this TorrentTitleResponse response, ILogger logger)
    {
        try
        {
            if (string.IsNullOrEmpty(response.Title))
            {
                return new(false, null);
            }

            var torrentInfo = new TorrentInfo
            {
                ParsedTitle = response.Title,
                RawTitle = response.OriginalTitle,
                NormalizedTitle = response.Title.ToLowerInvariant(),
                Audio = [.. response.Audio],
                BitDepth = response.BitDepth,
                Channels = [.. response.Channels],
                Codec = response.Codec.ToString(),
                Complete = response.Complete,
                Container = response.Container,
                Date = response.Date,
                Documentary = response.Documentary,
                Dubbed = response.Dubbed,
                Edition = response.Edition,
                EpisodeCode = response.EpisodeCode,
                Episodes = [.. response.Episodes],
                Extended = response.Extended,
                Extension = response.Extension,
                Group = response.Group,
                Hdr = [.. response.Hdr],
                Hardcoded = response.Hardcoded,
                Languages = [.. response.Languages.Select(x=> x.ToString())],
                Network = response.Network.ToString(),
                Proper = response.Proper,
                Quality = response.Quality.ToString(),
                Region = response.Region,
                Remastered = response.Remastered,
                Repack = response.Repack,
                Resolution = response.Resolution,
                Retail = response.Retail,
                Seasons = [.. response.Seasons],
                Site = response.Site,
                Size = response.Size,
                Subbed = response.Subbed,
                Unrated = response.Unrated,
                Upscaled = response.Upscaled,
                Volumes = [.. response.Volumes],
                Year = response.Year,
                IsAdult = response.Adult,
                Is3d = response.Is3D,
                Trash = response.Trash,
            };

            var mediaType = InferMediaType(torrentInfo);

            torrentInfo.Category = torrentInfo.IsAdult ? "xxx" :
                mediaType.Equals("movie", StringComparison.OrdinalIgnoreCase) ? "movie" : "tvSeries";

            return new(true, torrentInfo);
        }
        catch (Exception ex)
        {
            logger.LogError(ex, "Error occurred while parsing response");
            return new(false, null);
        }
    }

    private static string InferMediaType(TorrentInfo info) =>
        info.Seasons.Length > 0 || info.Episodes.Length > 0
            ? "tvSeries"
            : "movie";
}
