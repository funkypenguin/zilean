namespace Zilean.Database.Services;

public class TorrentInfoService(
    ILogger<TorrentInfoService> logger,
    ZileanConfiguration configuration,
    IServiceProvider serviceProvider)
    : BaseDapperService(logger, configuration), ITorrentInfoService
{
    private readonly ObjectPool<List<TorrentInfo[]>> _torrentChunkListPool =
        new DefaultObjectPoolProvider().Create<List<TorrentInfo[]>>();

    private readonly ObjectPool<HashSet<string>> _torrentFilterHashsets = new DefaultObjectPoolProvider().Create<HashSet<string>>();

    public async Task VaccumTorrentsIndexes(CancellationToken cancellationToken)
    {
        await using var serviceScope = serviceProvider.CreateAsyncScope();
        await using var dbContext = serviceScope.ServiceProvider.GetRequiredService<ZileanDbContext>();

        await dbContext.Database.ExecuteSqlRawAsync(
            "VACUUM (VERBOSE, ANALYZE) \"Torrents\"", cancellationToken: cancellationToken);
    }

    public async Task StoreTorrentInfo(List<TorrentInfo> torrents, int batchSize)
    {
        if (torrents.Count == 0)
        {
            logger.LogInformation("No torrents to store");
            return;
        }

        foreach (var torrentInfo in torrents)
        {
            torrentInfo.CleanedParsedTitle = Parsing.CleanQuery(torrentInfo.ParsedTitle);
        }

        await using var serviceScope = serviceProvider.CreateAsyncScope();
        await using var dbContext = serviceScope.ServiceProvider.GetRequiredService<ZileanDbContext>();
        await using var connection = new NpgsqlConnection(Configuration.Database.ConnectionString);

        var bulkConfig = new BulkConfig
        {
            SetOutputIdentity = false,
            BatchSize = batchSize,
            PropertiesToIncludeOnUpdate = [string.Empty],
            UpdateByProperties = ["InfoHash"],
            BulkCopyTimeout = 0,
            TrackingEntities = false,
        };

        dbContext.Database.SetCommandTimeout(0);

        var chunkList = GetTorrentChunksFromPool(torrents, batchSize);

        await StoreBatchesInDb(chunkList, dbContext, bulkConfig);

        ReturnTorrentChunksToPool(chunkList);
    }

    public async Task BulkCopyTorrentsAsync(List<TorrentInfo> torrents, CancellationToken cancellationToken)
    {
        await using var conn = new NpgsqlConnection(Configuration.Database.ConnectionString);
        await conn.OpenAsync(cancellationToken);

        var infoHashes = _torrentFilterHashsets.Get();
        var existing = _torrentFilterHashsets.Get();

        try
        {
            foreach (var torrentInfo in torrents)
            {
                infoHashes.Add(torrentInfo.InfoHash);
            }

            await GetExistingInfoHashesAsync(conn, existing, infoHashes, cancellationToken);

            var filtered = torrents.Where(t => !existing.Contains(t.InfoHash));

            await using var writer = await conn.BeginBinaryImportAsync(
                """
                COPY "Torrents" (
                            "InfoHash", "RawTitle", "ParsedTitle", "NormalizedTitle", "CleanedParsedTitle", "Trash", "Year",
                            "Resolution", "Seasons", "Episodes", "Complete", "Volumes", "Languages", "Quality", "Hdr",
                            "Codec", "Audio", "Channels", "Dubbed", "Subbed", "Date", "Group", "Edition", "BitDepth",
                            "Bitrate", "Network", "Extended", "Converted", "Hardcoded", "Region", "Ppv", "Is3d", "Site",
                            "Size", "Proper", "Repack", "Retail", "Upscaled", "Remastered", "Unrated", "Documentary",
                            "EpisodeCode", "Country", "Container", "Extension", "Torrent", "Category", "ImdbId",
                            "IsAdult", "IngestedAt"
                        ) FROM STDIN (FORMAT BINARY)
                """, cancellationToken);

            foreach (var t in filtered)
            {
                await writer.StartRowAsync(cancellationToken);
                await writer.WriteAsync(t.InfoHash, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.RawTitle, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.ParsedTitle, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.NormalizedTitle, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(Parsing.CleanQuery(t.ParsedTitle), NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Trash ?? false, NpgsqlDbType.Boolean, cancellationToken);
                await writer.WriteAsync(t.Year, NpgsqlDbType.Integer, cancellationToken);
                await writer.WriteAsync(t.Resolution, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Seasons, NpgsqlDbType.Array | NpgsqlDbType.Integer, cancellationToken);
                await writer.WriteAsync(t.Episodes, NpgsqlDbType.Array | NpgsqlDbType.Integer, cancellationToken);
                await writer.WriteAsync(t.Complete ?? false, NpgsqlDbType.Boolean, cancellationToken);
                await writer.WriteAsync(t.Volumes, NpgsqlDbType.Array | NpgsqlDbType.Integer, cancellationToken);
                await writer.WriteAsync(t.Languages, NpgsqlDbType.Array | NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Quality, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Hdr, NpgsqlDbType.Array | NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Codec, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Audio, NpgsqlDbType.Array | NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Channels, NpgsqlDbType.Array | NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Dubbed ?? false, NpgsqlDbType.Boolean, cancellationToken);
                await writer.WriteAsync(t.Subbed ?? false, NpgsqlDbType.Boolean, cancellationToken);
                await writer.WriteAsync(t.Date, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Group, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Edition, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.BitDepth, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Bitrate, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Network, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Extended ?? false, NpgsqlDbType.Boolean, cancellationToken);
                await writer.WriteAsync(t.Converted ?? false, NpgsqlDbType.Boolean, cancellationToken);
                await writer.WriteAsync(t.Hardcoded ?? false, NpgsqlDbType.Boolean, cancellationToken);
                await writer.WriteAsync(t.Region, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Ppv ?? false, NpgsqlDbType.Boolean, cancellationToken);
                await writer.WriteAsync(t.Is3d ?? false, NpgsqlDbType.Boolean, cancellationToken);
                await writer.WriteAsync(t.Site, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Size, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Proper ?? false, NpgsqlDbType.Boolean, cancellationToken);
                await writer.WriteAsync(t.Repack ?? false, NpgsqlDbType.Boolean, cancellationToken);
                await writer.WriteAsync(t.Retail ?? false, NpgsqlDbType.Boolean, cancellationToken);
                await writer.WriteAsync(t.Upscaled ?? false, NpgsqlDbType.Boolean, cancellationToken);
                await writer.WriteAsync(t.Remastered ?? false, NpgsqlDbType.Boolean, cancellationToken);
                await writer.WriteAsync(t.Unrated ?? false, NpgsqlDbType.Boolean, cancellationToken);
                await writer.WriteAsync(t.Documentary ?? false, NpgsqlDbType.Boolean, cancellationToken);
                await writer.WriteAsync(t.EpisodeCode, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Country, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Container, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Extension, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.Torrent ?? false, NpgsqlDbType.Boolean, cancellationToken);
                await writer.WriteAsync(t.Category, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.ImdbId, NpgsqlDbType.Text, cancellationToken);
                await writer.WriteAsync(t.IsAdult, NpgsqlDbType.Boolean, cancellationToken);
                await writer.WriteAsync(t.IngestedAt, NpgsqlDbType.TimestampTz, cancellationToken);
            }

            await writer.CompleteAsync(cancellationToken);
        }
        catch (Exception e)
        {
            logger.LogError(e, "Error during bulk copy of torrents");
        }
        finally
        {
            infoHashes.Clear();
            existing.Clear();
            _torrentFilterHashsets.Return(infoHashes);
            _torrentFilterHashsets.Return(existing);
        }
    }

    private async Task StoreBatchesInDb(List<TorrentInfo[]> chunks, ZileanDbContext dbContext, BulkConfig bulkConfig)
    {
        var currentBatch = 0;

        foreach (var batch in chunks)
        {
            currentBatch++;
            logger.LogInformation("Storing batch {CurrentBatch} of {TotalBatches}", currentBatch, chunks.Count);
            await dbContext.BulkInsertOrUpdateAsync(batch, bulkConfig);
        }
    }

    public async Task<TorrentInfo[]> SearchForTorrentInfoByOnlyTitle(string query)
    {
        var cleanQuery = Parsing.CleanQuery(query);

        return await ExecuteCommandAsync(
            async connection =>
            {
                var sql =
                    """
                    SELECT
                        *
                    FROM "Torrents"
                    WHERE "ParsedTitle" % @query
                    AND Length("InfoHash") = 40
                    LIMIT 100;
                    """;

                var parameters = new DynamicParameters();

                parameters.Add("@query", cleanQuery);

                var result = await connection.QueryAsync<TorrentInfo>(sql, parameters);

                return result.ToArray();
            }, "Error finding unfiltered dmm entries.");
    }

    public async Task<TorrentInfo[]> SearchForTorrentInfoFiltered(TorrentInfoFilter filter, int? limit = null)
    {
        var cleanQuery = Parsing.CleanQuery(filter.Query);
        var imdbId = EnsureCorrectFormatImdbId(filter);

        return await ExecuteCommandAsync(
            async connection =>
            {
                const string sql =
                    """
                       SELECT *
                       FROM search_torrents_meta(
                           @Query,
                           @Season,
                           @Episode,
                           @Year,
                           @Language,
                           @Resolution,
                           @ImdbId,
                           @Limit,
                           @Category,
                           @SimilarityThreshold
                       );
                    """;

                var parameters = new DynamicParameters();

                parameters.Add("@Query", cleanQuery);
                parameters.Add("@Season", filter.Season);
                parameters.Add("@Episode", filter.Episode);
                parameters.Add("@Year", filter.Year);
                parameters.Add("@Language", filter.Language);
                parameters.Add("@Resolution", filter.Resolution);
                parameters.Add("@Category", filter.Category);
                parameters.Add("@ImdbId", imdbId);
                parameters.Add("@Limit", limit ?? Configuration.Dmm.MaxFilteredResults);
                parameters.Add("@SimilarityThreshold", (float)Configuration.Dmm.MinimumScoreMatch);

                var results = await connection.QueryAsync<TorrentInfoResult>(sql, parameters);

                // assign imdb to torrent info
                return results.Select(MapImdbDataToTorrentInfo).ToArray();
            }, "Error finding unfiltered dmm entries.");
    }

    private static string? EnsureCorrectFormatImdbId(TorrentInfoFilter filter)
    {
        string? imdbId = null;

        if (!string.IsNullOrEmpty(filter.ImdbId))
        {
            imdbId = filter.ImdbId.StartsWith("tt") ? filter.ImdbId : $"tt{filter.ImdbId}";
        }

        return imdbId;
    }

    private static Func<TorrentInfoResult, TorrentInfoResult> MapImdbDataToTorrentInfo =>
        torrentInfo =>
        {
            if (torrentInfo.ImdbId != null)
            {
                torrentInfo.Imdb = new()
                {
                    ImdbId = torrentInfo.ImdbId,
                    Category = torrentInfo.ImdbCategory,
                    Title = torrentInfo.ImdbTitle,
                    Year = torrentInfo.ImdbYear ?? 0,
                    Adult = torrentInfo.ImdbAdult,
                };
            }

            return torrentInfo;
        };

    public async Task<HashSet<string>> GetExistingInfoHashesAsync(IEnumerable<string> infoHashes)
    {
        await using var serviceScope = serviceProvider.CreateAsyncScope();
        await using var dbContext = serviceScope.ServiceProvider.GetRequiredService<ZileanDbContext>();

        var existingHashes = await dbContext.Torrents
            .Where(t => infoHashes.Contains(t.InfoHash))
            .Select(t => t.InfoHash)
            .ToListAsync();

        return [..existingHashes];
    }

    public async Task<HashSet<string>> GetBlacklistedItems()
    {
        await using var serviceScope = serviceProvider.CreateAsyncScope();
        await using var dbContext = serviceScope.ServiceProvider.GetRequiredService<ZileanDbContext>();

        var existingHashes = await dbContext.BlacklistedItems
            .Select(t => t.InfoHash)
            .ToListAsync();

        return [..existingHashes];
    }

    private List<TorrentInfo[]> GetTorrentChunksFromPool(List<TorrentInfo> torrents, int batchSize)
    {
        var chunkList = _torrentChunkListPool.Get();
        chunkList.AddRange(torrents.Chunk(batchSize));
        return chunkList;
    }

    private void ReturnTorrentChunksToPool(List<TorrentInfo[]> chunkList)
    {
        chunkList.Clear();
        _torrentChunkListPool.Return(chunkList);
    }

    private async Task<HashSet<string>> GetExistingInfoHashesAsync(NpgsqlConnection conn,
        HashSet<string> results,
        IEnumerable<string> hashes,
        CancellationToken cancellationToken)
    {
        const int batchSize = 10_000;
        var hashList = hashes.ToList();

        for (int i = 0; i < hashList.Count; i += batchSize)
        {
            var batch = hashList.Skip(i).Take(batchSize).ToArray();

            var existing = await conn.QueryAsync<string>(
                """
                SELECT "InfoHash" FROM "Torrents" WHERE "InfoHash" = ANY(@Hashes)
                """,
                new
                {
                    Hashes = batch
                });

            foreach (var h in existing)
            {
                results.Add(h);
            }
        }

        return results;
    }
}
