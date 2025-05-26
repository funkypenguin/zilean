using GrpcClient = Zilean.Proto.RustServer.ZileanRustServer.ZileanRustServerClient;

namespace Zilean.Shared.Features.Grpc;

public class RustGrpcService(ILogger<GrpcClient> logger, ZileanConfiguration configuration)
    : BaseGrpcService<GrpcClient>(logger), IRustGrpcService
{
    private const string AppBinary = "/app/zilean_rust";
    private readonly ObjectPool<ConcurrentDictionary<string, string?>> _imdbCache = new DefaultObjectPoolProvider().Create<ConcurrentDictionary<string, string?>>();
    protected override string SocketPath => "/tmp/zilean_rust.sock";
    protected override async Task ShutdownClientAsync(GrpcClient client) => await client.ShutdownAsync(new());
    public override async Task StartServer()
    {
        if (_isInitialized)
        {
            return;
        }

        logger.LogInformation("Starting Zilean Rust server...");

        _grpcCts = new();

        var dbConnectionString = GetDatabaseUrl(configuration);

        var environmentalVariables = new Dictionary<string, string?>
        {
            ["RUST_LOG"] = "info",
            ["ZILEAN_DATABASE_URL"] = dbConnectionString,
            ["ZILEAN_PARSING_THREADS"] = configuration.Parsing.ParsingThreads.ToString(),
        };

        logger.LogDebug("Using database connection string: {DbConnectionString}", dbConnectionString);

        _ = Cli.Wrap(AppBinary)
            .WithEnvironmentVariables(environmentalVariables)
            .WithValidation(CommandResultValidation.None)
            .WithStandardOutputPipe(PipeTarget.ToDelegate(Console.WriteLine))
            .WithStandardErrorPipe(PipeTarget.ToDelegate(Console.WriteLine))
            .ExecuteBufferedAsync(_grpcCts.Token);

        await PostServerInitialization();
    }

    public async Task IngestImdbData(IngestImdbRequest ingestImdbRequest)
    {
        await StartServer();

        if (_client is null)
        {
            throw new InvalidOperationException("Rust gRPC client is not initialized.");
        }

        await _client.IngestImdbAsync(ingestImdbRequest, cancellationToken: _grpcCts?.Token ?? CancellationToken.None);
    }

    public async Task ParseAndPopulateAsync(Dictionary<string, ExtractedDmmEntry> torrents, List<TorrentInfo> output, int batchSize = 5000)
    {
        await StartServer();

        if (torrents.Count == 0)
        {
            logger.LogInformation("No torrents to parse, returning empty list");
            return;
        }

        if (batchSize <= 0)
        {
            throw new ArgumentOutOfRangeException(nameof(batchSize), "Batch size must be greater than zero.");
        }

        logger.LogDebug("Parsing titles with batch size: {BatchSize}", batchSize);

        if (_client == null)
        {
            throw new InvalidOperationException("Go PTT client is not initialized.");
        }

        var call = _client.ParseTorrentTitles();

        var input = Channel.CreateBounded<ExtractedDmmEntry>(batchSize);

        _ = Task.Run(async () =>
        {
            await foreach (var torrent in input.Reader.ReadAllAsync())
            {
                await call.RequestStream.WriteAsync(
                    new()
                    {
                        InfoHash = torrent.InfoHash,
                        Title = torrent.Filename,
                    });
            }

            await call.RequestStream.CompleteAsync();
        });

        foreach (var torrent in torrents.Values)
        {
            await input.Writer.WriteAsync(torrent);
        }

        input.Writer.Complete();

        await foreach (var response in call.ResponseStream.ReadAllAsync())
        {
            if (!torrents.TryGetValue(response.InfoHash, out var original))
            {
                continue;
            }

            var result = ParseResult(response);

            if (result.Success)
            {
                result.Response.InfoHash = original.InfoHash;
                result.Response.Size = original.Filesize.ToString();
                result.Response.RawTitle = original.Filename;
                original.ParseResponse = result.Response;
                output.Add(result.Response);
            }
        }
    }

    public async Task<TorrentInfo> ParseAndPopulateTorrentInfoAsync(TorrentInfo torrent)
    {
        await StartServer();

        if (_client == null)
        {
            throw new InvalidOperationException("Rust gRPC client is not initialized.");
        }

        using var call = _client.ParseTorrentTitles();

        await call.RequestStream.WriteAsync(new() { Title = torrent.RawTitle, InfoHash = torrent.InfoHash });
        await call.RequestStream.CompleteAsync();

        return await call.ResponseStream.MoveNext()
            ? ParseResult(call.ResponseStream.Current).Response
            : throw new InvalidOperationException("No response received from gRPC server");
    }

    private ParseTorrentTitleResponse ParseResult(TorrentTitleResponse response)
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
            ? "tv"
            : "movie";

    private static string GetDatabaseUrl(ZileanConfiguration zileanConfiguration)
    {
        var ef = zileanConfiguration.Database.ConnectionString;
        var builder = new Npgsql.NpgsqlConnectionStringBuilder(ef);

        var user = Uri.EscapeDataString(builder.Username);
        var password = Uri.EscapeDataString(builder.Password);
        var host = builder.Host;
        var port = builder.Port > 0 ? builder.Port : 5432;
        var database = builder.Database;

        return $"postgres://{user}:{password}@{host}:{port}/{database}";
    }

    public async Task<ConcurrentQueue<TorrentInfo>?> MatchImdbIdsForBatchAsync(IEnumerable<TorrentInfo> batch, bool returnQueue = true)
    {
        var imdbCache = _imdbCache.Get();

        try
        {
            var parallelOptions = new ParallelOptions
            {
                MaxDegreeOfParallelism = configuration.Imdb.UseAllCores switch
                {
                    true => Environment.ProcessorCount,
                    false => configuration.Imdb.NumberOfCores,
                },
            };

            var updatedTorrents = new ConcurrentQueue<TorrentInfo>();

            var groupedByYearAndCategory = batch.GroupBy(t => new
            {
                t.Year,
                t.Category,
            });

            await Parallel.ForEachAsync(
                groupedByYearAndCategory, parallelOptions, async (torrentGroup, _) =>
                {
                    foreach (var torrent in torrentGroup)
                    {
                        if (imdbCache.TryGetValue(torrent.CacheKey(), out var imdbId))
                        {
                            torrent.ImdbId = imdbId;
                            continue;
                        }

                        var bestMatches = await SearchImdbData(new()
                        {
                            Title = torrent.NormalizedTitle,
                            Year = torrent.Year ?? 0,
                            Category = torrent.Category,
                        });

                        if (bestMatches.Matches.Count == 0)
                        {
                            logger.NoSuitableMatchFound(torrent.NormalizedTitle, torrent.Category);
                            continue;
                        }

                        var bestMatch = bestMatches.Matches.ElementAt(0);

                        if (bestMatch.ImdbId != torrent.ImdbId)
                        {
                            logger.TorrentUpdated(
                                torrent.NormalizedTitle,
                                torrent.ImdbId,
                                bestMatch.ImdbId,
                                bestMatch.Score,
                                torrent.Category,
                                bestMatch.Title,
                                bestMatch.Year);

                            torrent.ImdbId = bestMatch.ImdbId;
                            imdbCache[torrent.CacheKey()] = bestMatch.ImdbId;

                            if (returnQueue)
                            {
                                updatedTorrents.Enqueue(torrent);
                            }
                        }
                        else
                        {
                            logger.TorrentRetained(
                                torrent.NormalizedTitle,
                                torrent.ImdbId,
                                bestMatch.Score,
                                torrent.Category,
                                bestMatch.Title,
                                bestMatch.Year);
                        }
                    }
                });


            return returnQueue ? updatedTorrents : null;
        }
        finally
        {
            imdbCache.Clear();
            _imdbCache.Return(imdbCache);
        }
    }

    public async Task<SearchImdbResponse?> SearchImdbData(SearchImdbRequest searchRequest)
    {
        await StartServer();

        return _client is null
            ? throw new InvalidOperationException("Rust gRPC client is not initialized.")
            : await _client.SearchImdbAsync(searchRequest, cancellationToken: _grpcCts?.Token ?? CancellationToken.None);
    }
}
