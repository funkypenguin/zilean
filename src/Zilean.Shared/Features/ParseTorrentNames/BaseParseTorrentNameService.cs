// using TorrentParserClient = Zilean.Proto.TorrentParser.TorrentParser.TorrentParserClient;
//
// namespace Zilean.Shared.Features.ParseTorrentNames;
//
// public abstract class BaseParseTorrentNameService(ILogger<TorrentParserClient> logger, ZileanConfiguration configuration) :
//     BaseGrpcService<TorrentParserClient>(logger),
//     IParseTorrentNameService
// {
//     protected readonly ZileanConfiguration _configuration = configuration;
//     protected override string SocketPath => "/tmp/zilean-torrent-parser.sock";
//
//     protected override async Task ShutdownClientAsync(TorrentParserClient client) => await client.ShutdownAsync(new());
//
//     public async Task ParseAndPopulateAsync(Dictionary<string, ExtractedDmmEntry> torrents, List<TorrentInfo> output, int batchSize = 5000)
//     {
//         await StartServer();
//
//         if (torrents.Count == 0)
//         {
//             Logger.LogInformation("No torrents to parse, returning empty list");
//             return;
//         }
//
//         if (batchSize <= 0)
//         {
//             throw new ArgumentOutOfRangeException(nameof(batchSize), "Batch size must be greater than zero.");
//         }
//
//         Logger.LogInformation("Starting PTT gRPC server parse with batch size: {BatchSize}", batchSize);
//
//         var stopwatch = Stopwatch.StartNew();
//
//         if (_client == null)
//         {
//             throw new InvalidOperationException("Go PTT client is not initialized.");
//         }
//
//         var call = _client.ParseTitles();
//
//         var input = Channel.CreateBounded<ExtractedDmmEntry>(batchSize);
//
//         _ = Task.Run(async () =>
//         {
//             await foreach (var torrent in input.Reader.ReadAllAsync())
//             {
//                 await call.RequestStream.WriteAsync(
//                     new()
//                     {
//                         InfoHash = torrent.InfoHash,
//                         Title = torrent.Filename,
//                     });
//             }
//
//             await call.RequestStream.CompleteAsync();
//         });
//
//         foreach (var torrent in torrents.Values)
//         {
//             await input.Writer.WriteAsync(torrent);
//         }
//
//         input.Writer.Complete();
//
//         await foreach (var response in call.ResponseStream.ReadAllAsync())
//         {
//             if (!torrents.TryGetValue(response.InfoHash, out var original))
//             {
//                 continue;
//             }
//
//             var result = ParseResult(response);
//
//             if (result.Success)
//             {
//                 result.Response.InfoHash = original.InfoHash;
//                 result.Response.Size = original.Filesize.ToString();
//                 result.Response.RawTitle = original.Filename;
//                 original.ParseResponse = result.Response;
//                 output.Add(result.Response);
//             }
//         }
//
//         Logger.LogInformation("PTT gRPC server parse complete");
//         Logger.LogInformation("Parsed {Count} torrents in {Elapsed} seconds", output.Count, stopwatch.Elapsed.TotalSeconds);
//     }
//
//     public async Task<TorrentInfo> ParseAndPopulateTorrentInfoAsync(TorrentInfo torrent)
//     {
//         await StartServer();
//
//         if (_client == null)
//         {
//             throw new InvalidOperationException("PTT gRPC client is not initialized.");
//         }
//
//         using var call = _client.ParseTitles();
//
//         await call.RequestStream.WriteAsync(new() { Title = torrent.RawTitle, InfoHash = torrent.InfoHash });
//         await call.RequestStream.CompleteAsync();
//
//         return await call.ResponseStream.MoveNext()
//             ? ParseResult(call.ResponseStream.Current).Response
//             : throw new InvalidOperationException("No response received from gRPC server");
//     }
//
//     private ParseTorrentTitleResponse ParseResult(TorrentTitleResponse response)
//     {
//         try
//         {
//             if (string.IsNullOrEmpty(response.Title))
//             {
//                 return new(false, null);
//             }
//
//             var torrentInfo = new TorrentInfo
//             {
//                 ParsedTitle = response.Title,
//                 RawTitle = response.OriginalTitle,
//                 NormalizedTitle = response.Title.ToLowerInvariant(),
//                 Audio = [.. response.Audio],
//                 BitDepth = response.BitDepth,
//                 Channels = [.. response.Channels],
//                 Codec = response.Codec,
//                 Complete = response.Complete,
//                 Container = response.Container,
//                 Date = response.Date,
//                 Documentary = response.Documentary,
//                 Dubbed = response.Dubbed,
//                 Edition = response.Edition,
//                 EpisodeCode = response.EpisodeCode,
//                 Episodes = [.. response.Episodes],
//                 Extended = response.Extended,
//                 Extension = response.Extension,
//                 Group = response.Group,
//                 Hdr = [.. response.Hdr],
//                 Hardcoded = response.Hardcoded,
//                 Languages = [.. response.Languages],
//                 Network = response.Network,
//                 Proper = response.Proper,
//                 Quality = response.Quality,
//                 Region = response.Region,
//                 Remastered = response.Remastered,
//                 Repack = response.Repack,
//                 Resolution = response.Resolution,
//                 Retail = response.Retail,
//                 Seasons = [.. response.Seasons],
//                 Site = response.Site,
//                 Size = response.Size,
//                 Subbed = response.Subbed,
//                 Unrated = response.Unrated,
//                 Upscaled = response.Upscaled,
//                 Volumes = [.. response.Volumes],
//                 Year = int.TryParse(response.Year, out var year) ? year : 0,
//             };
//
//             var mediaType = InferMediaType(torrentInfo);
//
//             torrentInfo.Category = torrentInfo.IsAdult ? "xxx" :
//                 mediaType.Equals("movie", StringComparison.OrdinalIgnoreCase) ? "movie" : "tvSeries";
//
//             return new(true, torrentInfo);
//         }
//         catch (Exception ex)
//         {
//             Logger.LogError(ex, "Error occurred while parsing response");
//             return new(false, null);
//         }
//     }
//
//     private static string InferMediaType(TorrentInfo info) =>
//         info.Seasons.Length > 0 || info.Episodes.Length > 0
//             ? "tv"
//             : "movie";
// }
